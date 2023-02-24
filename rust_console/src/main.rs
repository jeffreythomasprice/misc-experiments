mod console;

use std::{
    error::Error,
    io::{self, Write},
};

use console::*;

fn print_at(mut position: Position, s: &str) -> Result<(), io::Error> {
    for line in s.lines().into_iter() {
        set_cursor_position(position)?;
        io::stdout().write(line.as_bytes())?;
        position.y += 1;
    }
    io::stdout().flush()?;
    Ok(())
}

struct RectangleStyle {
    interior: u8,
    left: u8,
    right: u8,
    top: u8,
    bottom: u8,
    top_left: u8,
    top_right: u8,
    bottom_left: u8,
    bottom_right: u8,
}

fn draw_rect(position: Position, size: Size, style: RectangleStyle) -> Result<(), Box<dyn Error>> {
    let mut content = Vec::with_capacity(
        (
            // the size of the rectangle
            size.width * size.height + 
        // plus the newlines between rows
        (size.height - 1)
        )
        .into(),
    );
    for y in position.y..(position.y + size.height) {
        let is_top = y == position.y;
        let is_bottom = y == position.y + size.height - 1;
        for x in position.x..(position.x + size.width) {
            let is_left = x == position.x;
            let is_right = x == position.x + size.width - 1;
            content.push(if is_top && is_left {
                style.top_left
            } else if is_top && is_right {
                style.top_right
            } else if is_top {
                style.top
            } else if is_bottom && is_left {
                style.bottom_left
            } else if is_bottom && is_right {
                style.bottom_right
            } else if is_bottom {
                style.bottom
            } else if is_left {
                style.left
            } else if is_right {
                style.right
            } else {
                style.interior
            });
        }
    }
    print_at(position, std::str::from_utf8(&content)?)?;
    Ok(())
}

fn main() {
    fn do_it() -> Result<(), Box<dyn Error>> {
        set_raw_mode()?;

        draw_rect(
            Position { x: 0, y: 0 },
            get_window_size()?,
            RectangleStyle {
                interior: b' ',
                left: b'<',
                right: b'>',
                top: b'^',
                bottom: b'v',
                top_left: b'1',
                top_right: b'2',
                bottom_left: b'3',
                bottom_right: b'4',
            },
        )?;

        print_at(
            Position { x: 5, y: 4 },
            format!("window size = {}\n", get_window_size()?).as_str(),
        )?;
        print_at(
            Position { x: 5, y: 5 },
            format!("cursor = {}\n", get_cursor_position()?).as_str(),
        )?;

        print_at(
            Position { x: 10, y: 7 },
            format!("multi-line output\nfoo\nbar\n\nbaz").as_str(),
        )?;

        Ok(())
    }

    let e = do_it();
    set_cooked_mode().unwrap();
    e.unwrap();
}
