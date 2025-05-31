use anyhow::Result;
use crate::Config;
use crate::commands::utils::*;
use colored::*;
use std::process::Command;
use std::io::{self, Write};

pub async fn execute_configure(
    yes: bool,
    power: bool,
    monitoring: bool,
    services: bool,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    print_header("Plan 10 Server Configuration");
    
    // Check if we're on macOS
    if !cfg!(target_os = "macos") {
        print_warning("Server configuration is optimized for macOS");
        if !yes && !prompt_yes_no("Continue anyway?", false)? {
            return Ok(());
        }
    }
    
    let configure_all = !power && !monitoring && !services;
    
    if configure_all || power {
        configure_power_management(yes, verbose).await?;
    }
    
    if configure_all || monitoring {
        configure_monitoring(config, verbose).await?;
    }
    
    if configure_all || services {
        configure_services(verbose).await?;
    }
    
    print_success("Server configuration completed successfully!");
    
    // Show next steps
    println!("\n{}:", "Next Steps".bold());
    println!("1. Check status: plan10 status --detailed");
    println!("2. Start monitoring: plan10 monitor watch");
    println!("3. Test power settings: plan10 monitor power --all");
    
    Ok(())
}

async fn configure_power_management(skip_prompts: bool, verbose: bool) -> Result<()> {
    print_header("Power Management Configuration");
    
    if !skip_prompts {
        print_info("This will configure macOS power settings for optimal server operation");
        print_warning("This requires sudo privileges and will modify system settings");
        
        if !prompt_yes_no("Continue with power management configuration?", true)? {
            print_info("Skipping power management configuration");
            return Ok(());
        }
    }
    
    print_info("Configuring power management settings...");
    
    let power_commands = vec![
        ("Disable hibernation", vec!["pmset", "-a", "hibernatemode", "0"]),
        ("Disable standby", vec!["pmset", "-a", "standby", "0"]),
        ("Disable power nap", vec!["pmset", "-a", "powernap", "0"]),
        ("Disable system sleep", vec!["pmset", "-a", "sleep", "0"]),
        ("Disable disk sleep", vec!["pmset", "-a", "disksleep", "0"]),
        ("Set battery halt level", vec!["pmset", "-b", "haltlevel", "5"]),
        ("Disable auto power off", vec!["pmset", "-a", "autopoweroff", "0"]),
        ("Enable restart after power failure", vec!["pmset", "-a", "autorestart", "1"]),
        ("Enable restart after system freeze", vec!["pmset", "-a", "restartfreeze", "1"]),
    ];
    
    for (description, args) in power_commands {
        print_verbose(&format!("Running: sudo {}", args.join(" ")), verbose);
        
        let output = Command::new("sudo")
            .args(&args)
            .output()?;
        
        if output.status.success() {
            print_success(description);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            print_error(&format!("{} failed: {}", description, stderr));
        }
    }
    
    // Start caffeinate if not already running
    print_info("Starting caffeinate process...");
    let caffeinate_check = Command::new("pgrep")
        .args(&["-x", "caffeinate"])
        .output()?;
    
    if !caffeinate_check.status.success() || caffeinate_check.stdout.is_empty() {
        let _caffeinate_start = Command::new("nohup")
            .args(&["caffeinate", "-imsud"])
            .spawn()?;
        print_success("Caffeinate process started");
    } else {
        print_info("Caffeinate already running");
    }
    
    print_success("Power management configuration completed");
    Ok(())
}

async fn configure_monitoring(config: &Config, verbose: bool) -> Result<()> {
    print_header("Monitoring Configuration");
    
    // Ensure monitoring directories exist
    super::ensure_directories().await?;
    
    // Configure monitoring intervals and thresholds
    print_info("Setting up monitoring configuration...");
    
    let monitoring_config = format!(
        r#"# Plan 10 Monitoring Configuration
TEMP_THRESHOLD={}
BATTERY_WARNING={}
MONITORING_INTERVAL={}
LOG_LEVEL={}
"#,
        config.server.temp_threshold,
        config.server.battery_warning_level,
        config.server.monitoring_interval,
        config.server.log_level
    );
    
    let config_path = shellexpand::tilde("~/Library/Application Support/plan10/monitor.conf");
    let config_dir = std::path::Path::new(&*config_path).parent().unwrap();
    
    std::fs::create_dir_all(config_dir)?;
    std::fs::write(&*config_path, monitoring_config)?;
    
    print_success(&format!("Monitoring configuration written to {}", config_path));
    
    // Set up log rotation
    configure_log_rotation(verbose).await?;
    
    Ok(())
}

async fn configure_services(verbose: bool) -> Result<()> {
    print_header("Services Configuration");
    
    // Configure LaunchAgent for caffeinate
    print_info("Setting up LaunchAgent for caffeinate...");
    
    let launch_agent_path = shellexpand::tilde("~/Library/LaunchAgents/com.plan10.caffeinate.plist");
    let launch_agent_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.plan10.caffeinate</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/bin/caffeinate</string>
        <string>-imsud</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/plan10-caffeinate.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/plan10-caffeinate.log</string>
</dict>
</plist>"#;
    
    std::fs::write(&*launch_agent_path, launch_agent_content)?;
    print_success("LaunchAgent plist created");
    
    // Load the LaunchAgent
    let load_result = Command::new("launchctl")
        .args(&["load", &*launch_agent_path])
        .output()?;
    
    if load_result.status.success() {
        print_success("LaunchAgent loaded successfully");
    } else {
        print_warning("LaunchAgent may already be loaded or failed to load");
        if verbose {
            let stderr = String::from_utf8_lossy(&load_result.stderr);
            println!("launchctl output: {}", stderr);
        }
    }
    
    Ok(())
}

async fn configure_log_rotation(verbose: bool) -> Result<()> {
    print_info("Setting up log rotation...");
    
    let log_dir = shellexpand::tilde("~/logs");
    std::fs::create_dir_all(&*log_dir)?;
    
    // Create a simple log rotation script
    let log_rotate_script = shellexpand::tilde("~/scripts/rotate_logs.sh");
    let script_content = r#"#!/bin/bash
# Plan 10 log rotation script

LOG_DIR="$HOME/logs"
MAX_SIZE=10485760  # 10MB

for log_file in "$LOG_DIR"/*.log; do
    if [[ -f "$log_file" ]] && [[ $(stat -f%z "$log_file" 2>/dev/null || echo 0) -gt $MAX_SIZE ]]; then
        mv "$log_file" "${log_file}.old"
        touch "$log_file"
        echo "Rotated: $log_file"
    fi
done
"#;
    
    std::fs::write(&*log_rotate_script, script_content)?;
    
    // Make script executable
    Command::new("chmod")
        .args(&["+x", &*log_rotate_script])
        .output()?;
    
    print_success("Log rotation configured");
    Ok(())
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