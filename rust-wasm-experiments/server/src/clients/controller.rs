use std::time::Duration;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::stream::StreamExt;
use shared::models::messages::{
    ClientWebsocketMessage, CreateClientRequest, CreateClientResponse, ServerWebsocketMessage,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::*;

use crate::{auth, models::GenericErrorResponse};

use super::{Service, ServiceError};

impl From<ServiceError> for GenericErrorResponse {
    fn from(value: ServiceError) -> Self {
        // TODO different status codes for different kinds of errors
        GenericErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR, value.to_string())
    }
}

pub async fn create(
    State(mut service): State<Service>,
    State(auth_service): State<auth::Service>,
    // TODO needs extract to get client id from auth header
    request: Json<CreateClientRequest>,
) -> Result<Json<CreateClientResponse>, GenericErrorResponse> {
    let result = service.create(request.name.clone())?;
    let token = auth_service.create(&result)?;
    Ok(Json(CreateClientResponse {
        id: result.id.to_string(),
        token: token,
    }))
}

pub async fn websocket(
    State(client_service): State<Service>,
    State(auth_service): State<auth::Service>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(client_service, auth_service, socket))
}

async fn handle_websocket(
    client_service: Service,
    auth_service: auth::Service,
    mut socket: WebSocket,
) {
    let (sender, mut receiver) = websocket_to_channels(socket).await;

    let first_message = match tokio::time::timeout(Duration::from_secs(2), receiver.recv()).await {
        Ok(value) => match value {
            Some(value) => value,
            None => {
                error!("websocket closed before receiving initial message");
                return;
            }
        },
        Err(_) => {
            error!("did not receive initial websocket message within timeout");
            return;
        }
    };

    let claims = match first_message {
        ClientWebsocketMessage::Authenticate { token } => {
            let claims = match auth_service.validate(&token) {
                Ok(claims) => claims,
                Err(e) => {
                    error!("failed to parse protocol as claims: {e:?}");
                    return;
                }
            };
            debug!("incoming client jwt claims: {claims:?}");
            claims
        }
    };

    // TODO JEFF actually look up client by id

    // TODO JEFF start the main loop, reading messages and handling them, ignore auth messages
}

async fn websocket_to_channels(
    socket: WebSocket,
) -> (
    Sender<ServerWebsocketMessage>,
    Receiver<ClientWebsocketMessage>,
) {
    let (socket_sender, mut socket_receiver) = socket.split();

    let (incoming_msg_sender, inccoming_msg_receiver) = channel(32);
    let (outgoing_msg_sender, mut outgoing_msg_receiver) = channel(32);

    let mut sender_task = tokio::spawn(async move {
        while let Some(msg) = outgoing_msg_receiver.recv().await {
            // TODO write msg to socket_sender
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
            let msg: ClientWebsocketMessage = match serde_json::from_slice(&msg_bytes) {
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
