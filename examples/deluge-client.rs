use anyhow::{anyhow, Result};
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use tracing::info;
use tracing_log::AsTrace;
use windscribe_ephemeral_port::deluge::DelugeClient;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The URL of the Deluge web ui
    #[arg(short = 'U', long, default_value = "http://localhost:8112")]
    url: String,

    /// The password of the Deluge web ui
    #[arg(short, long, default_value = "deluge")]
    password: String,

    /// The deluge host id to connect to (if not specified, the first host will be used)
    #[arg(short = 'H', long)]
    host_id: Option<String>,

    /// The port to set Deluge to (if not specified, no changes will be made)
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

    let client = DelugeClient::new(&cli.url, &cli.password)?;

    info!("Logging in...");
    client.login().await?;

    info!("Getting hosts...");
    let hosts = client.get_hosts().await?;
    info!(
        "Hosts: {}",
        hosts
            .iter()
            .map(|host| format!("\n\t{:?}", host))
            .collect::<String>()
    );

    if hosts.len() == 0 {
        return Err(anyhow!("No hosts found"));
    }

    let host_id = match cli.host_id {
        Some(ref host_id) if hosts.iter().any(|h| h.id.eq(host_id)) => Ok(host_id),
        Some(_) => Err(anyhow!("Invalid host id")),
        None => Ok(&hosts[0].id),
    }?;

    info!("Connecting to host: {}...", host_id);
    client.connect(Some(host_id)).await?;

    info!("Getting version...");
    let version = client.get_version().await?;
    info!("Deluge version: {}", version);

    info!("Getting port config...");
    let config = client.get_port_config().await?;
    info!("Config: {:?}", config);

    if let Some(port) = cli.port {
        info!("Setting port to: {}...", port);
        client.set_port_config(false, port).await?;

        info!("Getting port config...");
        let config = client.get_port_config().await?;
        info!("Config: {:?}", config);
    }

    Ok(())
}
