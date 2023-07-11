use anyhow::{anyhow, Result};
use std::process::Stdio;
use tokio::process::Command;

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

        println!("executing: {} {}", &self.command, shell_words::join(&args));

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
