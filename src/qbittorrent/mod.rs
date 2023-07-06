use anyhow::{anyhow, Result};
use reqwest::{Client, ClientBuilder};

use self::types::{
    QBittorrentLoginRequest, QBittorrentPreferences, QBittorrentSetPreferencesRequest,
};

mod types;

pub struct QBittorrentClient {
    pub client: Client,
    url: String,
    username: String,
    password: String,
}

impl QBittorrentClient {
    pub fn new(base_url: &str, username: &str, password: &str) -> Result<Self> {
        let client = ClientBuilder::new().gzip(true).cookie_store(true).build()?;
        let url = match base_url {
            url if url.ends_with("/api/v2") => url.to_string(),
            url if url.ends_with("/") => format!("{}api/v2", url),
            url => format!("{}/api/v2", url),
        };

        Ok(Self {
            client,
            url,
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    pub async fn login(&self) -> Result<()> {
        let url = format!("{}/auth/login", self.url);
        let form = QBittorrentLoginRequest {
            username: &self.username,
            password: &self.password,
        };

        let res = self.client.post(url).form(&form).send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let body = res.text().await?;
                match body.as_str() {
                    "Ok." => Ok(()),
                    "Fails." => Err(anyhow!("Invalid username or password")),
                    _ => Err(anyhow!("Invalid response from server")),
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_version(&self) -> Result<String> {
        let url = format!("{}/app/version", self.url);
        let res = self.client.get(url).send().await?;

        match res.error_for_status() {
            Ok(res) => Ok(res.text().await?),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_preferences(&self) -> Result<QBittorrentPreferences> {
        let url = format!("{}/app/preferences", self.url);
        let res = self.client.get(url).send().await?;

        match res.error_for_status() {
            Ok(res) => Ok(res.json().await?),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn set_listen_port(&self, listen_port: u64) -> Result<()> {
        let url = format!("{}/app/setPreferences", self.url);
        let json = serde_json::to_string(&QBittorrentPreferences { listen_port })?;
        let form = QBittorrentSetPreferencesRequest { json: &json };

        let res = self.client.post(url).form(&form).send().await?;

        match res.error_for_status() {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
