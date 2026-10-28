use std::time::Duration;

use color_eyre::Result;
use futures_util::{SinkExt, StreamExt};
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

#[tokio::main]
async fn main() -> Result<()> {
    register(APP_ID, "Evil notifier", None)?;

    let manager = ToastManager::new(APP_ID);

    loop {
        _ = ws_logic(&manager).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn ws_logic(manager: &ToastManager) -> Result<()> {
    let mut request = "ws://127.0.0.1:3000/ws".into_client_request()?;
    request
        .headers_mut()
        .insert(USER_AGENT, HeaderValue::from_static("Test"));
    let (ws_stream, _) = connect_async(request).await?;

    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(Ok(msg)) = receiver.next().await {
        let bytes = match msg {
            Message::Binary(bytes) => bytes,
            _ => continue,
        };

        let message: NotifierMessage = rmp_serde::from_slice(bytes.iter().as_slice()).unwrap();

        let mut toast = Toast::new();
        toast
            .text1(format!("{} says", message.from))
            .text2(message.text);

        manager.show(&toast)?;
    }
    Ok(())
}
