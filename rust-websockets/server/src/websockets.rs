use axum::extract::ws::WebSocket;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::*;

pub async fn websocket_to_channels<SenderType, ReceiverType>(
    socket: WebSocket,
) -> (Sender<SenderType>, Receiver<ReceiverType>)
where
    SenderType: 'static + Serialize + Send,
    ReceiverType: 'static + DeserializeOwned + Send,
{
    let (mut socket_sender, mut socket_receiver) = socket.split();

    let (incoming_msg_sender, inccoming_msg_receiver) = channel::<ReceiverType>(32);
    let (outgoing_msg_sender, mut outgoing_msg_receiver) = channel::<SenderType>(32);

    let mut sender_task = tokio::spawn(async move {
        while let Some(msg) = outgoing_msg_receiver.recv().await {
            let send_future = match serde_json::to_string(&msg) {
                Ok(str) => socket_sender.send(axum::extract::ws::Message::Text(str)),
                Err(e) => {
                    error!("error serializing message: {e:?}");
                    return;
                }
            };
            if let Err(e) = send_future.await {
                error!("error sending message: {e:?}");
                return;
            }
        }
    });

    let mut receiver_task = tokio::spawn(async move {
        while let Some(msg) = socket_receiver.next().await {
            let msg_bytes = match msg {
                Ok(msg) => msg.into_data(),
                Err(e) => {
                    error!("error getting message bytes: {e}");
                    return;
                }
            };
            let msg: ReceiverType = match serde_json::from_slice(&msg_bytes) {
                Ok(value) => value,
                Err(e) => {
                    error!("error parsing message: {e}");
                    return;
                }
            };
            if let Err(e) = incoming_msg_sender.send(msg).await {
                error!("error sending message on: {e:?}");
                return;
            }
        }
    });

    tokio::spawn(async move {
        tokio::select! {
            _ = (&mut sender_task) => {
                receiver_task.abort();
            },
            _ = (&mut receiver_task) => {
                sender_task.abort();
            },
        }
    });

    (outgoing_msg_sender, inccoming_msg_receiver)
}
