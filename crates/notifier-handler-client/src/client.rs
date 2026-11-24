use color_eyre::Result;
use notifier_common::PuppyModeConfig;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

pub struct NotifierClient {
    base_url: String,
    client: Client,
}

impl NotifierClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::new();
        Self {
            base_url: base_url.to_owned(),
            client,
        }
    }

    pub fn refresh_clients(&self) -> Result<()> {
        self.client
            .post(self.build_url("/clients/refresh"))
            .send()?;
        Ok(())
    }

    pub fn get_clients(&self) -> Result<Vec<String>> {
        let response = self
            .client
            .get(self.build_url("/clients"))
            .send()
            .map(|r| r.json())
            .flatten()?;
        Ok(response)
    }

    pub fn send(&self, target: &str, from: &str, message: &str) -> Result<()> {
        let request = SendRequest {
            target: target.to_owned(),
            from: from.to_owned(),
            message: message.to_owned(),
        };
        self.client
            .post(self.build_url("/send"))
            .json(&request)
            .send()?;
        Ok(())
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub fn set_puppy_mode(&self, target: &str, request: PuppyModeConfig) -> Result<()> {
        self.client
            .post(self.build_url(&format!("/clients/{}/puppyfier", target)))
            .json(&request)
            .send()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct SendRequest {
    target: String,
    from: String,
    message: String,
}
