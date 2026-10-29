use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum WebSocketAction {
    Message(Message),
    NoAction,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub text: String,
}
