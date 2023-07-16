use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::Parser;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{pin, select, signal, sync::mpsc};
use tokio_cron_scheduler::{Job, JobScheduler};
use windscribe_ephemeral_port::{
    client::TimedPortClient,
    config::{self, WindscribeConfig},
    windscribe::{WindscribeClient, WindscribeEpfStatus},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// override config path
    #[arg(short = 'c', long)]
    config: Option<PathBuf>,

    /// override cache directory
    #[arg(long)]
    cache_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = config::load_config(cli.config).await?;

    if config.clients.len() == 0 {
        return Err(anyhow!("No clients configured"));
    }

    let mut sched = JobScheduler::new().await?;

    let port_clients = Arc::new(
        config
            .clients
            .iter()
            .map(|client| Arc::new(TimedPortClient::new(&client)))
            .collect::<Vec<_>>(),
    );

    for port_client in port_clients.iter() {
        if let Some(interval) = port_client.check_interval() {
            let client = port_client.clone();
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

    let WindscribeConfig {
        username,
        password,
        check_interval,
        extra_delay,
        retry_delay,
    } = config.windscribe;
    let check_interval = check_interval.map(Duration::from_secs);
    let retry_delay = Duration::from_secs(retry_delay);

    let windscribe_cache = config::get_cache(cli.cache_dir, "windscribe").await?;
    let windscribe_client = Arc::new(WindscribeClient::new(
        &username,
        &password,
        windscribe_cache,
    )?);

    let (tx, mut rx) = mpsc::channel::<Duration>(32);

    let run_tx = tx.clone();
    let run = move || {
        let windscribe_client = windscribe_client.clone();
        let port_clients = port_clients.clone();
        let tx = run_tx.clone();
        Box::pin(async move {
            let res = update_port(windscribe_client, &port_clients, extra_delay).await;
            match res {
                Ok(expires_in) => match check_interval {
                    None => {
                        println!("Scheduling next check in {} seconds", expires_in.as_secs());
                        tx.send(expires_in)
                            .await
                            .expect("Failed to queue check interval");
                    }
                    Some(interval) if expires_in < interval => {
                        println!("Port expires in less than check interval, scheduling check in {} seconds", expires_in.as_secs());
                        tx.send(expires_in)
                            .await
                            .expect("Failed to queue next check");
                    }
                    Some(interval) => {
                        println!("Scheduling next check in {} seconds", interval.as_secs());
                        tx.send(interval)
                            .await
                            .expect("Failed to queue check interval");
                    }
                },
                Err(e) => {
                    println!("Error updating port: {}", e);
                    println!("Scheduling next check in {} seconds", retry_delay.as_secs());
                    tx.send(retry_delay)
                        .await
                        .expect("Failed to queue check interval");
                }
            }
        })
    };

    sched.start().await?;

    let int_signal = signal::ctrl_c();
    pin!(int_signal);

    tx.send(Duration::ZERO)
        .await
        .expect("Failed to queue initial check");

    loop {
        select! {
            _ = &mut int_signal => {
                println!("Received interrupt signal, shutting down...");
                break;
            },
            Some(delay) = rx.recv() => {
                let run = run.clone();
                let job = Job::new_one_shot_async(delay, move |_uuid, _l| run())?;
                sched.add(job).await?;
            }
        }
    }

    sched.shutdown().await?;
    Ok(())
}

async fn update_port(
    windscribe_client: Arc<WindscribeClient>,
    port_clients: &Vec<Arc<TimedPortClient>>,
    extra_delay: i64,
) -> Result<Duration> {
    let epf_info = windscribe_client.get_epf_info().await?;

    let existing_port = match epf_info {
        WindscribeEpfStatus::Enabled(ref info) if info.internal_port == info.external_port => {
            let expires_in = info.expires + chrono::Duration::seconds(extra_delay) - Utc::now();
            if expires_in.num_seconds() < 0 {
                Err("Ephemeral port expired")
            } else {
                Ok((info.internal_port, expires_in))
            }
        }
        WindscribeEpfStatus::Enabled(_) => Err("Internal and external ports are different"),
        WindscribeEpfStatus::Disabled => Err("No ephemeral port found"),
    };

    let (new_port, expires_in) = match existing_port {
        Ok(val) => {
            println!("Using existing ephemeral port: {}", val.0);
            Ok(val)
        }
        Err(reason) => {
            println!("Creating new ephemeral port: {}", reason);
            let csrf_token = windscribe_client.get_my_account_csrf_token().await?;

            if matches!(epf_info, WindscribeEpfStatus::Enabled(_)) {
                println!("Deleting existing ephemeral port...");
                let deleted = windscribe_client.remove_epf(&csrf_token).await?;
                match deleted {
                    true => println!("Ephemeral port forwarding deleted"),
                    false => println!("Failed to delete ephemeral port, continuing..."),
                }
            };

            // request matching port
            let new_port = windscribe_client.request_epf(&csrf_token, None).await?;
            assert_eq!(new_port.internal_port, new_port.external_port);

            let expires_in = new_port.expires + chrono::Duration::seconds(extra_delay) - Utc::now();

            println!(
                "New ephemeral port: {}, expires in {} seconds",
                new_port.internal_port,
                expires_in.num_seconds()
            );
            anyhow::Ok((new_port.internal_port, expires_in))
        }
    }?;

    // update all clients
    for port_client in port_clients {
        match port_client.update(Some(new_port)).await {
            Ok(false) => {} // no change
            Ok(true) => println!("Updated port for {} to {}", port_client.name(), new_port),
            Err(e) => println!("Error updating port for {}: {}", port_client.name(), e),
        }
    }

    Ok(expires_in.to_std()?)
}
