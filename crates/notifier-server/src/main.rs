use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    self, Json, Router,
    body::Bytes,
    extract::{
        ConnectInfo, Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{any, get, post},
};
use axum_extra::{
    TypedHeader,
    headers::{self, Header},
};
use color_eyre::eyre::Result;
use notifier_common::{PuppyModeConfig, WebSocketAction};
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
type Db = Arc<Mutex<HashMap<String, ClientState>>>;

#[derive(Clone)]
struct ClientState {
    sender: UnboundedSender<WebSocketAction>,
    puppy_mode_config: PuppyModeConfig,
}

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
        .route("/clients/{id}/puppyfier", get(get_puppy_config))
        .route("/clients/{id}/puppyfier", post(set_puppy_config))
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
    real_ip: Option<TypedHeader<XRealIp>>,
    State(db): State<Db>,
) -> impl IntoResponse {
    let user_agent = match user_agent {
        Some(ua) => ua.to_string(),
        None => "unknown".to_owned(),
    };

    let address = real_ip.map_or(addr.to_string(), |ip| {
        format!("{}:{}", ip.0.0, addr.to_string())
    });

    let id = format!("{}:{}", address, user_agent);

    info!("\"{}\" connected", id);

    let (sender, reciever) = unbounded_channel::<WebSocketAction>();
    let state = ClientState {
        sender,
        puppy_mode_config: PuppyModeConfig {
            enabled: false,
            mode: notifier_common::PuppyMode::Insertion(Default::default()),
        },
    };

    db.lock().await.insert(id.clone(), state);

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
    let state = match db.get(&request.target) {
        Some(state) => state,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let action = WebSocketAction::Message(NotifierMessage {
        from: request.from,
        text: request.message,
    });

    let result = state.sender.send(action);

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
    for (key, state) in db
        .iter_mut()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>()
    {
        if state.sender.send(WebSocketAction::NoAction).is_err() {
            db.remove(&key);
        };
    }
}

async fn get_puppy_config(Path(id): Path<String>, State(db): State<Db>) -> impl IntoResponse {
    let db = db.lock().await;
    let state = match db.get(&id) {
        Some(state) => state,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    Json(state.puppy_mode_config.clone()).into_response()
}

async fn set_puppy_config(
    Path(id): Path<String>,
    State(db): State<Db>,
    Json(request): Json<PuppyModeConfig>,
) -> impl IntoResponse {
    let mut db = db.lock().await;
    let mut state = match db.get(&id) {
        Some(state) => state.to_owned(),
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let _result = state
        .sender
        .send(WebSocketAction::PuppyModeConfig(request.clone()));

    state.puppy_mode_config = request;

    db.insert(id, state);

    StatusCode::OK.into_response()
}

struct XRealIp(String);

static X_REAL_IP: HeaderName = HeaderName::from_static("x-real-ip");

impl Header for XRealIp {
    fn name() -> &'static axum::http::HeaderName {
        &X_REAL_IP
    }

    fn decode<'i, I>(values: &mut I) -> std::prelude::v1::Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;

        Ok(Self(value.to_str().unwrap_or_default().to_owned()))
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        let value = HeaderValue::from_str(&self.0).unwrap_or(HeaderValue::from_static(""));
        values.extend(std::iter::once(value));
    }
}
