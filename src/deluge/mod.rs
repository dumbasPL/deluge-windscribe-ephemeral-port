use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use reqwest::{Client, ClientBuilder};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

use self::types::{DelugeConfig, DelugeHost, DelugeRequest, DelugeResponse};

mod types;

pub struct DelugeClient {
    client: Client,
    url: String,
    password: String,
    request_id: Arc<Mutex<u32>>,
}

impl DelugeClient {
    pub fn new(base_url: &str, password: &str) -> Result<Self> {
        let client = ClientBuilder::new().gzip(true).cookie_store(true).build()?;
        let url = match base_url {
            url if url.ends_with("/json") => url.to_string(),
            url if url.ends_with("/") => format!("{}json", url),
            url => format!("{}/json", url),
        };
        Ok(Self {
            client,
            url,
            password: password.to_string(),
            request_id: Arc::new(Mutex::new(0)),
        })
    }

    fn get_next_request_id(&self) -> u32 {
        let mut request_id = self.request_id.lock().unwrap();
        *request_id += 1;

        // not sure if this is necessary, but just in case
        if *request_id > 0x1000 {
            *request_id = 1;
        }
        *request_id
    }

    #[async_recursion]
    async fn request(&self, method: &str, params: &[Value]) -> Result<Value> {
        let request = DelugeRequest {
            method: method.to_string(),
            params: params.to_vec(),
            id: self.get_next_request_id(),
        };

        let res: DelugeResponse = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        match res.id {
            id if id == request.id => (),
            _ => return Err(anyhow!("Invalid response from Deluge")),
        }

        match res.error {
            Some(error) => match error.code {
                // Not authenticated
                1 if method != "auth.login" => {
                    self.login().await?;
                    self.request(method, params).await
                }
                code => Err(anyhow!("Deluge error {}: {}", code, error.message)),
            },
            None => Ok(res.result),
        }
    }

    pub async fn login(&self) -> Result<()> {
        let params = vec![json!(self.password)];
        let hosts = self.request("auth.login", &params).await?;

        match hosts {
            Value::Bool(success) => {
                if success {
                    Ok(())
                } else {
                    Err(anyhow!("Invalid password"))
                }
            }
            _ => Err(anyhow!("Invalid response from Deluge")),
        }
    }

    pub async fn get_hosts(&self) -> Result<Vec<DelugeHost>> {
        let hosts = self.request("web.get_hosts", &[]).await?;

        match hosts {
            Value::Array(hosts) => hosts
                .iter()
                .map(|host| {
                    let data_array = host
                        .as_array()
                        .and_then(|x| if x.len() == 4 { Some(x) } else { None })
                        .ok_or(anyhow!("Invalid response from Deluge"))?;

                    let id = data_array[0]
                        .as_str()
                        .ok_or(anyhow!("Invalid response from Deluge"))?
                        .to_string();

                    let ip = data_array[1]
                        .as_str()
                        .ok_or(anyhow!("Invalid response from Deluge"))?
                        .to_string();

                    let port = data_array[2]
                        .as_u64()
                        .ok_or(anyhow!("Invalid response from Deluge"))?;

                    let name = data_array[3]
                        .as_str()
                        .ok_or(anyhow!("Invalid response from Deluge"))?
                        .to_string();

                    Ok(DelugeHost { id, ip, port, name })
                })
                .collect(),
            _ => Err(anyhow!("Invalid response from Deluge")),
        }
    }

    pub async fn connect(&self, host_id: Option<&str>) -> Result<()> {
        let host_id = match host_id {
            Some(host_id) => host_id.to_string(),
            None => self
                .get_hosts()
                .await?
                .first()
                .ok_or(anyhow!("No Deluge hosts found"))?
                .id
                .clone(),
        };

        let params = vec![json!(host_id)];
        self.request("web.connect", &params).await?;

        Ok(())
    }

    pub async fn get_port_config(&self) -> Result<DelugeConfig> {
        let params = vec![json!([json!("random_port"), json!("listen_ports")])];
        let config = self.request("core.get_config_values", &params).await?;

        serde_json::from_value(config).map_err(|_| anyhow!("Invalid response from Deluge"))
    }

    pub async fn set_port_config(&self, random_port: bool, listen_port: u64) -> Result<()> {
        let params = vec![json!(DelugeConfig {
            random_port,
            listen_ports: [listen_port, listen_port],
        })];
        self.request("core.set_config", &params).await?;

        Ok(())
    }

    pub async fn get_version(&self) -> Result<String> {
        let version = self.request("daemon.get_version", &vec![]).await?;

        match version {
            Value::String(version) => Ok(version),
            _ => Err(anyhow!("Invalid response from Deluge")),
        }
    }
}
