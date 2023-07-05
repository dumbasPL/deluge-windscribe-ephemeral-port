use anyhow::Result;
use deluge::DelugeClient;

mod deluge;

#[tokio::main]
async fn main() -> Result<()> {
    let client = DelugeClient::new("http://localhost:8112", "deluge")?;

    client.login().await?;
    client.connect(None).await?;

    println!("{:?}", client.get_version().await?);

    println!("{:?}", client.get_port_config().await?);

    client.set_port_config(false, [6969, 6969]).await?;

    println!("{:?}", client.get_port_config().await?);

    Ok(())
}
