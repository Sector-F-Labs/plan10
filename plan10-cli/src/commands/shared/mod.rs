pub mod temp;
pub mod battery;
pub mod power_diagnostics;
pub mod monitor;
pub mod status;
pub mod setup;
pub mod config_cmd;

use anyhow::Result;
use crate::{Config, ExecutionMode, MonitorCommands, WatchType};
use crate::commands::utils::*;
use colored::*;

pub async fn execute_monitor_command(
    cmd: MonitorCommands,
    config: &Config,
    execution_mode: ExecutionMode,
    verbose: bool,
) -> Result<()> {
    match cmd {
        MonitorCommands::Temp { raw, host } => {
            temp::execute_temp_command(raw, host, config, execution_mode, verbose).await
        }
        MonitorCommands::Battery { detailed, raw, host } => {
            battery::execute_battery_command(detailed, raw, host, config, execution_mode, verbose).await
        }
        MonitorCommands::Power { verbose: power_verbose, battery, sleep, all, fixes, host } => {
            power_diagnostics::execute_power_diagnostics_command(
                power_verbose, battery, sleep, all, fixes, host, config, execution_mode, verbose
            ).await
        }
        MonitorCommands::System { host } => {
            execute_system_monitor(host, config, execution_mode, verbose).await
        }
        MonitorCommands::Watch { interval, monitor, host } => {
            execute_watch_monitor(interval, monitor, host, config, execution_mode, verbose).await
        }
    }
}

async fn execute_system_monitor(
    host: Option<String>,
    config: &Config,
    execution_mode: ExecutionMode,
    verbose: bool,
) -> Result<()> {
    print_header("System Overview");
    
    match execution_mode {
        ExecutionMode::Local => {
            execute_local_system_monitor(verbose).await
        }
        ExecutionMode::Remote { host: default_host } => {
            let target_host = host.unwrap_or(default_host);
            execute_remote_system_monitor(&target_host, config, verbose).await
        }
        ExecutionMode::Auto => {
            if let Some(target_host) = host {
                execute_remote_system_monitor(&target_host, config, verbose).await
            } else {
                execute_local_system_monitor(verbose).await
            }
        }
    }
}

async fn execute_local_system_monitor(verbose: bool) -> Result<()> {
    use sysinfo::{System, SystemExt, CpuExt, DiskExt};
    
    let mut system = System::new_all();
    system.refresh_all();
    
    // System info
    println!("{}:", "System Information".bold());
    println!("  Hostname: {}", hostname::get().unwrap_or_default().to_string_lossy());
    println!("  Uptime: {} seconds", system.uptime());
    
    // CPU info
    println!("\n{}:", "CPU".bold());
    println!("  Usage: {:.1}%", system.global_cpu_info().cpu_usage());
    println!("  Load Average: {:?}", system.load_average());
    
    // Memory info
    println!("\n{}:", "Memory".bold());
    println!("  Total: {} GB", system.total_memory() / 1_000_000);
    println!("  Used: {} GB", system.used_memory() / 1_000_000);
    println!("  Available: {} GB", system.available_memory() / 1_000_000);
    
    // Disk info
    println!("\n{}:", "Storage".bold());
    for disk in system.disks() {
        let total_gb = disk.total_space() / 1_000_000_000;
        let available_gb = disk.available_space() / 1_000_000_000;
        let used_gb = total_gb - available_gb;
        let usage_pct = if total_gb > 0 { (used_gb * 100) / total_gb } else { 0 };
        
        println!("  {}: {}/{} GB ({}% used)", 
                 disk.mount_point().display(),
                 used_gb, total_gb, usage_pct);
    }
    
    Ok(())
}

async fn execute_remote_system_monitor(
    host: &str,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    let server = config.resolve_server(host)
        .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", host))?;

    let client = crate::ssh::SshClient::connect(server, config).await?;
    
    // Get system information
    let system_info = client.get_system_info()?;
    
    println!("{}:", "System Information".bold());
    println!("  Hostname: {}", system_info.hostname);
    println!("  System: {}", system_info.uname);
    println!("  Uptime: {}", system_info.uptime);
    println!("  User: {}", system_info.current_user);
    
    println!("\n{}:", "Storage".bold());
    println!("{}", system_info.disk_usage);
    
    Ok(())
}

async fn execute_watch_monitor(
    interval: u64,
    monitor_type: WatchType,
    host: Option<String>,
    config: &Config,
    execution_mode: ExecutionMode,
    verbose: bool,
) -> Result<()> {
    use tokio::time::{sleep, Duration};
    use std::io::{self, Write};
    
    print_info(&format!("Starting continuous monitoring ({}s interval)", interval));
    print_info("Press Ctrl+C to stop");
    
    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
        
        // Show timestamp
        let now = chrono::Utc::now();
        println!("{} Monitor Update - {}", "🕐".cyan(), now.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("{}", "=".repeat(50));
        
        match monitor_type {
            WatchType::All => {
                // Show all monitoring data
                temp::execute_temp_command(false, host.clone(), config, execution_mode.clone(), false).await?;
                println!();
                battery::execute_battery_command(false, false, host.clone(), config, execution_mode.clone(), false).await?;
                println!();
                execute_system_monitor(host.clone(), config, execution_mode.clone(), false).await?;
            }
            WatchType::Temp => {
                temp::execute_temp_command(false, host.clone(), config, execution_mode.clone(), false).await?;
            }
            WatchType::Battery => {
                battery::execute_battery_command(false, false, host.clone(), config, execution_mode.clone(), false).await?;
            }
            WatchType::Power => {
                power_diagnostics::execute_power_diagnostics_command(
                    false, false, false, false, false, host.clone(), config, execution_mode.clone(), false
                ).await?;
            }
            WatchType::System => {
                execute_system_monitor(host.clone(), config, execution_mode.clone(), false).await?;
            }
        }
        
        println!("\n{} Next update in {}s...", "⏰".dimmed(), interval);
        sleep(Duration::from_secs(interval)).await;
    }
}