use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    self, Json, Router,
    body::Bytes,
    extract::{
        ConnectInfo, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get, post},
};
use axum_extra::{TypedHeader, headers};
use color_eyre::eyre::Result;
use notifier_common::WebSocketAction;
use serde::Deserialize;
use tokio::{
    net::TcpListener,
    sync::{
        Mutex,
        mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    },
};
use tracing::{Level, info};

type NotifierMessage = notifier_common::Message;
type Db = Arc<Mutex<HashMap<String, UnboundedSender<WebSocketAction>>>>;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let db = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .route("/send", post(send_msg))
        .route("/clients", get(get_clients))
        .route("/clients/refresh", post(test_all_clients))
        .with_state(db);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(db): State<Db>,
) -> impl IntoResponse {
    let user_agent = match user_agent {
        Some(ua) => ua.to_string(),
        None => "unknown".to_owned(),
    };

    let id = format!("{}:{}", addr.to_string(), user_agent);

    info!("\"{}\" connected", id);

    let (sender, reciever) = unbounded_channel::<WebSocketAction>();
    db.lock().await.insert(id.clone(), sender);

    ws.on_failed_upgrade(|error| tracing::error!("Error upgrading ws: {}", error))
        .on_upgrade(move |socket| handle_socket(id, socket, reciever, db))
}

async fn handle_socket(
    id: String,
    mut socket: WebSocket,
    mut reciever: UnboundedReceiver<WebSocketAction>,
    db: Db,
) {
    loop {
        let msg = match reciever.recv().await {
            Some(msg) => msg,
            None => break,
        };

        let bytes = rmp_serde::to_vec(&msg).unwrap();

        let res = socket
            .send(Message::Binary(Bytes::copy_from_slice(bytes.as_slice())))
            .await;

        if let Err(_) = res {
            break;
        }
    }
    db.lock().await.remove(&id);
}

#[derive(Deserialize)]
struct SendRequest {
    target: String,
    from: String,
    message: String,
}

async fn send_msg(State(db): State<Db>, Json(request): Json<SendRequest>) -> impl IntoResponse {
    let mut db = db.lock().await;
    let sender = match db.get(&request.target) {
        Some(sender) => sender,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let action = WebSocketAction::Message(NotifierMessage {
        from: request.from,
        text: request.message,
    });

    let result = sender.send(action);

    if let Err(err) = result {
        tracing::error!("{}", err);
        db.remove(&request.target);
        return StatusCode::CONFLICT.into_response();
    }

    StatusCode::OK.into_response()
}

async fn get_clients(State(db): State<Db>) -> impl IntoResponse {
    let db = db.lock().await;
    let clients: Vec<String> = db.keys().into_iter().map(|k| k.to_owned()).collect();
    Json(clients)
}

async fn test_all_clients(State(db): State<Db>) -> impl IntoResponse {
    let mut db = db.lock().await;
    for (key, sender) in db
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>()
    {
        if sender.send(WebSocketAction::NoAction).is_err() {
            db.remove(&key);
        };
    }
}
