use anyhow::Result;
use crate::{Config, SetupMode};
use crate::commands::utils::*;
use crate::config::ServerDefinition;
use colored::*;
use std::io::{self, Write};

pub async fn execute(
    mode: SetupMode,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    match mode {
        SetupMode::Auto => {
            auto_setup(config, verbose).await
        }
        SetupMode::Client => {
            client_setup(config, verbose).await
        }
        SetupMode::Server => {
            server_setup(config, verbose).await
        }
        SetupMode::Both => {
            client_setup(config, verbose).await?;
            println!();
            server_setup(config, verbose).await
        }
    }
}

async fn auto_setup(config: &Config, verbose: bool) -> Result<()> {
    print_header("Plan 10 Interactive Setup");
    
    println!("Welcome to Plan 10! This wizard will help you configure your environment.\n");
    
    // Detect environment
    let is_macos = cfg!(target_os = "macos");
    let has_servers = !config.servers.is_empty();
    
    if is_macos {
        println!("{} Detected macOS system", "✅".green());
        if has_servers {
            println!("{} Found {} configured server(s)", "ℹ️".blue(), config.servers.len());
            println!("\nWhat would you like to set up?");
            println!("1. Configure this machine as a Plan 10 server");
            println!("2. Add/manage remote servers (client mode)");
            println!("3. Both server and client configuration");
            println!("4. Skip setup");
            
            let choice = prompt_choice("Enter your choice (1-4)", &["1", "2", "3", "4"])?;
            match choice.as_str() {
                "1" => server_setup(config, verbose).await,
                "2" => client_setup(config, verbose).await,
                "3" => {
                    server_setup(config, verbose).await?;
                    println!();
                    client_setup(config, verbose).await
                }
                _ => {
                    print_info("Setup skipped");
                    Ok(())
                }
            }
        } else {
            println!("No servers configured yet.\n");
            println!("Since you're on macOS, would you like to:");
            println!("1. Set up this machine as a Plan 10 server");
            println!("2. Configure client mode to manage remote servers");
            println!("3. Both");
            
            let choice = prompt_choice("Enter your choice (1-3)", &["1", "2", "3"])?;
            match choice.as_str() {
                "1" => server_setup(config, verbose).await,
                "2" => client_setup(config, verbose).await,
                _ => {
                    server_setup(config, verbose).await?;
                    println!();
                    client_setup(config, verbose).await
                }
            }
        }
    } else {
        println!("{} Non-macOS system detected", "ℹ️".blue());
        println!("Client mode is recommended for non-macOS systems.\n");
        client_setup(config, verbose).await
    }
}

async fn client_setup(config: &Config, verbose: bool) -> Result<()> {
    print_header("Client Configuration");
    
    println!("Setting up Plan 10 client for managing remote servers.\n");
    
    let mut new_config = config.clone();
    
    // SSH key configuration
    println!("{}:", "SSH Configuration".bold());
    let default_key = dirs::home_dir()
        .map(|home| home.join(".ssh").join("id_rsa"))
        .and_then(|path| if path.exists() { Some(path.display().to_string()) } else { None });
    
    let ssh_key = if let Some(default) = default_key {
        let use_default = prompt_yes_no(&format!("Use SSH key at {}?", default), true)?;
        if use_default {
            Some(default)
        } else {
            prompt_optional("Enter SSH key path (or press Enter to skip)")
        }
    } else {
        prompt_optional("Enter SSH key path (or press Enter to skip)")
    };
    
    if let Some(key_path) = ssh_key {
        new_config.ssh.key_path = Some(key_path);
        print_success("SSH key path configured");
    }
    
    // Server configuration
    println!("\n{}:", "Server Configuration".bold());
    
    if config.servers.is_empty() {
        println!("No servers configured yet. Let's add your first server!");
        add_server_interactive(&mut new_config).await?;
    } else {
        println!("Current servers:");
        for (name, server) in &config.servers {
            let status = if server.enabled { "enabled" } else { "disabled" };
            println!("  • {} ({}@{}) - {}", name, server.user, server.host, status);
        }
        
        if prompt_yes_no("Would you like to add another server?", false)? {
            add_server_interactive(&mut new_config).await?;
        }
    }
    
    // Default server
    if new_config.servers.len() > 1 {
        println!("\n{}:", "Default Server".bold());
        let server_names: Vec<String> = new_config.servers.keys().cloned().collect();
        println!("Available servers:");
        for (i, name) in server_names.iter().enumerate() {
            println!("  {}. {}", i + 1, name);
        }
        
        let choice = prompt_choice("Select default server (or press Enter to skip)", 
                                 &server_names.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;
        if !choice.is_empty() {
            new_config.client.default_server = Some(choice.clone());
            print_success(&format!("Default server set to: {}", choice));
        }
    } else if new_config.servers.len() == 1 {
        let server_name = new_config.servers.keys().next().unwrap().clone();
        new_config.client.default_server = Some(server_name.clone());
        print_success(&format!("Default server set to: {}", server_name));
    }
    
    // Save configuration
    new_config.save(None)?;
    print_success("Client configuration saved!");
    
    // Next steps
    println!("\n{}:", "Next Steps".bold());
    println!("1. Test connection: plan10 client list");
    println!("2. Deploy to server: plan10 client deploy --host <server>");
    println!("3. Monitor remotely: plan10 monitor system --host <server>");
    
    Ok(())
}

async fn server_setup(config: &Config, verbose: bool) -> Result<()> {
    print_header("Server Configuration");
    
    println!("Setting up this machine as a Plan 10 server.\n");
    
    // Check requirements
    if !cfg!(target_os = "macos") {
        print_warning("Server mode is designed for macOS systems");
        if !prompt_yes_no("Continue anyway?", false)? {
            return Ok(());
        }
    }
    
    let mut new_config = config.clone();
    
    // Server name
    println!("{}:", "Server Identity".bold());
    let current_hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let server_name = prompt_with_default("Server name", &current_hostname)?;
    new_config.server.name = server_name;
    
    // Monitoring configuration
    println!("\n{}:", "Monitoring Configuration".bold());
    
    let temp_threshold = prompt_with_default_parsed(
        "Temperature warning threshold (°C)", 
        new_config.server.temp_threshold
    )?;
    new_config.server.temp_threshold = temp_threshold;
    
    let battery_warning = prompt_with_default_parsed(
        "Battery warning level (%)", 
        new_config.server.battery_warning_level as f32
    )? as u8;
    new_config.server.battery_warning_level = battery_warning;
    
    let monitor_interval = prompt_with_default_parsed(
        "Monitoring interval (seconds)", 
        new_config.server.monitoring_interval as f32
    )? as u64;
    new_config.server.monitoring_interval = monitor_interval;
    
    // Services configuration
    println!("\n{}:", "Services Configuration".bold());
    let auto_restart = prompt_yes_no("Auto-restart services on failure?", 
                                   new_config.server.auto_restart_services)?;
    new_config.server.auto_restart_services = auto_restart;
    
    // Power management setup
    println!("\n{}:", "Power Management".bold());
    println!("Plan 10 requires specific power settings for reliable server operation.");
    
    if prompt_yes_no("Configure power management now? (requires sudo)", true)? {
        print_info("You may be prompted for your password to configure power settings");
        
        match configure_power_management().await {
            Ok(_) => print_success("Power management configured successfully"),
            Err(e) => {
                print_warning(&format!("Power management setup failed: {}", e));
                println!("You can run this manually later: sudo ./server_setup.sh");
            }
        }
    } else {
        print_info("Power management setup skipped");
        println!("Remember to run: sudo ./server_setup.sh");
    }
    
    // Save configuration
    new_config.save(None)?;
    print_success("Server configuration saved!");
    
    // Setup monitoring scripts
    if prompt_yes_no("Set up monitoring script aliases?", true)? {
        setup_monitoring_aliases().await?;
    }
    
    // Next steps
    println!("\n{}:", "Next Steps".bold());
    println!("1. Test monitoring: plan10 monitor system");
    println!("2. Check status: plan10 status --detailed");
    println!("3. View logs: tail -f /var/log/plan10.log");
    
    if !new_config.server.auto_restart_services {
        println!("4. Start services: plan10 server start");
    }
    
    Ok(())
}

async fn add_server_interactive(config: &mut Config) -> Result<()> {
    println!("\n{}:", "Add Server".bold());
    
    let name = prompt("Server name")?;
    let host = prompt("Hostname or IP address")?;
    let user = prompt("SSH username")?;
    let port = prompt_with_default_parsed("SSH port", 22.0)? as u16;
    
    let ssh_key = prompt_optional("SSH key path (or press Enter to use default)");
    
    let server = ServerDefinition {
        name: name.clone(),
        host,
        user,
        port,
        ssh_key,
        tags: vec!["manual".to_string()],
        enabled: true,
        last_seen: None,
    };
    
    config.add_server(server)?;
    print_success(&format!("Server '{}' added successfully", name));
    
    Ok(())
}

async fn configure_power_management() -> Result<()> {
    use std::process::Command;
    
    let commands = vec![
        ("pmset", vec!["-a", "hibernatemode", "0"]),
        ("pmset", vec!["-a", "standby", "0"]),
        ("pmset", vec!["-a", "powernap", "0"]),
        ("pmset", vec!["-a", "sleep", "0"]),
        ("pmset", vec!["-a", "disksleep", "0"]),
        ("pmset", vec!["-b", "haltlevel", "5"]),
        ("pmset", vec!["-a", "autopoweroff", "0"]),
    ];
    
    for (cmd, args) in commands {
        let output = Command::new("sudo")
            .arg(cmd)
            .args(&args)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to run {} {}: {}", cmd, args.join(" "), stderr);
        }
    }
    
    // Start caffeinate
    Command::new("nohup")
        .args(&["caffeinate", "-imsud"])
        .spawn()?;
    
    Ok(())
}

async fn setup_monitoring_aliases() -> Result<()> {
    use std::fs;
    use std::path::PathBuf;
    
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let shell_rc = if std::env::var("SHELL").unwrap_or_default().contains("zsh") {
        home.join(".zshrc")
    } else {
        home.join(".bashrc")
    };
    
    let aliases = r#"
# Plan 10 System Monitoring Aliases
alias temp='plan10 monitor temp'
alias battery='plan10 monitor battery'
alias sysmon='plan10 monitor system'
alias plan10-status='plan10 status'
"#;
    
    if shell_rc.exists() {
        fs::write(&shell_rc, format!("{}\n{}", fs::read_to_string(&shell_rc)?, aliases))?;
    } else {
        fs::write(&shell_rc, aliases)?;
    }
    
    print_success(&format!("Aliases added to {}", shell_rc.display()));
    println!("Run 'source {}' to activate aliases", shell_rc.display());
    
    Ok(())
}

// Helper functions for user input
fn prompt(message: &str) -> Result<String> {
    print!("{}: ", message.cyan());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_with_default(message: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", message.cyan(), default.dimmed());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}

fn prompt_with_default_parsed<T: std::str::FromStr + std::fmt::Display>(
    message: &str, 
    default: T
) -> Result<T> {
    loop {
        let input = prompt_with_default(message, &default.to_string())?;
        match input.parse() {
            Ok(value) => return Ok(value),
            Err(_) => print_warning(&format!("Invalid input: {}", input)),
        }
    }
}

fn prompt_optional(message: &str) -> Option<String> {
    print!("{}: ", message.cyan());
    io::stdout().flush().ok()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let input = input.trim();
    
    if input.is_empty() {
        None
    } else {
        Some(input.to_string())
    }
}

fn prompt_yes_no(message: &str, default: bool) -> Result<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", message.cyan(), default_str.dimmed());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    match input.as_str() {
        "" => Ok(default),
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => {
            print_warning("Please enter 'y' or 'n'");
            prompt_yes_no(message, default)
        }
    }
}

fn prompt_choice(message: &str, choices: &[&str]) -> Result<String> {
    print!("{}: ", message.cyan());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    
    if input.is_empty() {
        return Ok(String::new());
    }
    
    // Try to match by number
    if let Ok(index) = input.parse::<usize>() {
        if index > 0 && index <= choices.len() {
            return Ok(choices[index - 1].to_string());
        }
    }
    
    // Try to match by string
    for choice in choices {
        if choice.eq_ignore_ascii_case(input) {
            return Ok(choice.to_string());
        }
    }
    
    print_warning(&format!("Invalid choice: {}", input));
    prompt_choice(message, choices)
}

pub fn show_help() {
    println!("Usage: plan10 setup [mode]");
    println!();
    println!("Modes:");
    println!("  auto     Auto-detect and configure (default)");
    println!("  client   Configure client mode only");
    println!("  server   Configure server mode only");
    println!("  both     Configure both client and server");
    println!();
    println!("Examples:");
    println!("  plan10 setup              # Interactive auto-setup");
    println!("  plan10 setup client       # Client-only setup");
    println!("  plan10 setup server       # Server-only setup");
}