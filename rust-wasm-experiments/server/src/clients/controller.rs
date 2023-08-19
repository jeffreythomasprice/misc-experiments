use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    response::IntoResponse,
    Json, TypedHeader,
};
use futures::{sink::SinkExt, stream::StreamExt};
use shared::models::messages::{ClientWebsocketMessage, CreateClientRequest, CreateClientResponse};
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
    let (sender, mut receiver) = socket.split();

    // TODO needs sender stuff
    // let send_task = tokio::spawn(async move {
    //     loop {

    //     }
    //     });

    let receiver_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            let msg_bytes = match msg {
                Ok(msg) => msg.into_data(),
                Err(e) => {
                    error!("TODO JEFF error getting message bytes: {e}");
                    continue;
                }
            };
            let msg: ClientWebsocketMessage = match serde_json::from_slice(&msg_bytes) {
                Ok(value) => value,
                Err(e) => {
                    error!("TODO JEFF error parsing message: {e}");
                    continue;
                }
            };
            info!("TODO JEFF msg = {msg:?}");
        }
    });

    // TODO should be using tokio::select! and aborting the other task when the first closes
    receiver_task.await.unwrap();

    // TODO can't read auth header from protocol, but shohuld do something like this on first message

    // let protocol = match socket.protocol() {
    //     Some(header) => match header.to_str() {
    //         Ok(header) => header,
    //         Err(e) => {
    //             error!("failed to get string value for protocol: {e:?}");
    //             return;
    //         }
    //     },
    //     None => {
    //         error!("no protocol provided");
    //         return;
    //     }
    // };

    // let claims = match auth_service.validate(protocol) {
    //     Ok(claims) => claims,
    //     Err(e) => {
    //         error!("failed to parse protocol as claims: {e:?}");
    //         return;
    //     }
    // };
    // debug!("accepting new client connection: {claims:?}");
}
