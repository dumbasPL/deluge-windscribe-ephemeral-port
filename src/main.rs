use std::path::PathBuf;

use anyhow::Result;
use windscribe::WindscribeClient;

use crate::cache::SimpleCache;

pub mod constants {
    pub const WINDSCRIBE_USER_AGENT: &str = 
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";
}

pub mod cache;
pub mod deluge;
pub mod qbittorrent;
pub mod transmission;
pub mod windscribe;

#[tokio::main]
async fn main() -> Result<()> {
    let cache = SimpleCache::new(Some(PathBuf::from("test.json")))?;
    let client = WindscribeClient::new("", "", Some(cache))?;

    println!("{:?}", client.get_epf_info().await?);

    let csrf_token = client.get_my_account_csrf_token().await?;

    println!("{:?}", csrf_token);

    println!("{:?}", client.remove_epf(&csrf_token).await?);

    println!("{:?}", client.request_matching_epf(&csrf_token).await?);

    println!("{:?}", client.get_epf_info().await?);

    Ok(())
}
