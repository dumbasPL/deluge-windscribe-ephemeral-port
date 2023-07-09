use anyhow::{anyhow, Result};
use clap::Parser;
use windscribe_ephemeral_port::transmission::{TransmissionClient, TransmissionCredentials};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The base URL of the transmission web ui web ui
    #[arg(short = 'U', long, default_value = "http://localhost:9091/")]
    url: String,

    // The basic auth username of the transmission web ui
    #[arg(short, long)]
    username: Option<String>,

    /// The basic auth password of the transmission web ui
    #[arg(short, long)]
    password: Option<String>,

    /// The port to set qBittorrent to (if not specified, no changes will be made)
    #[arg(short = 'P', long)]
    port: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let credentials = match (cli.username, cli.password) {
        (Some(username), Some(password)) => {
            Ok(Some(TransmissionCredentials { username, password }))
        }
        (Some(_), None) | (None, Some(_)) => Err(anyhow!(
            "Must specify both username and password or neither"
        )),
        _ => Ok(None),
    }?;

    let client = TransmissionClient::new(&cli.url, credentials)?;

    println!("Getting version...");
    let version = client.get_version().await?;
    println!("qBittorrent version: {}", version);

    println!("Getting port config...");
    let config = client.get_session_arguments().await?;
    println!("Config: {:?}", config);

    if let Some(port) = cli.port {
        println!("Setting port to: {}...", port);
        client.set_session_arguments(false, port).await?;

        println!("Getting port config...");
        let config = client.get_session_arguments().await?;
        println!("Config: {:?}", config);
    }

    Ok(())
}
