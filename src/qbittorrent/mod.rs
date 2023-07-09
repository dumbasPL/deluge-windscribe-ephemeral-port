use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use reqwest::{Client, ClientBuilder, Method, Response, StatusCode};
use serde::Serialize;

use self::types::{
    QBittorrentLoginRequest, QBittorrentPreferences, QBittorrentSetPreferencesRequest,
};

mod types;

pub struct QBittorrentClient {
    client: Client,
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

    async fn request_impl<T: Serialize>(
        &self,
        method: Method,
        url: &str,
        form: Option<&T>,
    ) -> Result<Response> {
        let mut request_builder = self
            .client
            .request(method.clone(), format!("{}{}", self.url, url));

        if let Some(form) = form {
            request_builder = request_builder.form(form);
        }

        Ok(request_builder.send().await?)
    }

    #[async_recursion]
    async fn request<T: Serialize + Send + Sync>(
        &self,
        method: Method,
        url: &str,
        form: Option<T>,
    ) -> Result<Response> {
        let res = self
            .request_impl::<T>(method.clone(), url, form.as_ref())
            .await?;

        match res.status() {
            // qBittorrent returns 403 when not logged in
            StatusCode::FORBIDDEN => {
                self.login().await?;
                self.request_impl::<T>(method, url, form.as_ref()).await
            }
            _ => Ok(res),
        }
    }

    pub async fn login(&self) -> Result<()> {
        let form = QBittorrentLoginRequest {
            username: &self.username,
            password: &self.password,
        };

        let res = self
            .request(Method::POST, "/auth/login", Some(form))
            .await?;

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
        let res = self
            .request(Method::GET, "/app/version", None::<()>)
            .await?;

        match res.error_for_status() {
            Ok(res) => Ok(res.text().await?),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_preferences(&self) -> Result<QBittorrentPreferences> {
        let res = self
            .request(Method::GET, "/app/preferences", None::<()>)
            .await?;

        match res.error_for_status() {
            Ok(res) => Ok(res.json().await?),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn set_listen_port(&self, listen_port: u64) -> Result<()> {
        let form = QBittorrentSetPreferencesRequest {
            json: &serde_json::to_string(&QBittorrentPreferences { listen_port })?,
        };

        let res = self
            .request(Method::POST, "/app/setPreferences", Some(form))
            .await?;

        match res.error_for_status() {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
