use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Duration,
};

fn main() {
    let port = 8000;

    let server_thread = thread::spawn(move || {
        if let Err(e) = server(format!("127.0.0.1:{port}")) {
            println!("error running server: {e:?}");
        }
    });

    let client_thread = thread::spawn(move || {
        if let Err(e) = client(format!("127.0.0.1:{port}")) {
            println!("error running client: {e:?}");
        }
    });

    server_thread.join().expect("failed to join server thread");
    client_thread.join().expect("failed to join client thread");
    println!("done");
}

fn server<A>(addr: A) -> Result<(), std::io::Error>
where
    A: ToSocketAddrs,
{
    let listener = TcpListener::bind(addr)?;
    println!("server listening on {}", listener.local_addr()?);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    if let Err(e) = handle_server_stream(stream) {
                        println!("error handling stream: {e:?}");
                    }
                });
            }
            Err(e) => {
                println!("error handling stream: {e:?}")
            }
        }
    }
    Ok(())
}

fn handle_server_stream(stream: TcpStream) -> Result<(), std::io::Error> {
    let stream_description = format!(
        "server {} handling client stream from {}",
        stream.local_addr()?,
        stream.peer_addr()?
    );
    println!("{stream_description} connected");

    let (sender, receiver) = tcp_stream_to_channels(stream_description.clone(), stream)?;
    let read_thread = {
        let stream_description = stream_description.clone();
        thread::spawn(move || print_strings_from_receiver_in_loop(&stream_description, receiver))
    };
    let write_thread = {
        thread::spawn(move || {
            write_strings_to_sender_in_loop(sender, "Hello from server".to_string())
        })
    };

    read_thread.join().expect("failed to join read thread");
    write_thread.join().expect("failed to join write thread");
    println!("{stream_description} done");
    Ok(())
}

fn client<A>(addr: A) -> Result<(), std::io::Error>
where
    A: ToSocketAddrs,
{
    let stream = TcpStream::connect(addr)?;
    let stream_description = format!(
        "client stream from {} to {}",
        stream.local_addr()?,
        stream.peer_addr()?
    );
    println!("{stream_description} connected");

    let (sender, receiver) = tcp_stream_to_channels(stream_description.clone(), stream)?;
    let read_thread = {
        let stream_description = stream_description.clone();
        thread::spawn(move || print_strings_from_receiver_in_loop(&stream_description, receiver))
    };
    let write_thread = {
        thread::spawn(move || {
            write_strings_to_sender_in_loop(sender, "Hello from client".to_string())
        })
    };

    read_thread.join().expect("failed to join read thread");
    write_thread.join().expect("failed to join write thread");
    println!("{stream_description} done");
    Ok(())
}

fn tcp_stream_to_channels(
    context: String,
    mut stream: TcpStream,
) -> Result<(Sender<Vec<u8>>, Receiver<Vec<u8>>), std::io::Error> {
    stream.set_nonblocking(true)?;

    let (read_sender, read_receiver) = channel::<Vec<u8>>();
    let (write_sender, write_receiver) = channel::<Vec<u8>>();

    // TODO how to signal this thread to die?
    thread::spawn(move || {
        // TODO read buffer size could be configurable?
        let mut buf = vec![0_u8; 2048];

        // until we're done
        loop {
            // try reading
            match stream.read(buf.as_mut()) {
                // nothing to do, no bytes available
                Ok(0) => (),
                // actual data available
                Ok(n) => {
                    // TODO no need to init to 0?
                    let mut buf_copy = vec![0_u8; n];
                    buf_copy.clone_from(&buf);
                    if let Err(e) = read_sender.send(buf_copy) {
                        println!("{context}: error sending {n} bytes read to channel: {e:?}");
                    }
                }
                Err(e) => match e.kind() {
                    // nothing to do, no bytes available
                    io::ErrorKind::WouldBlock => (),
                    // real error
                    e => {
                        println!("{context}: error reading from stream: {e:?}");
                        return;
                    }
                },
            }

            // try writing
            match write_receiver.try_recv() {
                // actual data available
                Ok(buf) => {
                    let n = buf.len();
                    if let Err(e) = stream.write_all(&buf) {
                        println!("{context}: error writing {n} bytes to stream: {e:?}");
                    }
                }
                // nothing to do, no bytes available
                Err(std::sync::mpsc::TryRecvError::Empty) => (),
                // remaining error case, should just be the Disconnected case
                Err(e) => {
                    println!("{context}: error reading from write channel: {e:?}");
                    return;
                }
            }
        }
    });

    Ok((write_sender, read_receiver))
}

fn print_strings_from_receiver_in_loop(context: &str, receiver: Receiver<Vec<u8>>) {
    loop {
        match receiver.recv() {
            Ok(buf) => match std::str::from_utf8(&buf) {
                Ok(str) => println!("{context}: read: {str}"),
                Err(e) => println!(
                    "{context}: error converting {} bytes read into string {e:?}",
                    buf.len()
                ),
            },
            Err(e) => {
                println!("{context}: receive error: {e:?}");
                return;
            }
        }
    }
}

fn write_strings_to_sender_in_loop(sender: Sender<Vec<u8>>, content: String) {
    // TODO how to exit?
    loop {
        send_string(&sender, content.as_str());
        thread::sleep(Duration::from_secs(2));
    }
}

fn send_string(sender: &Sender<Vec<u8>>, s: &str) {
    if let Err(e) = sender.send(s.bytes().collect::<Vec<u8>>()) {
        println!("error sending str as bytes: {e:?}");
    }
}
