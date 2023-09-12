use tokio::sync::mpsc::{Receiver, Sender};

pub mod client;
pub mod server;

pub enum Message {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug)]
pub enum Error {
    Unit(()),
    String(String),
    Io(std::io::Error),
}

pub trait WebSocketChannel {
    fn split(&mut self) -> (Sender<Message>, Receiver<Result<Message, Error>>);
}

impl From<()> for Error {
    fn from(_value: ()) -> Self {
        Self::Unit(())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
