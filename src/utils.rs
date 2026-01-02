use anyhow::Result;
use std::process::Command;

#[allow(dead_code)]
pub fn run_cmd(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(command).args(args).output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Ok(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[allow(dead_code)]
pub fn run_cmd_exit_only(command: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(command).args(args).status()?;

    if output.success() {
        Ok(())
    } else {
        anyhow::bail!("Command failed with exit code: {:?}", output.code())
    }
}
