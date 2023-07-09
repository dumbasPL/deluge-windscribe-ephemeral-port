use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, ErrorKind},
    path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    pub value: String,
    pub expires: Option<DateTime<Utc>>,
}

pub struct SimpleCache {
    cache: RefCell<HashMap<String, CacheEntry>>,
    file: Option<PathBuf>,
}

impl SimpleCache {
    pub fn new(file: Option<PathBuf>) -> Result<Self> {
        let cache = match &file {
            Some(file) => {
                let file = File::open(file);
                match file {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        serde_json::from_reader(reader)?
                    }
                    Err(e) => match e.kind() {
                        ErrorKind::NotFound => HashMap::new(),
                        _ => return Err(e.into()),
                    },
                }
            }
            None => HashMap::new(),
        };

        let new_self = Self {
            cache: RefCell::new(cache),
            file,
        };

        new_self.save()?;

        Ok(new_self)
    }

    fn save(&self) -> Result<()> {
        if let Some(file) = &self.file {
            let file = File::create(file)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer(writer, &self.cache)?;
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.borrow_mut();
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

    pub fn set(&self, key: &str, value: String, expires: Option<DateTime<Utc>>) -> Result<()> {
        self.cache
            .borrow_mut()
            .insert(key.to_string(), CacheEntry { value, expires });
        self.save()
    }
}

impl Default for SimpleCache {
    fn default() -> Self {
        // unwrap() is safe here because it can't fail if there is no file IO
        Self::new(None).unwrap()
    }
}
