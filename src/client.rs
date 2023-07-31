use crate::{
    config::{ClientConfig, ClientConfigType},
    deluge::DelugeClient,
    exec::ExecClient,
    qbittorrent::QBittorrentClient,
    transmission::{TransmissionClient, TransmissionCredentials},
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[async_trait]
pub trait PortClient: Send + Sync {
    /// Get the port that the client is listening on.
    /// None if port is set to random/not possible to retrieve.
    async fn get_port(&self) -> Result<Option<u64>>;

    /// Set the port that the client is listening on.
    async fn set_port(&self, port: u64) -> Result<()>;

    /// Update the port that the client is listening on.
    /// Returns true if the port was updated.
    async fn update_port(&self, port: u64) -> Result<bool> {
        let current_port = self.get_port().await?;
        match current_port {
            Some(current_port) if current_port == port => Ok(false),
            _ => {
                self.set_port(port).await?;
                Ok(true)
            }
        }
    }
}

pub fn create_port_client(config: &ClientConfigType) -> Result<Box<dyn PortClient>> {
    let client: Result<Box<dyn PortClient>> = match config {
        ClientConfigType::Deluge(config) => {
            Ok(Box::new(DelugeClient::new(&config.url, &config.password)?))
        }
        ClientConfigType::QBittorrent(config) => Ok(Box::new(QBittorrentClient::new(
            &config.url,
            &config.username,
            &config.password,
        )?)),
        ClientConfigType::Transmission(config) => {
            let credentials = match (&config.username, &config.password) {
                (Some(username), Some(password)) => Ok(Some(TransmissionCredentials {
                    username: username.clone(),
                    password: password.clone(),
                })),
                (Some(_), None) | (None, Some(_)) => Err(anyhow!(
                    "Transmission username and password must both be specified or neither."
                )),
                (None, None) => Ok(None),
            }?;
            Ok(Box::new(TransmissionClient::new(&config.url, credentials)?))
        }
        ClientConfigType::Exec(config) => Ok(Box::new(ExecClient::new(&config.command)?)),
    };
    client
}

pub struct TimedPortClient {
    client: Arc<Box<dyn PortClient>>,
    name: String,
    check_interval: Option<Duration>,
    desired_port: Mutex<Option<u64>>,
}

impl TimedPortClient {
    pub fn new(client_config: &ClientConfig) -> Self {
        let client =
            create_port_client(&client_config.config).expect("Failed to create port client");
        Self {
            client: Arc::new(client),
            name: client_config.name.clone(),
            check_interval: client_config.check_interval.map(Duration::from_secs),
            desired_port: Mutex::new(None),
        }
    }

    pub async fn update(&self, new_port: Option<u64>) -> Result<bool> {
        let mut desired_port = self.desired_port.lock().await;
        if let Some(port) = new_port {
            *desired_port = Some(port);
        }

        if let Some(port) = *desired_port {
            let updated = self.client.update_port(port).await?;
            Ok(updated)
        } else {
            Ok(false)
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub async fn port(&self) -> Option<u64> {
        *self.desired_port.lock().await
    }

    pub fn check_interval(&self) -> Option<Duration> {
        self.check_interval
    }
}
