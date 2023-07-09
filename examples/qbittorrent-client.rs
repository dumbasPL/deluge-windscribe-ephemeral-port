use anyhow::Result;
use clap::Parser;
use windscribe_ephemeral_port::qbittorrent::QBittorrentClient;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The URL of the qBittorrent web ui
    #[arg(short = 'U', long, default_value = "http://localhost:9092")]
    url: String,

    // The username of the qBittorrent web ui
    #[arg(short, long, default_value = "admin")]
    username: String,

    /// The password of the qBittorrent web ui
    #[arg(short, long, default_value = "adminadmin")]
    password: String,

    /// The port to set qBittorrent to (if not specified, no changes will be made)
    #[arg(short = 'P', long)]
    port: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = QBittorrentClient::new(&cli.url, &cli.username, &cli.password)?;

    println!("Logging in...");
    client.login().await?;

    println!("Getting version...");
    let version = client.get_version().await?;
    println!("qBittorrent version: {}", version);

    println!("Getting port config...");
    let config = client.get_preferences().await?;
    println!("Config: {:?}", config);

    if let Some(port) = cli.port {
        println!("Setting port to: {}...", port);
        client.set_listen_port(port).await?;

        println!("Getting port config...");
        let config = client.get_preferences().await?;
        println!("Config: {:?}", config);
    }

    Ok(())
}
