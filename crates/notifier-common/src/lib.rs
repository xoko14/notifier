use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum WebSocketAction {
    Message(Message),
    PuppyModeConfig(PuppyModeConfig),
    NoAction,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClientConfig {
    pub real_ip: Option<String>,
    pub puppy_mode: PuppyModeConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PuppyModeConfig {
    pub enabled: bool,
    pub mode: PuppyMode,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PuppyReplacementMode {
    pub probability: u32,
    pub characters: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PuppyInsertionMode {
    pub probability: u32,
    pub puppy_words: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PuppyMode {
    Replacement(PuppyReplacementMode),
    Insertion(PuppyInsertionMode),
}

impl Default for PuppyReplacementMode {
    fn default() -> Self {
        Self {
            probability: 20,
            characters: "wrofau".to_string(),
        }
    }
}

impl Default for PuppyInsertionMode {
    fn default() -> Self {
        Self {
            probability: 20,
            puppy_words: vec![
                "woof".to_owned(),
                "arf".to_owned(),
                "wroof".to_owned(),
                "bark".to_owned(),
                "awooooo".to_owned(),
            ],
        }
    }
}

impl Default for PuppyModeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: PuppyMode::Replacement(Default::default()),
        }
    }
}
