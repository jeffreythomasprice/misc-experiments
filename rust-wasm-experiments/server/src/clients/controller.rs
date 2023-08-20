use std::{str::FromStr, time::Duration};

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use shared::models::messages::{
    ClientWebsocketMessage, CreateClientRequest, CreateClientResponse, ServerWebsocketMessage,
};

use tracing::*;
use uuid::Uuid;

use crate::{auth, models::GenericErrorResponse, websockets::websocket_to_channels};

use super::{Service, ServiceError};

impl From<ServiceError> for GenericErrorResponse {
    fn from(value: ServiceError) -> Self {
        let status_code = match value {
            ServiceError::DuplicateId(_) => StatusCode::BAD_REQUEST,
            ServiceError::NoSuchId(_) => StatusCode::NOT_FOUND,
        };
        GenericErrorResponse::new(status_code, value.to_string())
    }
}

pub async fn create(
    State(mut service): State<Service>,
    State(auth_service): State<auth::Service>,
    request: Json<CreateClientRequest>,
) -> Result<Json<CreateClientResponse>, GenericErrorResponse> {
    let result = service.create(request.name.clone()).await?;
    let token = auth_service.create(&result)?;
    Ok(Json(CreateClientResponse {
        id: result.id.to_string(),
        token,
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
    mut client_service: Service,
    auth_service: auth::Service,
    socket: WebSocket,
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
        ClientWebsocketMessage::Authenticate(token) => {
            let claims = match auth_service.validate(&token) {
                Ok(claims) => claims,
                Err(e) => {
                    error!("auth token didn't pass validation: {e:?}");
                    return;
                }
            };
            debug!("incoming client jwt claims: {claims:?}");
            claims
        }
        _ => {
            error!("first message wasn't an auth token");
            return;
        }
    };

    let client_id = match Uuid::from_str(&claims.id) {
        Ok(result) => result,
        Err(e) => {
            error!("failed to parse id as uuid: {e:?}");
            return;
        }
    };
    trace!("id: {client_id}");

    if let Err(e) = client_service.update_with_sender(client_id, sender).await {
        error!("failed to update client with new websocket channel: {e:?}");
        return;
    }

    let mut heartbeat_task = {
        let mut client_service = client_service.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                if let Err(e) = client_service.update_last_seen_time(client_id).await {
                    error!("error trying to update client last seen time: {e}");
                    return;
                }
            }
        })
    };

    let mut receiver_task = {
        let client_service = client_service.clone();
        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    ClientWebsocketMessage::Authenticate(_) => {
                        warn!("client is already connected, extra auth message received: {msg:?}")
                    }
                    ClientWebsocketMessage::Message(message) => {
                        info!("TODO JEFF message: {message}");
                        let client_service = client_service.clone();
                        tokio::spawn(async move {
                            client_service
                                .broadcast(ServerWebsocketMessage::Message {
                                    sender_id: client_id.to_string(),
                                    message,
                                })
                                .await;
                        });
                    }
                }
            }
        })
    };

    tokio::spawn(async move {
        tokio::select! {
            _ = (&mut heartbeat_task) => {
                receiver_task.abort();
            },
            _ = (&mut receiver_task) => {
                heartbeat_task.abort();
            },
        }
        client_service.delete(client_id).await;
    });
}
