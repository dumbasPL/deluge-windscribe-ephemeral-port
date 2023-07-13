use std::{path::PathBuf, sync::Arc};

use anyhow::{Ok, Result};
use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};
use windscribe_ephemeral_port::{client::TimedPortClient, config::load_config};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg_path = PathBuf::from("./config/example.yaml");
    let config = load_config(Some(cfg_path)).await?;

    let mut sched = JobScheduler::new().await?;

    let clients = config
        .clients
        .iter()
        .map(|client| Arc::new(TimedPortClient::new(&client)))
        .collect::<Vec<_>>();

    for client in clients {
        if let Some(interval) = client.check_interval() {
            let client = client.clone();
            let job = Job::new_repeated_async(interval, move |_uuid, _l| {
                let client = client.clone();
                Box::pin(async move {
                    if let Err(e) = client.update(None).await {
                        println!("Error updating port for {}: {}", client.name(), e);
                    }
                })
            })?;
            sched.add(job).await?;
        }
    }

    println!("Starting scheduler");
    sched.start().await?;
    signal::ctrl_c().await?;
    sched.shutdown().await?;
    Ok(())
}
