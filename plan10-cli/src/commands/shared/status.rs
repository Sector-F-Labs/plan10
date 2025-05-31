use anyhow::Result;
use crate::{Config, ExecutionMode};
use crate::commands::utils::*;
use crate::ssh::SshClient;
use crate::utils::system::{get_system_info, is_caffeinate_running, is_on_battery, is_on_ac_power, get_battery_percentage};
use crate::utils::formatting::*;
use colored::*;
use chrono::Utc;

pub async fn execute(
    host: Option<String>,
    detailed: bool,
    config: &Config,
    execution_mode: ExecutionMode,
    verbose: bool,
) -> Result<()> {
    match execution_mode {
        ExecutionMode::Local => {
            execute_local_status(detailed, verbose).await
        }
        ExecutionMode::Remote { host: default_host } => {
            let target_host = host.unwrap_or(default_host);
            execute_remote_status(&target_host, detailed, config, verbose).await
        }
        ExecutionMode::Auto => {
            if let Some(target_host) = host {
                execute_remote_status(&target_host, detailed, config, verbose).await
            } else {
                execute_local_status(detailed, verbose).await
            }
        }
    }
}

async fn execute_local_status(detailed: bool, verbose: bool) -> Result<()> {
    let timestamp = Utc::now();
    
    print_header(&format!("Plan 10 Status - {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Power status
    let on_battery = is_on_battery().unwrap_or(false);
    let on_ac = is_on_ac_power().unwrap_or(false);
    let battery_pct = get_battery_percentage().unwrap_or(None);
    
    println!("{}:", "Power Status".bold());
    println!("  Source: {}", format_power_source(on_battery, on_ac));
    
    if let Some(pct) = battery_pct {
        let (icon, status) = format_percentage_status(pct);
        println!("  Battery: {} {}% ({})", icon, pct, status);
    }
    
    // Service status
    println!("\n{}:", "Services".bold());
    let caffeinate_running = is_caffeinate_running().unwrap_or(false);
    println!("  Caffeinate: {}", format_service_status(caffeinate_running, true));
    
    if detailed {
        // System information
        println!("\n{}:", "System Information".bold());
        if let Ok(sys_info) = get_system_info() {
            println!("  Hostname: {}", sys_info.hostname);
            println!("  Uptime: {}", format_duration(sys_info.uptime));
            println!("  CPU Usage: {}", format_cpu_usage(sys_info.cpu_usage));
            println!("  Memory: {}", format_memory_usage(sys_info.memory_used, sys_info.memory_total));
            println!("  Load: {:.2} {:.2} {:.2}", 
                     sys_info.load_average.0, 
                     sys_info.load_average.1, 
                     sys_info.load_average.2);
            
            if !sys_info.disks.is_empty() {
                println!("\n{}:", "Storage".bold());
                for disk in &sys_info.disks {
                    println!("  {}: {}", 
                             disk.mount_point,
                             format_disk_usage(disk.used_space, disk.total_space));
                }
            }
        }
        
        // Configuration status
        println!("\n{}:", "Configuration".bold());
        println!("  Config file: {}", 
                 Config::default_config_path()
                     .map(|p| p.display().to_string())
                     .unwrap_or_else(|| "Not found".to_string()));
        
        // Load config to show server count and default server
        if let Ok(loaded_config) = Config::load(None) {
            println!("  Servers configured: {}", loaded_config.servers.len());
            
            if let Some(default_server) = &loaded_config.client.default_server {
                println!("  Default server: {}", default_server);
            }
        } else {
            println!("  Servers configured: Unable to load config");
        }
    }
    
    // Health summary
    println!("\n{}:", "Health Summary".bold());
    let mut health_issues = 0;
    
    if !caffeinate_running {
        println!("  {} Caffeinate is not running", "⚠️".yellow());
        health_issues += 1;
    }
    
    if on_battery {
        if let Some(pct) = battery_pct {
            if pct < 20 {
                println!("  {} Battery level critical ({}%)", "🔴".red(), pct);
                health_issues += 1;
            } else if pct < 50 {
                println!("  {} Battery level low ({}%)", "🟡".yellow(), pct);
                health_issues += 1;
            }
        }
    }
    
    if health_issues == 0 {
        println!("  {} All systems operational", "🟢".green());
    } else {
        println!("  {} {} issue(s) detected", "⚠️".yellow(), health_issues);
    }
    
    Ok(())
}

async fn execute_remote_status(
    host: &str,
    detailed: bool,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    let server = config.resolve_server(host)
        .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", host))?;

    print_verbose(&format!("Connecting to {}", host), verbose);
    let client = SshClient::connect(server, config).await?;
    
    let timestamp = Utc::now();
    print_header(&format!("Plan 10 Status - {} - {}", host, timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Test connectivity
    match client.test_connection() {
        Ok(_) => println!("{}:", "Connection".bold()),
        Err(e) => {
            print_error(&format!("Failed to connect to {}: {}", host, e));
            return Ok(());
        }
    }
    
    let (icon, _) = format_percentage_status(100);
    println!("  Status: {} Connected", icon);
    
    // Get remote status using scripts
    println!("\n{}:", "Power Status".bold());
    match client.execute_command("pmset -g batt | head -1") {
        Ok(result) if result.success => {
            let output = result.stdout.trim();
            if output.contains("Battery Power") {
                println!("  Source: {}", "🔋 Battery Power".yellow());
            } else if output.contains("AC Power") {
                println!("  Source: {}", "🔌 AC Power".green());
            } else {
                println!("  Source: {}", "❓ Unknown".dimmed());
            }
            
            // Extract battery percentage
            for line in output.lines() {
                if let Some(start) = line.find(char::is_numeric) {
                    if let Some(end) = line[start..].find('%') {
                        let pct_str = &line[start..start + end];
                        if let Ok(pct) = pct_str.parse::<u8>() {
                            let (icon, status) = format_percentage_status(pct);
                            println!("  Battery: {} {}% ({})", icon, pct, status);
                            break;
                        }
                    }
                }
            }
        }
        _ => println!("  Source: {}", "❓ Unable to determine".dimmed()),
    }
    
    // Service status
    println!("\n{}:", "Services".bold());
    let caffeinate_running = match client.execute_command("pgrep -x caffeinate") {
        Ok(result) => result.success && !result.stdout.trim().is_empty(),
        _ => false,
    };
    println!("  Caffeinate: {}", format_service_status(caffeinate_running, true));
    
    if detailed {
        // System information
        println!("\n{}:", "System Information".bold());
        if let Ok(sys_info) = client.get_system_info() {
            println!("  Hostname: {}", sys_info.hostname);
            println!("  System: {}", sys_info.uname);
            println!("  Uptime: {}", sys_info.uptime);
            println!("  User: {}", sys_info.current_user);
            
            if !sys_info.disk_usage.is_empty() {
                println!("\n{}:", "Storage".bold());
                for line in sys_info.disk_usage.lines().skip(1) {
                    if !line.trim().is_empty() {
                        println!("  {}", line);
                    }
                }
            }
        }
        
        // Check for Plan 10 files
        println!("\n{}:", "Plan 10 Installation".bold());
        let files_to_check = vec![
            ("server_setup.sh", "~/server_setup.sh"),
            ("temp script", "~/scripts/temp"),
            ("battery script", "~/scripts/battery"),
            ("power_diagnostics script", "~/scripts/power_diagnostics"),
        ];
        
        for (name, path) in files_to_check {
            match client.file_exists(path) {
                Ok(true) => println!("  {}: {}", name, "✅ Present".green()),
                Ok(false) => println!("  {}: {}", name, "❌ Missing".red()),
                Err(_) => println!("  {}: {}", name, "❓ Unknown".dimmed()),
            }
        }
    }
    
    // Health summary
    println!("\n{}:", "Health Summary".bold());
    let mut health_issues = 0;
    
    if !caffeinate_running {
        println!("  {} Caffeinate is not running", "⚠️".yellow());
        health_issues += 1;
    }
    
    // Check if we can run basic commands
    match client.execute_command("echo 'test'") {
        Ok(result) if result.success => {},
        _ => {
            println!("  {} Command execution issues detected", "🔴".red());
            health_issues += 1;
        }
    }
    
    if health_issues == 0 {
        println!("  {} All systems operational", "🟢".green());
    } else {
        println!("  {} {} issue(s) detected", "⚠️".yellow(), health_issues);
    }
    
    Ok(())
}

pub fn show_help() {
    println!("Usage: plan10 status [options]");
    println!();
    println!("Options:");
    println!("  -d, --detailed    Show detailed status information");
    println!("  -H, --host <HOST> Target server (remote status check)");
    println!("  -v, --verbose     Verbose output");
    println!("  -h, --help        Show this help message");
    println!();
    println!("Examples:");
    println!("  plan10 status                    # Local status check");
    println!("  plan10 status --detailed         # Detailed local status");
    println!("  plan10 status --host myserver    # Remote status check");
}