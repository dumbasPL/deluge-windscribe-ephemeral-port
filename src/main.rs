use anyhow::Result;
use transmission::TransmissionClient;

pub mod deluge;
pub mod transmission;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = TransmissionClient::new("http://localhost:9091", None)?;

    println!("{:?}", client.get_session_arguments().await?);

    client.set_session_arguments(false, 7272).await?;

    println!("{:?}", client.get_session_arguments().await?);

    Ok(())
}
