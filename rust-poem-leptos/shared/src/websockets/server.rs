#![cfg(feature = "server")]

use futures_util::{SinkExt, StreamExt};
use log::*;
use poem::{
    web::{websocket::WebSocket, RemoteAddr},
    IntoResponse,
};
use tokio::{
    spawn,
    sync::mpsc::{channel, Receiver, Sender},
};
use tokio_stream::wrappers::ReceiverStream;

use crate::websockets::{Error, Message};

pub struct WebSocketChannel {
    incoming_messages_receiver: Option<Receiver<Result<Message, Error>>>,
    outgoing_messages_sender: Option<Sender<Message>>,
}

impl super::WebSocketChannel for WebSocketChannel {
    fn split(&mut self) -> (Sender<Message>, Receiver<Result<Message, Error>>) {
        (
            self.outgoing_messages_sender
                .take()
                .expect("cannot split twice"),
            self.incoming_messages_receiver
                .take()
                .expect("cannot split twice"),
        )
    }
}

pub fn handler<F>(ws: WebSocket, remote_addr: &RemoteAddr, f: F) -> impl IntoResponse
where
    F: FnOnce(WebSocketChannel) + Send + Sync + 'static,
{
    let remote_addr = remote_addr.to_string();
    ws.on_upgrade(|stream| async move {
        trace!("websocket upgraded: {remote_addr}");

        let (mut original_sink, original_stream) = stream.split();

        let (incoming_messages_sender, incoming_messages_receiver) = channel(1);
        let (outgoing_messages_sender, outgoing_messages_receiver) = channel(1);

        spawn(async move {
            original_stream
                .filter_map(|message| async {
                    match message {
                        Ok(poem::web::websocket::Message::Text(value)) => {
                            Some(Ok(Message::Text(value)))
                        }
                        Ok(poem::web::websocket::Message::Binary(value)) => {
                            Some(Ok(Message::Binary(value)))
                        }
                        Ok(poem::web::websocket::Message::Ping(_)) => None,
                        Ok(poem::web::websocket::Message::Pong(_)) => None,
                        Ok(poem::web::websocket::Message::Close(Some((code, reason)))) => {
                            trace!("websocket closed, code={code:?}, reason={reason}");
                            None
                        }
                        Ok(poem::web::websocket::Message::Close(None)) => {
                            trace!("websocket closed");
                            None
                        }
                        Err(e) => {
                            trace!("incoming websocket error: {e:?}");
                            Some(Err(e.into()))
                        }
                    }
                })
                .for_each(|message| async {
                    if let Err(e) = incoming_messages_sender.send(message).await {
                        error!("error in websocket handler: {e:?}");
                    }
                })
                .await;
        });

        spawn(async move {
            if let Err(e) = original_sink
                .send_all(
                    &mut ReceiverStream::new(outgoing_messages_receiver).map(
                        |message| match message {
                            Message::Text(value) => Ok(poem::web::websocket::Message::Text(value)),
                            Message::Binary(value) => {
                                Ok(poem::web::websocket::Message::Binary(value))
                            }
                        },
                    ),
                )
                .await
            {
                error!("error in websocket handler: {e:?}");
            }
        });

        let stream = WebSocketChannel {
            incoming_messages_receiver: Some(incoming_messages_receiver),
            outgoing_messages_sender: Some(outgoing_messages_sender),
        };
        f(stream);
    })
}
