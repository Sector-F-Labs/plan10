use anyhow::Result;
use crate::{PowerActions, Config};
use crate::commands::utils::*;
use colored::*;
use std::process::Command;

pub async fn execute_power_action(
    action: PowerActions,
    _config: &Config,
    verbose: bool,
) -> Result<()> {
    match action {
        PowerActions::Status => {
            show_power_status(verbose).await
        }
        PowerActions::Configure { no_hibernate, no_sleep, halt_level } => {
            configure_power_settings(no_hibernate, no_sleep, halt_level, verbose).await
        }
        PowerActions::Reset => {
            reset_power_settings(verbose).await
        }
        PowerActions::Diagnostics => {
            run_power_diagnostics(verbose).await
        }
    }
}

async fn show_power_status(_verbose: bool) -> Result<()> {
    print_header("Power Management Status");
    
    // Get current power settings
    let output = Command::new("pmset")
        .arg("-g")
        .output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        println!("{}:", "Current Settings".bold());
        for line in stdout.lines() {
            if line.contains("hibernatemode") || 
               line.contains("standby") || 
               line.contains("powernap") ||
               line.contains("sleep") ||
               line.contains("disksleep") {
                println!("  {}", line.trim());
            }
        }
    }
    
    // Check power source
    let battery_output = Command::new("pmset")
        .args(&["-g", "batt"])
        .output()?;
    
    if battery_output.status.success() {
        let battery_info = String::from_utf8_lossy(&battery_output.stdout);
        println!("\n{}:", "Power Source".bold());
        if battery_info.contains("AC Power") {
            println!("  🔌 AC Power");
        } else if battery_info.contains("Battery Power") {
            println!("  🔋 Battery Power");
        } else {
            println!("  ❓ Unknown");
        }
        
        // Extract battery percentage if available
        for line in battery_info.lines() {
            if let Some(start) = line.find(char::is_numeric) {
                if let Some(end) = line[start..].find('%') {
                    let percentage = &line[start..start + end + 1];
                    println!("  Battery Level: {}", percentage);
                    break;
                }
            }
        }
    }
    
    // Check caffeinate status
    let caffeinate_output = Command::new("pgrep")
        .args(&["-x", "caffeinate"])
        .output()?;
    
    println!("\n{}:", "Keep Awake Status".bold());
    if caffeinate_output.status.success() && !caffeinate_output.stdout.is_empty() {
        let pid_output = String::from_utf8_lossy(&caffeinate_output.stdout);
        let pid = pid_output.trim();
        println!("  ☕ Caffeinate: ✅ Running (PID: {})", pid);
    } else {
        println!("  ☕ Caffeinate: ❌ Not running");
    }
    
    Ok(())
}

async fn configure_power_settings(
    no_hibernate: bool,
    no_sleep: bool,
    halt_level: Option<u8>,
    verbose: bool,
) -> Result<()> {
    print_header("Configuring Power Settings");
    
    let mut commands = Vec::new();
    
    if no_hibernate {
        commands.push(("Disable hibernation", vec!["pmset", "-a", "hibernatemode", "0"]));
    }
    
    if no_sleep {
        commands.push(("Disable system sleep", vec!["pmset", "-a", "sleep", "0"]));
        commands.push(("Disable disk sleep", vec!["pmset", "-a", "disksleep", "0"]));
        commands.push(("Disable standby", vec!["pmset", "-a", "standby", "0"]));
    }
    
    let halt_level_string;
    if let Some(level) = halt_level {
        halt_level_string = level.to_string();
        commands.push(("Set battery halt level", vec!["pmset", "-b", "haltlevel", &halt_level_string]));
    }
    
    // Always disable powernap for servers
    commands.push(("Disable power nap", vec!["pmset", "-a", "powernap", "0"]));
    
    for (description, args) in commands {
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
    
    print_success("Power settings configuration completed");
    Ok(())
}

async fn reset_power_settings(_verbose: bool) -> Result<()> {
    print_header("Resetting Power Settings");
    
    print_warning("This will reset ALL power management settings to macOS defaults");
    print_info("You may need to reconfigure settings for server operation afterwards");
    
    let output = Command::new("sudo")
        .args(&["pmset", "-a", "restoredefaults"])
        .output()?;
    
    if output.status.success() {
        print_success("Power settings reset to defaults");
        print_info("Consider running 'plan10 server power configure' to optimize for server use");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        print_error(&format!("Failed to reset power settings: {}", stderr));
    }
    
    Ok(())
}

async fn run_power_diagnostics(verbose: bool) -> Result<()> {
    print_header("Power Management Diagnostics");
    
    // Use the power diagnostics from the shared module
    crate::commands::shared::power_diagnostics::execute_power_diagnostics_command(
        false, false, false, true, false, None, 
        &Config::default(), 
        crate::ExecutionMode::Local, 
        verbose
    ).await
}