use anyhow::Result;
use crate::Config;
use crate::commands::utils::*;
use crate::config::ServerDefinition;
use crate::ssh::test_connectivity;
use colored::*;
use chrono::Utc;

pub async fn list_servers(config: &Config, detailed: bool, verbose: bool) -> Result<()> {
    print_header("Configured Servers");
    
    if config.servers.is_empty() {
        print_info("No servers configured");
        println!("Use 'plan10 client add <name> --host <host> --user <user>' to add a server");
        return Ok(());
    }

    let mut servers: Vec<_> = config.servers.iter().collect();
    servers.sort_by_key(|(name, _)| *name);

    if detailed {
        for (name, server) in servers {
            print_server_detailed(name, server, verbose).await;
            println!();
        }
    } else {
        print_servers_table(&servers);
    }

    Ok(())
}

pub async fn add_server(
    name: String,
    host: String,
    user: String,
    port: u16,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    print_header(&format!("Adding Server: {}", name));

    // Check if server already exists
    if config.servers.contains_key(&name) {
        print_error(&format!("Server '{}' already exists", name));
        return Ok(());
    }

    let server = ServerDefinition {
        name: name.clone(),
        host: host.clone(),
        user: user.clone(),
        port,
        ssh_key: None,
        tags: vec!["manual".to_string()],
        enabled: true,
        last_seen: None,
    };

    // Test connectivity if verbose
    if verbose {
        print_info("Testing connectivity...");
        match test_connectivity(&server, config).await {
            Ok(true) => print_success("Connection test successful"),
            Ok(false) => print_warning("Connection test failed - server added anyway"),
            Err(e) => print_warning(&format!("Connection test error: {} - server added anyway", e)),
        }
    }

    // Save the updated configuration
    let mut new_config = config.clone();
    new_config.add_server(server)?;
    new_config.save(None)?;

    print_success(&format!("Server '{}' added successfully", name));
    println!("Connection details:");
    println!("  Host: {}", host);
    println!("  User: {}", user);
    println!("  Port: {}", port);
    println!();
    println!("Next steps:");
    println!("  1. Test connection: plan10 client list --detailed");
    println!("  2. Deploy Plan 10: plan10 client deploy --host {}", name);

    Ok(())
}

pub async fn remove_server(name: String, config: &Config, verbose: bool) -> Result<()> {
    print_header(&format!("Removing Server: {}", name));

    if !config.servers.contains_key(&name) {
        print_error(&format!("Server '{}' not found", name));
        return Ok(());
    }

    let server = config.get_server(&name).unwrap();
    
    if verbose {
        println!("Server details:");
        println!("  Host: {}", server.host);
        println!("  User: {}", server.user);
        println!("  Port: {}", server.port);
        println!();
    }

    // Save the updated configuration
    let mut new_config = config.clone();
    new_config.remove_server(&name)?;
    new_config.save(None)?;

    print_success(&format!("Server '{}' removed successfully", name));

    // Warn if this was the default server
    if config.client.default_server.as_ref() == Some(&name) {
        print_warning("This was your default server. You may want to set a new default.");
    }

    Ok(())
}

async fn print_server_detailed(name: &str, server: &ServerDefinition, verbose: bool) {
    let status_icon = if server.enabled { "🟢" } else { "🔴" };
    println!("{} {}", status_icon, name.bold());
    println!("  Host: {}", server.host);
    println!("  User: {}", server.user);
    println!("  Port: {}", server.port);
    println!("  Status: {}", if server.enabled { "Enabled".green() } else { "Disabled".red() });
    
    if !server.tags.is_empty() {
        println!("  Tags: {}", server.tags.join(", ").dimmed());
    }
    
    if let Some(ssh_key) = &server.ssh_key {
        println!("  SSH Key: {}", ssh_key.dimmed());
    }

    match server.last_seen {
        Some(time) => {
            let now = Utc::now();
            let duration = now.signed_duration_since(time);
            if duration.num_hours() < 1 {
                println!("  Last seen: {} ({})", time.format("%Y-%m-%d %H:%M UTC"), "Recently".green());
            } else if duration.num_days() < 1 {
                println!("  Last seen: {} ({})", time.format("%Y-%m-%d %H:%M UTC"), "Today".yellow());
            } else {
                println!("  Last seen: {} ({})", time.format("%Y-%m-%d %H:%M UTC"), format!("{} days ago", duration.num_days()).red());
            }
        }
        None => println!("  Last seen: {}", "Never".dimmed()),
    }

    if verbose {
        // Test connectivity
        print!("  Connectivity: ");
        match test_connectivity(server, &Config::default()).await {
            Ok(true) => println!("{}", "✅ Connected".green()),
            Ok(false) => println!("{}", "❌ Failed".red()),
            Err(_) => println!("{}", "❓ Error".yellow()),
        }
    }
}

fn print_servers_table(servers: &[(&String, &ServerDefinition)]) {
    let widths = [20, 25, 15, 8, 10];
    
    // Header
    println!("{}", format_table_row(&["NAME", "HOST", "USER", "PORT", "STATUS"], &widths));
    println!("{}", format_table_separator(&widths));
    
    // Rows
    for (name, server) in servers {
        let status = if server.enabled { "enabled" } else { "disabled" };
        let status_colored = if server.enabled { 
            status.green().to_string() 
        } else { 
            status.red().to_string() 
        };
        
        println!("{}", format_table_row(&[
            name,
            &server.host,
            &server.user,
            &server.port.to_string(),
            &status_colored,
        ], &widths));
    }
    
    println!();
    println!("Use --detailed flag for more information");
}

fn format_table_row(columns: &[&str], widths: &[usize]) -> String {
    let mut row = String::new();
    for (i, col) in columns.iter().enumerate() {
        if i < widths.len() {
            row.push_str(&format!("{:<width$}", col, width = widths[i]));
            if i < columns.len() - 1 {
                row.push_str(" | ");
            }
        }
    }
    row
}

fn format_table_separator(widths: &[usize]) -> String {
    widths.iter()
        .map(|&w| "-".repeat(w))
        .collect::<Vec<_>>()
        .join("-+-")
}