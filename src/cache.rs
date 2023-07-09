use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::ErrorKind, path::PathBuf, sync::Mutex};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    pub value: String,
    pub expires: Option<DateTime<Utc>>,
}

pub struct SimpleCache {
    cache: Mutex<HashMap<String, CacheEntry>>,
    file: Option<PathBuf>,
}

impl SimpleCache {
    pub async fn load(file: PathBuf) -> Result<Self> {
        let cache = match File::open(&file).await {
            Ok(mut file) => {
                let mut contents = vec![];
                file.read_to_end(&mut contents).await?;
                serde_json::from_slice(&contents)?
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => HashMap::new(),
                _ => return Err(e.into()),
            },
        };

        let new_self = Self {
            cache: Mutex::new(cache),
            file: Some(file),
        };

        new_self.save().await?;

        Ok(new_self)
    }

    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            file: None,
        }
    }

    async fn save(&self) -> Result<()> {
        if let Some(file) = &self.file {
            let mut file = File::create(file).await?;
            let contents = serde_json::to_vec(&self.cache)?;
            file.write_all(&contents).await?;
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.lock().unwrap();
        match cache.get(key) {
            Some(entry) => match entry.expires {
                Some(expires) if expires <= Utc::now() => {
                    cache.remove(key);
                    None
                }
                _ => Some(entry.value.clone()),
            },
            None => None,
        }
    }

    pub async fn set(
        &self,
        key: &str,
        value: String,
        expires: Option<DateTime<Utc>>,
    ) -> Result<()> {
        self.cache
            .lock()
            .unwrap()
            .insert(key.to_string(), CacheEntry { value, expires });
        self.save().await
    }
}

impl Default for SimpleCache {
    fn default() -> Self {
        Self::new()
    }
}
