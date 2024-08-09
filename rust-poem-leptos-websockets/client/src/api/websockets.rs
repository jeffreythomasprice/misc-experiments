use std::{
    fmt::Debug,
    pin::Pin,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use futures::{stream, Sink, SinkExt, Stream, StreamExt};
use leptos::*;
use log::*;
use pharos::{Observable, ObserveConfig};
use serde::{de::DeserializeOwned, Serialize};
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use ws_stream_wasm::{WsErr, WsMessage, WsMeta};

#[derive(Clone)]
pub struct WebsocketService {
    url: String,
    sink: Arc<
        Mutex<
            Option<
                Pin<Box<dyn Sink<WebsocketClientToServerMessage, Error = ws_stream_wasm::WsErr>>>,
            >,
        >,
    >,
}

impl WebsocketService {
    pub fn new(url: String) -> Self {
        Self {
            url,
            sink: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(
        &mut self,
        output: impl Fn(WebsocketServerToClientMessage) + 'static,
    ) -> Result<()> {
        let (sink, mut stream) = new_websocket_with_url::<
            WebsocketClientToServerMessage,
            WebsocketServerToClientMessage,
        >(&self.url)
        .await?;

        let mut self_sink = self.sink.lock().unwrap();
        self_sink.replace(sink);

        spawn_local(async move {
            while let Some(msg) = stream.next().await {
                info!("received message from websocket: {msg:?}");
                output(msg);
            }
        });

        Ok(())
    }

    pub async fn send(&self, msg: WebsocketClientToServerMessage) -> Result<()> {
        let sink = &mut *self.sink.lock().unwrap();
        if let Some(sink) = sink {
            sink.send(msg).await?;
        }
        Ok(())
    }
}

async fn new_websocket_with_url<OutgoingMessage, IncomingMessage>(
    url: &str,
) -> Result<(
    Pin<Box<dyn Sink<OutgoingMessage, Error = WsErr>>>,
    Pin<Box<dyn Stream<Item = IncomingMessage>>>,
)>
where
    OutgoingMessage: Serialize + 'static,
    IncomingMessage: DeserializeOwned + Debug + 'static,
{
    let (mut ws, wsio) = WsMeta::connect(url, None).await?;

    let mut events = ws.observe(ObserveConfig::default()).await?;

    spawn_local(async move {
        while let Some(e) = events.next().await {
            trace!("websocket event: {e:?}");
        }
    });

    let (sink, stream) = wsio.split();

    let sink = Box::pin(sink.with_flat_map(|msg| {
        stream::iter(match serde_json::to_string(&msg) {
            Ok(msg) => vec![Ok(WsMessage::Text(msg))],
            Err(e) => {
                error!("failed to serialize outgoing websocket message: {e:}");
                Vec::new()
            }
        })
    }));

    let stream = stream
        .filter_map(|msg| async {
            match msg {
                WsMessage::Text(msg) => match serde_json::from_str::<IncomingMessage>(&msg) {
                    Ok(msg) => Some(msg),
                    Err(e) => {
                        error!("failed to deserialize incoming websocket message: {e:?}");
                        None
                    }
                },
                WsMessage::Binary(_) => todo!(),
            }
        })
        .boxed();

    Ok((sink, stream))
}
