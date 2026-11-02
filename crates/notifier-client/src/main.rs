//#![windows_subsystem = "windows"]

use std::{env::current_exe, path::PathBuf, time::Duration};

use color_eyre::{Result, eyre::OptionExt};
use futures_util::StreamExt;
use notifier_common::WebSocketAction;
use shortcuts_rs::ShellLink;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        Message,
        client::IntoClientRequest,
        http::{HeaderValue, header::USER_AGENT},
    },
};
use winrt_toast_reborn::{Toast, ToastManager, register};

type NotifierMessage = notifier_common::Message;

const APP_ID: &str = "com.shoudev.cicin.evil.notifier";
const ID: &str = "test";
const BASE_URL: &str = "ws://localhost:3000";

#[tokio::main]
async fn main() -> Result<()> {
    register(APP_ID, "Evil notifier", None)?;
    setup()?;

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
            WebSocketAction::NoAction => {}
        }
    }
    Ok(())
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
        return Ok(());
    }

    let current_exectutable = current_exe()?;

    let link = ShellLink::new(current_exectutable, None, None, None)?;
    link.create_lnk(link_path)?;

    Ok(())
}
