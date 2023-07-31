use crate::cache::SimpleCache;
use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    env::{current_dir, current_exe},
    path::PathBuf,
};
use tokio::fs;
use tracing::warn;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub windscribe: WindscribeConfig,
    pub clients: Vec<ClientConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindscribeConfig {
    pub username: String,
    pub password: String,
    pub check_interval: Option<u64>,
    pub retry_delay: u64,
    pub extra_delay: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub name: String,
    pub check_interval: Option<u64>,
    #[serde(flatten)]
    pub config: ClientConfigType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "config", rename_all = "lowercase")]
pub enum ClientConfigType {
    Deluge(DelugeClientConfig),
    QBittorrent(QBittorrentClientConfig),
    Transmission(TransmissionClientConfig),
    Exec(ExecClientConfig),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DelugeClientConfig {
    pub url: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QBittorrentClientConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransmissionClientConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecClientConfig {
    pub command: String,
}

pub fn get_config_dir() -> Option<PathBuf> {
    ProjectDirs::from("cc", "nezu", "windscribe-ephemeral-port")
        .map(|dirs| dirs.config_dir().to_owned())
}

fn get_config_paths(filename: &str) -> Vec<PathBuf> {
    let cfg_dir = get_config_dir();
    let exe_dir = current_exe()
        .ok()
        .and_then(|path| path.parent().map(|path| path.to_owned()));
    let current_dir = current_dir().ok();
    vec![cfg_dir, exe_dir, current_dir]
        .into_iter()
        .filter_map(|path| path)
        .map(|path| path.join(filename))
        .collect()
}

pub async fn load_config(config_path: Option<PathBuf>) -> Result<Config> {
    let config_paths = match config_path {
        Some(path) => vec![path],
        None => get_config_paths("config.yaml"),
    };

    let config_path = config_paths
        .iter()
        .find(|path| path.exists())
        .ok_or_else(|| anyhow!("No config file found, tried: {:?}", config_paths))?;

    let config = fs::read_to_string(config_path).await?;
    let config: Config =
        serde_path_to_error::deserialize(serde_yaml::Deserializer::from_str(&config))?;

    Ok(config)
}

pub async fn get_cache(cache_path: Option<PathBuf>, name: &str) -> Result<SimpleCache> {
    let cache_dir = match cache_path {
        Some(path) => vec![path],
        None => get_config_paths(format!("{}.json", name).as_str()),
    };

    let existing_cache_path = cache_dir.iter().find(|path| path.exists());
    let cache_path = match existing_cache_path {
        Some(path) => Some(path),
        None => cache_dir.first(),
    };

    // create cache directory if it doesn't exist
    if let Some(directory) = cache_path.and_then(|path| path.parent()) {
        fs::create_dir_all(directory).await?;
    };

    match cache_path {
        Some(path) => SimpleCache::load(path.to_owned()).await,
        None => {
            warn!("No cache path found, using in-memory cache");
            Ok(SimpleCache::new())
        }
    }
}
