use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use poem::{
    get, handler,
    listener::TcpListener,
    middleware::Tracing,
    web::{
        websocket::{Message, WebSocket},
        Data, RealIp, RemoteAddr,
    },
    EndpointExt, IntoResponse, Route, Server,
};
use tracing::*;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "server=trace,poem=debug");
    }
    tracing_subscriber::fmt::init();

    let (websocket_sender, _) = tokio::sync::broadcast::channel::<String>(32);

    let app = Route::new()
        .at("/websocket", get(websocket.data(websocket_sender)))
        .with(Tracing);
    Server::new(TcpListener::bind("127.0.0.1:8001"))
        .name("hello-world")
        .run(app)
        .await?;

    Ok(())
}

#[handler]
fn websocket(
    ws: WebSocket,
    remote_addr: &RemoteAddr,
    sender: Data<&tokio::sync::broadcast::Sender<String>>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    debug!("incoming websocket connection from: {remote_addr}, id={id}");

    let sender = sender.clone();
    let mut receiver = sender.subscribe();

    ws.on_upgrade(move |socket| async move {
        let (mut sink, mut stream) = socket.split();

        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                match msg {
                    Ok(msg) => match msg {
                        poem::web::websocket::Message::Text(msg) => {
                            debug!("received message from websocket {id}: {msg}");
                            if let Err(e) = sender.send(format!("{id}:{msg}")) {
                                error!("error broadcasting to other websockets on websocket {id}: {e:?}");
                            }
                        }
                        poem::web::websocket::Message::Binary(msg) => todo!(),
                        _ => (),
                    },
                    Err(e) => error!("error receiving from websocket {id}: {e:?}"),
                }
            }
        });

        tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                if let Err(e) = sink.send(Message::Text(format!("{id}:{msg}"))).await {
                    error!("error sending to websocket client on websocket {id}: {e:?}");
                    break;
                }
            }
        });
    })
}
