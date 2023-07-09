use anyhow::Result;
use std::path::PathBuf;
use windscribe_ephemeral_port::{cache::SimpleCache, windscribe::WindscribeClient};

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
