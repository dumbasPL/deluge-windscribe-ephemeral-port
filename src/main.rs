use anyhow::Result;

use crate::qbittorrent::QBittorrentClient;

pub mod deluge;
pub mod qbittorrent;
pub mod transmission;

#[tokio::main]
async fn main() -> Result<()> {
    let client = QBittorrentClient::new("http://localhost:9092", "admin", "adminadmin")?;

    client.login().await?;

    println!("{:?}", client.get_version().await?);
    println!("{:?}", client.get_preferences().await?);

    client.set_listen_port(2222).await?;

    println!("{:?}", client.get_preferences().await?);

    Ok(())
}
