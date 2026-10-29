//#![windows_subsystem = "windows"]

use std::time::Duration;

use color_eyre::Result;
use futures_util::StreamExt;
use notifier_common::WebSocketAction;
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
