
use anyhow::{anyhow, Result};
use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    SinkExt, StreamExt,
};
use leptos::spawn_local;
use log::*;
use serde::{de::DeserializeOwned, Serialize};

pub async fn connect<S, R>(url: &str) -> Result<(Sender<S>, Receiver<R>)>
where
    S: Serialize + 'static,
    R: DeserializeOwned + 'static,
{
    let websocket = gloo::net::websocket::futures::WebSocket::open(url)
        .map_err(|e| anyhow!("failed to create websocket, url: {}, error: {:?}", url, e))?;

    let (mut sender, mut receiver) = websocket.split();

    let (output_sender, mut output_receiver) = channel::<S>(1);
    {
        let url = url.to_owned();
        spawn_local(async move {
            while let Some(message) = output_receiver.next().await {
                match serde_json::to_string(&message) {
                    Ok(message) => {
                        if let Err(e) = sender.send(gloo::net::websocket::Message::Text(message)).await {
                            error!("error sending string message to websocket, url: {}, error: {:?}", url, e);
                        }
                    }
                    Err(e) => error!(
                        "error serializing outgoing websocket  message to json for websocket, url: {}, error: {:?}",
                        url, e
                    ),
                };
            }
        });
    }

    let (mut input_sender, input_receiver) = channel::<R>(1);
    {
        let url = url.to_owned();
        spawn_local(async move {
            while let Some(message) = receiver.next().await {
                match message {
                    Ok(gloo::net::websocket::Message::Text(message)) => {
                        match serde_json::from_str(&message) {
                            Ok(message) => {
                                if let Err(e) = input_sender.send(message).await {
                                    error!(
                                        "error sending incoming message to channel for websocket, url: {}, error: {:?}",
                                        url, e
                                    );
                                }
                            }
                            Err(e) => error!("error deserializing incoming websocket message, url: {}, error: {:?}", url, e),
                        };
                    }
                    Ok(gloo::net::websocket::Message::Bytes(_message)) => {
                        todo!("handle binary messages");
                    }
                    Err(e) => {
                        error!("error receiving from websocket, url: {}, error: {:?}", url, e);
                        break;
                    }
                };
            }
        });
    }

    Ok((output_sender, input_receiver))
}
