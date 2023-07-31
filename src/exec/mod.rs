use crate::client::PortClient;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;
use tracing::info;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

pub struct ExecClient {
    command: String,
    args: Vec<String>,
}

impl ExecClient {
    pub fn new(command: &str) -> Result<Self> {
        let args = shell_words::split(command)?;
        match &args[..] {
            [command, args @ ..] => {
                if !args.iter().any(|arg| arg.contains("{}")) {
                    return Err(anyhow!(
                        "Exec command arguments must contain the placeholder {{}} for the port"
                    ));
                }
                Ok(Self {
                    command: command.to_string(),
                    args: args.to_owned(),
                })
            }
            _ => Err(anyhow!("Invalid exec command")),
        }
    }

    pub async fn exec(&self, port: u64) -> Result<()> {
        let args: Vec<String> = self
            .args
            .iter()
            .map(|arg| arg.replace("{}", &port.to_string()))
            .collect();

        info!("executing: {} {}", &self.command, shell_words::join(&args));

        let res = Command::new(&self.command)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await?;

        if let Some(code) = res.code() {
            match code {
                0 => Ok(()),
                code => Err(anyhow!("Command exited with code {}", code)),
            }
        } else {
            #[cfg(unix)]
            match res.signal() {
                Some(signal) => Err(anyhow!("Command exited with signal {}", signal)),
                None => Err(anyhow!("Command exited with unknown status")),
            }
            #[cfg(not(unix))]
            Err(anyhow!("Command exited with unknown status"))
        }
    }
}

#[async_trait]
impl PortClient for ExecClient {
    async fn get_port(&self) -> Result<Option<u64>> {
        Ok(None)
    }

    async fn set_port(&self, port: u64) -> Result<()> {
        self.exec(port).await
    }
}
