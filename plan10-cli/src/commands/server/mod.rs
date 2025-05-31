use anyhow::Result;
use crate::{ServerCommands, PowerActions, MaintenanceActions, Config};
use crate::commands::utils::*;
use colored::*;
use std::process::Command;

pub mod configure;
pub mod services;
pub mod power;
pub mod maintenance;

pub async fn execute(cmd: ServerCommands, config: &Config, verbose: bool) -> Result<()> {
    // Ensure we're on macOS for server operations
    if !cfg!(target_os = "macos") {
        print_warning("Server commands are designed for macOS systems");
        if !crate::utils::require_macos().is_ok() {
            return Ok(());
        }
    }

    match cmd {
        ServerCommands::Configure { yes, power, monitoring, services } => {
            configure::execute_configure(yes, power, monitoring, services, config, verbose).await
        }
        ServerCommands::Start { service } => {
            services::start_services(service, config, verbose).await
        }
        ServerCommands::Stop { service } => {
            services::stop_services(service, config, verbose).await
        }
        ServerCommands::Restart { service } => {
            services::restart_services(service, config, verbose).await
        }
        ServerCommands::Services { detailed } => {
            services::show_services(detailed, config, verbose).await
        }
        ServerCommands::Power { action } => {
            power::execute_power_action(action, config, verbose).await
        }
        ServerCommands::Maintenance { action } => {
            maintenance::execute_maintenance_action(action, config, verbose).await
        }
    }
}

async fn check_server_requirements() -> Result<()> {
    print_verbose("Checking server requirements", true);
    
    // Check if we're on macOS
    if !cfg!(target_os = "macos") {
        anyhow::bail!("Server mode requires macOS");
    }
    
    // Check if we have required tools
    let required_tools = vec!["pmset", "launchctl", "pgrep", "pkill"];
    for tool in required_tools {
        if !tool_exists(tool) {
            anyhow::bail!("Required tool not found: {}", tool);
        }
    }
    
    Ok(())
}

fn tool_exists(tool: &str) -> bool {
    Command::new("which")
        .arg(tool)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub async fn ensure_directories() -> Result<()> {
    use std::fs;
    
    let dirs_to_create = vec![
        "~/scripts",
        "~/Library/LaunchAgents",
        "~/logs",
    ];
    
    for dir in dirs_to_create {
        let expanded_dir = shellexpand::tilde(dir);
        let path = std::path::Path::new(&*expanded_dir);
        
        if !path.exists() {
            fs::create_dir_all(path)?;
            print_verbose(&format!("Created directory: {}", path.display()), true);
        }
    }
    
    Ok(())
}

pub fn is_service_running(service_name: &str) -> Result<bool> {
    let output = Command::new("pgrep")
        .args(&["-x", service_name])
        .output()?;
    
    Ok(output.status.success() && !output.stdout.is_empty())
}

pub fn get_service_pid(service_name: &str) -> Result<Option<u32>> {
    let output = Command::new("pgrep")
        .arg(service_name)
        .output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().next() {
            if let Ok(pid) = line.trim().parse::<u32>() {
                return Ok(Some(pid));
            }
        }
    }
    
    Ok(None)
}

pub fn is_launchagent_loaded(label: &str) -> Result<bool> {
    let output = Command::new("launchctl")
        .args(&["list", label])
        .output()?;
    
    Ok(output.status.success())
}

pub fn load_launchagent(plist_path: &str) -> Result<()> {
    let output = Command::new("launchctl")
        .args(&["load", plist_path])
        .output()?;
    
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to load LaunchAgent: {}", stderr)
    }
}

pub fn unload_launchagent(label: &str) -> Result<()> {
    let output = Command::new("launchctl")
        .args(&["unload", "-w", &format!("~/Library/LaunchAgents/{}.plist", label)])
        .output()?;
    
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to unload LaunchAgent: {}", stderr)
    }
}