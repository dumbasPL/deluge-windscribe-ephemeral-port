use anyhow::{anyhow, Result};
use clap::Parser;
use directories::ProjectDirs;
use tokio::fs;
use windscribe_ephemeral_port::{
    cache::SimpleCache,
    windscribe::{WindscribeClient, WindscribeEpfStatus},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // windscribe.com username
    #[arg(short, long)]
    username: String,

    /// windscribe.com password
    #[arg(short, long)]
    password: String,

    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(Parser, Debug)]
enum Subcommand {
    /// Get the current port forwarding info
    GetPort,

    /// Delete the current port forwarding
    DeletePort,

    /// Request a new port
    RequestPort {
        /// request a specific port (if not specified, a matching port will be requested)
        #[arg(short = 'P', long)]
        port: Option<u64>,

        /// Delete the current port forwarding if present
        #[arg(short, long)]
        delete: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let dirs = ProjectDirs::from("cc", "nezu", "windscribe-client")
        .map(|dirs| dirs.data_local_dir().to_owned());

    let cache = match dirs {
        Some(dirs) => {
            let cache_file = dirs.join("cache.json");
            println!("Loading cache from: {:?}", cache_file);
            fs::create_dir_all(&dirs).await?;
            SimpleCache::load(cache_file).await
        }
        None => {
            println!("No cache directory found, using in-memory cache");
            Ok(SimpleCache::default())
        }
    }?;

    let client = WindscribeClient::new(&cli.username, &cli.password, cache)?;

    println!("Getting current port info...");
    let info = client.get_epf_info().await?;
    println!("Info: {:?}", info);

    match cli.subcommand {
        Subcommand::GetPort => Ok(()),
        Subcommand::DeletePort => {
            if matches!(info, WindscribeEpfStatus::Disabled) {
                return Err(anyhow!("No port forwarding to delete"));
            }

            println!("Getting CSRF token...");
            let csrf_token = client.get_my_account_csrf_token().await?;
            println!("Deleting current port forwarding...");
            let deleted = client.remove_epf(&csrf_token).await?;
            match deleted {
                true => {
                    println!("Port forwarding deleted");
                    Ok(())
                }
                false => Err(anyhow!("Failed to delete port forwarding")),
            }
        }
        Subcommand::RequestPort { port, delete } => {
            println!("Getting CSRF token...");
            let csrf_token = client.get_my_account_csrf_token().await?;

            if delete && matches!(info, WindscribeEpfStatus::Enabled(_)) {
                println!("Deleting current port forwarding...");
                let deleted = client.remove_epf(&csrf_token).await?;
                match deleted {
                    true => println!("Port forwarding deleted"),
                    false => println!("Failed to delete port forwarding, continuing..."),
                }
            }

            println!("Requesting port forwarding...");
            let info = client.request_epf(&csrf_token, port).await?;
            println!("Info: {:?}", info);

            Ok(())
        }
    }
}
