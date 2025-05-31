pub mod system;
pub mod formatting;

use anyhow::Result;
use std::process::Command;

pub fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr)
    }
}

pub fn run_command_with_status(cmd: &str, args: &[&str]) -> Result<(String, String, bool)> {
    let output = Command::new(cmd)
        .args(args)
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();
    
    Ok((stdout, stderr, success))
}

pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

pub fn require_macos() -> Result<()> {
    if !is_macos() {
        anyhow::bail!("This command requires macOS");
    }
    Ok(())
}

pub fn check_sudo() -> Result<()> {
    let output = Command::new("id")
        .arg("-u")
        .output()?;
    
    if output.status.success() {
        let uid_string = String::from_utf8_lossy(&output.stdout);
        let uid = uid_string.trim();
        if uid != "0" {
            anyhow::bail!("This command requires sudo privileges");
        }
    }
    
    Ok(())
}