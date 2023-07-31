use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use tracing::info;
use tracing_log::AsTrace;
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

    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(cli.verbose.log_level_filter().as_trace())
        .without_time() // your log driver should do that
        .init();

    let client = QBittorrentClient::new(&cli.url, &cli.username, &cli.password)?;

    info!("Logging in...");
    client.login().await?;

    info!("Getting version...");
    let version = client.get_version().await?;
    info!("qBittorrent version: {}", version);

    info!("Getting port config...");
    let config = client.get_preferences().await?;
    info!("Config: {:?}", config);

    if let Some(port) = cli.port {
        info!("Setting port to: {}...", port);
        client.set_listen_port(port).await?;

        info!("Getting port config...");
        let config = client.get_preferences().await?;
        info!("Config: {:?}", config);
    }

    Ok(())
}
