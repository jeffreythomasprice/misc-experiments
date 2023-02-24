use std::{
    error::Error,
    fmt::{Debug, Display},
    io::{self, Write},
};

#[derive(Debug)]
struct ReturnCodeError {
    operation: String,
    return_code: i32,
}

impl Display for ReturnCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ReturnCodeError {{ operation = {:?}, return_code = {:?} }}",
            self.operation, self.return_code
        )
    }
}

impl Error for ReturnCodeError {}

#[derive(Debug)]
struct ExpectedValueError<ExpectedType, ActualType> {
    expected: ExpectedType,
    actual: ActualType,
}

impl<ExpectedType, ActualType> Display for ExpectedValueError<ExpectedType, ActualType>
where
    ExpectedType: Debug,
    ActualType: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ExpectedValueError {{ expected = {:?}, actual = {:?} }}",
            self.expected, self.actual
        )
    }
}

impl<ExpectedType, ActualType> std::error::Error for ExpectedValueError<ExpectedType, ActualType>
where
    ExpectedType: Debug,
    ActualType: Debug,
{
}

fn get_window_size() -> Result<libc::winsize, ReturnCodeError> {
    let result = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe {
        match libc::ioctl(libc::STDIN_FILENO, libc::TIOCGWINSZ, &result) {
            0 => Ok(result),
            return_code => Err(ReturnCodeError {
                operation: "ioctl TIOCGWINSZ".to_string(),
                return_code,
            }),
        }
    }
}

#[derive(Debug)]
struct Position {
    row: u16,
    column: u16,
}

fn get_cursor_position() -> Result<Position, Box<dyn Error>> {
    io::stdout().write(&[0x1b, b'[', b'6', b'n'])?;
    io::stdout().flush()?;

    wait_for_u8(0x1b);
    expect_u8(b'[')?;
    let row = std::str::from_utf8(expect_until_u8(b';', 4)?.as_ref())?.parse::<u16>()?;
    let column = std::str::from_utf8(expect_until_u8(b'R', 4)?.as_ref())?.parse::<u16>()?;
    Ok(Position { row, column })
}

fn set_cursor_position(p: Position) -> Result<(), io::Error> {
    io::stdout().write(&[0x1b])?;
    io::stdout().write(format!("[{};{}H", p.row, p.column).as_bytes())?;
    io::stdout().flush()?;
    Ok(())
}

fn wait_for_u8(value: u8) {
    loop {
        let result: Result<u8, _> = unsafe { libc::getchar() }.try_into();
        match result {
            Ok(result) => {
                if result == value {
                    return;
                }
            }
            Err(_) => continue,
        }
    }
}

fn expect_u8(expected: u8) -> Result<(), ExpectedValueError<u8, i32>> {
    let actual_i32 = unsafe { libc::getchar() };
    let actual_u8: Result<u8, _> = actual_i32.try_into();
    match actual_u8 {
        Ok(actual_u8) => {
            if actual_u8 == expected {
                Ok(())
            } else {
                Err(ExpectedValueError {
                    expected,
                    actual: actual_i32,
                })
            }
        }
        Err(_) => Err(ExpectedValueError {
            expected,
            actual: actual_i32,
        }),
    }
}

fn expect_until_u8(expected: u8, max: usize) -> Result<Vec<u8>, ExpectedValueError<u8, i32>> {
    let mut results = Vec::new();
    for _ in 0..max {
        let next_i32 = unsafe { libc::getchar() };
        let next_u8: Result<u8, _> = next_i32.try_into();
        match next_u8 {
            Ok(next_u8) => {
                if next_u8 == expected {
                    break;
                } else {
                    results.push(next_u8);
                }
            }
            _ => Err(ExpectedValueError {
                expected: expected,
                actual: next_i32,
            })?,
        }
    }
    Ok(results)
}

fn set_raw_mode() -> Result<(), ReturnCodeError> {
    // set raw mode
    // https://github.com/wertarbyte/coreutils/blob/master/src/stty.c#L1180
    let mut termios = get_termios()?;
    termios.c_iflag = 0;
    termios.c_oflag &= !libc::OPOST;
    termios.c_lflag &= !(libc::ISIG | libc::ICANON | libc::ECHO);
    termios.c_cc[libc::VMIN] = 1;
    termios.c_cc[libc::VTIME] = 0;
    set_termios(termios, libc::TCSADRAIN)?;
    Ok(())
}

fn set_cooked_mode() -> Result<(), ReturnCodeError> {
    // set cooked mode
    // https://github.com/wertarbyte/coreutils/blob/master/src/stty.c#L1167
    let mut termios = get_termios()?;
    termios.c_iflag |= libc::BRKINT | libc::IGNPAR | libc::ISTRIP | libc::ICRNL | libc::IXON;
    termios.c_oflag |= libc::OPOST;
    termios.c_lflag |= libc::ISIG | libc::ICANON | libc::ECHO;
    set_termios(termios, libc::TCSANOW)?;
    Ok(())
}

fn get_termios() -> Result<libc::termios, ReturnCodeError> {
    let mut result = libc::termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c_ispeed: 0,
        c_ospeed: 0,
    };
    match unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut result) } {
        0 => Ok(result),
        return_code => Err(ReturnCodeError {
            operation: "get termios".to_string(),
            return_code,
        }),
    }
}

fn set_termios(value: libc::termios, optional_actions: libc::c_int) -> Result<(), ReturnCodeError> {
    match unsafe { libc::tcsetattr(libc::STDIN_FILENO, optional_actions, &value) } {
        0 => Ok(()),
        return_code => Err(ReturnCodeError {
            operation: "set termios".to_string(),
            return_code,
        }),
    }
}

fn main() {
    fn do_it() -> Result<(), Box<dyn Error>> {
        set_raw_mode()?;

        let winsize = get_window_size()?;
        println!("winsize = {} x {}\r", winsize.ws_col, winsize.ws_row);

        let cursor = get_cursor_position()?;
        println!("cursor = {:?}\r", cursor);

        set_cursor_position(Position { row: 4, column: 5 })?;
        println!("moved cursor!");

        Ok(())
    }

    let e = do_it();
    set_cooked_mode().unwrap();
    e.unwrap();
}
