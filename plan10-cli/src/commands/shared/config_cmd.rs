use anyhow::Result;
use crate::Config;
use crate::commands::utils::*;
use colored::*;
use std::process::Command;

pub async fn execute(
    server: Option<String>,
    edit: bool,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    if edit {
        edit_config(config, verbose).await
    } else if let Some(server_name) = server {
        show_server_config(&server_name, config, verbose).await
    } else {
        show_full_config(config, verbose).await
    }
}

async fn show_full_config(config: &Config, verbose: bool) -> Result<()> {
    print_header("Plan 10 Configuration");
    
    // Configuration file location
    let config_path = Config::default_config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Not found".to_string());
    
    println!("{}:", "Configuration File".bold());
    println!("  Location: {}", config_path);
    
    // Client configuration
    println!("\n{}:", "Client Settings".bold());
    println!("  Default server: {}", 
             config.client.default_server.as_deref().unwrap_or("None"));
    println!("  Deployment timeout: {}s", config.client.deployment_timeout);
    println!("  Concurrent operations: {}", config.client.concurrent_operations);
    println!("  Auto backup: {}", config.client.auto_backup);
    
    // Server configuration
    println!("\n{}:", "Server Settings".bold());
    println!("  Name: {}", config.server.name);
    println!("  Monitoring interval: {}s", config.server.monitoring_interval);
    println!("  Temperature threshold: {:.1}°C", config.server.temp_threshold);
    println!("  Battery warning level: {}%", config.server.battery_warning_level);
    println!("  Auto restart services: {}", config.server.auto_restart_services);
    println!("  Log level: {}", config.server.log_level);
    
    // SSH configuration
    println!("\n{}:", "SSH Settings".bold());
    println!("  Connect timeout: {}s", config.ssh.connect_timeout);
    println!("  Command timeout: {}s", config.ssh.command_timeout);
    println!("  Key path: {}", config.ssh.key_path.as_deref().unwrap_or("Default"));
    println!("  Known hosts: {}", config.ssh.known_hosts_file.as_deref().unwrap_or("Default"));
    println!("  Compression: {}", config.ssh.compression);
    println!("  Keep alive: {}", config.ssh.keep_alive);
    
    // Servers
    println!("\n{}:", "Configured Servers".bold());
    if config.servers.is_empty() {
        println!("  No servers configured");
    } else {
        let mut servers: Vec<_> = config.servers.iter().collect();
        servers.sort_by_key(|(name, _)| *name);
        
        for (name, server) in servers {
            let status_icon = if server.enabled { "🟢" } else { "🔴" };
            let last_seen = server.last_seen
                .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
                .unwrap_or_else(|| "Never".to_string());
            
            println!("  {} {} ({}@{}:{})", status_icon, name, server.user, server.host, server.port);
            if verbose {
                println!("    Tags: {}", server.tags.join(", "));
                println!("    Last seen: {}", last_seen);
                if let Some(key) = &server.ssh_key {
                    println!("    SSH key: {}", key);
                }
            }
        }
    }
    
    // Services
    if verbose {
        println!("\n{}:", "Services".bold());
        for service in &config.server.services {
            println!("  • {}", service);
        }
    }
    
    Ok(())
}

async fn show_server_config(server_name: &str, config: &Config, verbose: bool) -> Result<()> {
    let server = config.get_server(server_name)
        .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", server_name))?;
    
    print_header(&format!("Server Configuration - {}", server_name));
    
    println!("{}:", "Connection Details".bold());
    println!("  Name: {}", server.name);
    println!("  Host: {}", server.host);
    println!("  User: {}", server.user);
    println!("  Port: {}", server.port);
    println!("  Enabled: {}", if server.enabled { "Yes" } else { "No" });
    
    if let Some(key) = &server.ssh_key {
        println!("  SSH Key: {}", key);
    }
    
    if !server.tags.is_empty() {
        println!("  Tags: {}", server.tags.join(", "));
    }
    
    if let Some(last_seen) = server.last_seen {
        println!("  Last seen: {}", last_seen.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    if verbose {
        // Test connectivity
        println!("\n{}:", "Connectivity Test".bold());
        print_info("Testing connection...");
        
        match crate::ssh::test_connectivity(server, config).await {
            Ok(true) => print_success("Connection successful"),
            Ok(false) => print_error("Connection failed"),
            Err(e) => print_error(&format!("Connection test error: {}", e)),
        }
    }
    
    Ok(())
}

async fn edit_config(config: &Config, verbose: bool) -> Result<()> {
    let config_path = Config::default_config_path()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config file path"))?;
    
    if !config_path.exists() {
        print_warning("Configuration file does not exist, creating it...");
        config.save(Some(&config_path))?;
        print_success(&format!("Created config file: {}", config_path.display()));
    }
    
    // Try to find an editor
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "macos") {
                "nano".to_string()
            } else {
                "vi".to_string()
            }
        });
    
    print_info(&format!("Opening config file with {}", editor));
    print_verbose(&format!("Config file: {}", config_path.display()), verbose);
    
    let status = Command::new(&editor)
        .arg(&config_path)
        .status()?;
    
    if status.success() {
        print_success("Configuration file updated");
        
        // Validate the updated configuration
        match Config::load(Some(&config_path.to_string_lossy())) {
            Ok(new_config) => {
                if let Err(e) = new_config.validate() {
                    print_warning(&format!("Configuration validation failed: {}", e));
                    print_info("Please fix the configuration file and try again");
                } else {
                    print_success("Configuration is valid");
                }
            }
            Err(e) => {
                print_error(&format!("Failed to parse configuration: {}", e));
                print_info("Please check the configuration file syntax");
            }
        }
    } else {
        print_error("Editor exited with error");
    }
    
    Ok(())
}

pub fn show_help() {
    println!("Usage: plan10 config [options]");
    println!();
    println!("Options:");
    println!("  -s, --server <NAME>  Show configuration for specific server");
    println!("  -e, --edit           Edit configuration file");
    println!("  -v, --verbose        Show detailed information");
    println!("  -h, --help           Show this help message");
    println!();
    println!("Examples:");
    println!("  plan10 config                    # Show full configuration");
    println!("  plan10 config --server myserver  # Show specific server config");
    println!("  plan10 config --edit             # Edit configuration file");
    println!("  plan10 config --verbose          # Show detailed configuration");
}