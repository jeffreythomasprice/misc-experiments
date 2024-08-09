use std::sync::Arc;

use futures_util::StreamExt;
use poem::{
    handler,
    web::{websocket::WebSocket, Data, RemoteAddr},
    IntoResponse,
};
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use tracing::*;
use uuid::Uuid;

use crate::{service::websockets::split_websocket_stream, AppState};

pub struct ActiveWebsocket {
    id: Uuid,
    sender: tokio::sync::mpsc::Sender<WebsocketServerToClientMessage>,
}

#[handler]
pub fn websocket(
    Data(state): Data<&Arc<AppState>>,
    ws: WebSocket,
    remote_addr: &RemoteAddr,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    debug!("incoming websocket connection from: {remote_addr}, id={id}");

    let active_websockets = state.active_websockets.clone();
    ws.on_upgrade(move |socket| async move {
        let (sender, mut stream) = split_websocket_stream::<
            WebsocketServerToClientMessage,
            WebsocketClientToServerMessage,
        >(socket);

        {
            let active_websockets = active_websockets.clone();
            tokio::spawn(async move {
                while let Some(msg) = stream.next().await {
                    debug!("received incoming websocket message: {msg:?}");
                    let new_msg =
                        WebsocketServerToClientMessage::Message(format!("response to: {:?}", msg));
                    let active_websockets = active_websockets.lock().unwrap();
                    for (_, ws) in active_websockets.iter() {
                        debug!("sending to {}, msg = {:?}", ws.id, new_msg.clone());
                        let ws = ws.clone();
                        let new_msg = new_msg.clone();
                        tokio::spawn(async move {
                            if let Err(e) = ws.sender.send(new_msg).await {
                                error!("error sending message to websocket: {e:?}");
                            }
                        });
                    }
                }

                debug!("websocket closed {id:?}");
                let mut active_websockets = active_websockets.lock().unwrap();
                active_websockets.remove(&id);
            });
        }

        let mut active_websockets = active_websockets.lock().unwrap();
        active_websockets.insert(id, Arc::new(ActiveWebsocket { id, sender }));
    })
}
