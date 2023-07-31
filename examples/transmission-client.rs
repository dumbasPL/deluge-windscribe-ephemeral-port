use anyhow::{anyhow, Result};
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use tracing::info;
use tracing_log::AsTrace;
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

    info!("Getting version...");
    let version = client.get_version().await?;
    info!("qBittorrent version: {}", version);

    info!("Getting port config...");
    let config = client.get_session_arguments().await?;
    info!("Config: {:?}", config);

    if let Some(port) = cli.port {
        info!("Setting port to: {}...", port);
        client.set_session_arguments(false, port).await?;

        info!("Getting port config...");
        let config = client.get_session_arguments().await?;
        info!("Config: {:?}", config);
    }

    Ok(())
}
