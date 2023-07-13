use self::types::*;
use crate::client::PortClient;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::sync::Mutex;

mod types;

pub struct TransmissionCredentials {
    pub username: String,
    pub password: String,
}

pub struct TransmissionClient {
    client: Client,
    url: String,
    credentials: Option<TransmissionCredentials>,
    session_id: Mutex<Option<String>>,
}

impl TransmissionClient {
    pub fn new(base_url: &str, credentials: Option<TransmissionCredentials>) -> Result<Self> {
        let client = ClientBuilder::new().gzip(true).build()?;

        let url = match base_url {
            url if url.ends_with("/transmission/rpc") => url.to_string(),
            url if url.ends_with("/") => format!("{}transmission/rpc", url),
            url => format!("{}/transmission/rpc", url),
        };

        Ok(Self {
            client,
            url,
            credentials,
            session_id: Mutex::new(None),
        })
    }

    async fn request_impl<T: DeserializeOwned>(
        &self,
        method: &str,
        arguments: Value,
    ) -> Result<Option<T>> {
        let mut request_builder = self
            .client
            .post(&self.url)
            .json(&TransmissionRequest { method, arguments });

        if let Some(ref session_id) = *self.session_id.lock().await {
            request_builder = request_builder.header("X-Transmission-Session-Id", session_id);
        };

        if let Some(TransmissionCredentials { username, password }) = &self.credentials {
            request_builder = request_builder.basic_auth(username, Some(password));
        }

        let res = request_builder.send().await?;

        if let Some(session_id) = res.headers().get("X-Transmission-Session-Id") {
            *self.session_id.lock().await = Some(session_id.to_str()?.to_string());
        }

        match res.error_for_status() {
            Ok(res) => {
                let res: TransmissionResponse = res.json().await?;
                match res.result.as_str() {
                    "success" => Ok(Some(serde_json::from_value(res.arguments)?)),
                    error => Err(anyhow!("Transmission request error: {}", error)),
                }
            }
            Err(err) if err.status() == Some(StatusCode::CONFLICT) => {
                match *self.session_id.lock().await {
                    Some(_) => Ok(None),
                    None => Err(anyhow!("Transmission session ID is missing")),
                }
            }
            Err(err) => Err(err.into()),
        }
    }

    async fn request<T: DeserializeOwned>(&self, method: &str, params: Value) -> Result<T> {
        // start off by getting the session ID if we don't have it yet
        if self.session_id.lock().await.is_none() && method != "session-get" {
            self.request_impl::<Value>("session-get", json!({})).await?;
        };

        // normal request
        let res = self.request_impl::<T>(method, params.clone()).await?;

        if let Some(res) = res {
            return Ok(res);
        }

        // retry request with session new ID
        let res = self.request_impl::<T>(method, params).await?;

        Ok(res.expect("Transmission session ID is missing"))
    }

    pub async fn get_version(&self) -> Result<String> {
        let res: TransmissionSessionArgumentsVersion =
            self.request("session-get", json!({})).await?;
        Ok(res.version)
    }

    pub async fn get_session_arguments(&self) -> Result<TransmissionSessionArgumentsPort> {
        self.request("session-get", json!({})).await
    }

    pub async fn set_session_arguments(
        &self,
        peer_port_random_on_start: bool,
        peer_port: u64,
    ) -> Result<()> {
        let arguments = TransmissionSessionArgumentsPort {
            peer_port,
            peer_port_random_on_start,
        };
        self.request::<Value>("session-set", json!(arguments))
            .await
            .map(|_| ())
    }
}

#[async_trait]
impl PortClient for TransmissionClient {
    async fn get_port(&self) -> Result<Option<u64>> {
        let arguments = self.get_session_arguments().await?;
        match arguments.peer_port_random_on_start {
            true => Ok(None),
            false => Ok(Some(arguments.peer_port)),
        }
    }

    async fn set_port(&self, port: u64) -> Result<()> {
        self.set_session_arguments(false, port).await
    }
}
