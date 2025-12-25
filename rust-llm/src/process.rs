use std::process::Stdio;

use anyhow::{Result, anyhow};
use tokio::{
    io::{AsyncReadExt, BufReader},
    process::Command,
};
use tracing::*;

pub async fn exec(command: &str, args: &[&str]) -> Result<String> {
    let desc = format!("command: {}, args: {:?}", command, args);
    trace!("exec, {}", desc);
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().ok_or(anyhow!("no stdout available for command {command}"))?;
    let stderr = child.stderr.take().ok_or(anyhow!("no stderr available for command {command}"))?;
    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);
    let exit_status = tokio::spawn(async move { child.wait().await });
    let mut stdout = String::new();
    stdout_reader.read_to_string(&mut stdout).await?;
    let mut stderr = String::new();
    stderr_reader.read_to_string(&mut stderr).await?;
    let exit_status = exit_status.await??;
    trace!("{} exit status: {:?}", desc, exit_status);
    if !exit_status.success() {
        error!(
            "process {} exited with non-0 exit code {:?}\nstderr:\n{}",
            command,
            exit_status.code(),
            stderr
        );
        Err(anyhow!("process exited with non-0 exit code: {:?}", exit_status.code(),))?;
    }
    Ok(stdout)
}
