use anyhow::Result;
use crate::{MaintenanceActions, Config};
use crate::commands::utils::*;
use colored::*;
use std::process::Command;
use std::fs;
use std::path::Path;

pub async fn execute_maintenance_action(
    action: MaintenanceActions,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    match action {
        MaintenanceActions::Update => {
            update_system(verbose).await
        }
        MaintenanceActions::Clean => {
            clean_temporary_files(verbose).await
        }
        MaintenanceActions::Backup { output } => {
            backup_configuration(output, config, verbose).await
        }
        MaintenanceActions::Restore { input } => {
            restore_configuration(input, verbose).await
        }
        MaintenanceActions::Health => {
            run_health_check(config, verbose).await
        }
    }
}

async fn update_system(verbose: bool) -> Result<()> {
    print_header("System Update");
    
    // Check if Homebrew is available
    if Command::new("which").arg("brew").output()?.status.success() {
        print_info("Updating Homebrew packages...");
        
        let brew_update = Command::new("brew")
            .arg("update")
            .output()?;
        
        if brew_update.status.success() {
            print_success("Homebrew updated");
            
            let brew_upgrade = Command::new("brew")
                .arg("upgrade")
                .output()?;
            
            if brew_upgrade.status.success() {
                print_success("Homebrew packages upgraded");
            } else {
                print_warning("Some Homebrew packages failed to upgrade");
            }
        } else {
            print_warning("Failed to update Homebrew");
        }
    } else {
        print_info("Homebrew not found, skipping package updates");
    }
    
    // Check for macOS updates
    print_info("Checking for macOS updates...");
    let softwareupdate = Command::new("softwareupdate")
        .args(&["-l", "--no-scan"])
        .output()?;
    
    if softwareupdate.status.success() {
        let output_str = String::from_utf8_lossy(&softwareupdate.stdout);
        if output_str.contains("No new software available") {
            print_success("macOS is up to date");
        } else {
            print_info("macOS updates available. Run 'sudo softwareupdate -i -a' to install");
        }
    }
    
    Ok(())
}

async fn clean_temporary_files(verbose: bool) -> Result<()> {
    print_header("Cleaning Temporary Files");
    
    let temp_paths = vec![
        "/tmp/plan10-*",
        "~/Library/Caches/plan10",
        "~/logs/*.log.old",
        "/var/log/plan10*.log.*",
    ];
    
    for path_pattern in temp_paths {
        let expanded_path = shellexpand::tilde(path_pattern);
        print_verbose(&format!("Cleaning: {}", expanded_path), verbose);
        
        // Use find command to locate and remove files
        let find_result = Command::new("find")
            .args(&[&*expanded_path, "-type", "f", "-delete"])
            .output();
        
        match find_result {
            Ok(output) if output.status.success() => {
                print_success(&format!("Cleaned: {}", path_pattern));
            }
            Ok(_) => {
                print_verbose(&format!("No files found matching: {}", path_pattern), verbose);
            }
            Err(_) => {
                print_verbose(&format!("Could not clean: {}", path_pattern), verbose);
            }
        }
    }
    
    // Clean system caches if requested
    print_info("Cleaning system caches...");
    let cache_clean = Command::new("sudo")
        .args(&["purge"])
        .output();
    
    match cache_clean {
        Ok(output) if output.status.success() => {
            print_success("System caches purged");
        }
        _ => {
            print_info("Could not purge system caches (requires sudo)");
        }
    }
    
    Ok(())
}

async fn backup_configuration(
    output: Option<String>,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    print_header("Configuration Backup");
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_filename = output.unwrap_or_else(|| {
        format!("plan10_backup_{}.tar.gz", timestamp)
    });
    
    print_info(&format!("Creating backup: {}", backup_filename));
    
    // Create temporary directory for backup
    let temp_dir = tempfile::tempdir()?;
    let backup_dir = temp_dir.path().join("plan10_backup");
    fs::create_dir_all(&backup_dir)?;
    
    // Copy configuration files
    let config_files = vec![
        ("config.toml", Config::default_config_path()),
        ("caffeinate.plist", Some(shellexpand::tilde("~/Library/LaunchAgents/caffeinate.plist").to_string().into())),
        ("scripts", Some(shellexpand::tilde("~/scripts").to_string().into())),
    ];
    
    for (name, source_path_opt) in config_files {
        if let Some(source_path) = source_path_opt {
            if source_path.exists() {
                let dest_path = backup_dir.join(name);
                if source_path.is_dir() {
                    copy_dir_recursive(&source_path, &dest_path)?;
                } else {
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(&source_path, &dest_path)?;
                }
                print_verbose(&format!("Backed up: {}", name), verbose);
            }
        }
    }
    
    // Create backup info file
    let backup_info = format!(
        "Plan 10 Backup\nCreated: {}\nHostname: {}\nVersion: {}\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        hostname::get().unwrap_or_default().to_string_lossy(),
        env!("CARGO_PKG_VERSION")
    );
    fs::write(backup_dir.join("backup_info.txt"), backup_info)?;
    
    // Create tar archive
    let tar_result = Command::new("tar")
        .args(&["-czf", &backup_filename, "-C", temp_dir.path().to_str().unwrap(), "plan10_backup"])
        .output()?;
    
    if tar_result.status.success() {
        print_success(&format!("Backup created: {}", backup_filename));
    } else {
        let stderr = String::from_utf8_lossy(&tar_result.stderr);
        print_error(&format!("Backup failed: {}", stderr));
    }
    
    Ok(())
}

async fn restore_configuration(input: String, verbose: bool) -> Result<()> {
    print_header(&format!("Restoring Configuration from: {}", input));
    
    if !Path::new(&input).exists() {
        anyhow::bail!("Backup file not found: {}", input);
    }
    
    print_warning("This will overwrite existing Plan 10 configuration!");
    print_info("Make sure to backup current configuration first");
    
    // Extract backup
    let temp_dir = tempfile::tempdir()?;
    let extract_result = Command::new("tar")
        .args(&["-xzf", &input, "-C", temp_dir.path().to_str().unwrap()])
        .output()?;
    
    if !extract_result.status.success() {
        let stderr = String::from_utf8_lossy(&extract_result.stderr);
        anyhow::bail!("Failed to extract backup: {}", stderr);
    }
    
    let backup_dir = temp_dir.path().join("plan10_backup");
    if !backup_dir.exists() {
        anyhow::bail!("Invalid backup format");
    }
    
    // Show backup info
    let backup_info_path = backup_dir.join("backup_info.txt");
    if backup_info_path.exists() {
        let backup_info = fs::read_to_string(&backup_info_path)?;
        print_info("Backup Information:");
        for line in backup_info.lines() {
            println!("  {}", line);
        }
        println!();
    }
    
    // Restore files
    let restore_files = vec![
        ("config.toml", Config::default_config_path()),
        ("caffeinate.plist", Some(shellexpand::tilde("~/Library/LaunchAgents/caffeinate.plist").to_string().into())),
        ("scripts", Some(shellexpand::tilde("~/scripts").to_string().into())),
    ];
    
    for (name, dest_path_opt) in restore_files {
        let source_path = backup_dir.join(name);
        if source_path.exists() {
            if let Some(dest_path) = dest_path_opt {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                if source_path.is_dir() {
                    if dest_path.exists() {
                        fs::remove_dir_all(&dest_path)?;
                    }
                    copy_dir_recursive(&source_path, &dest_path)?;
                } else {
                    fs::copy(&source_path, &dest_path)?;
                }
                print_success(&format!("Restored: {}", name));
            }
        }
    }
    
    print_success("Configuration restored successfully");
    print_info("You may need to restart services for changes to take effect");
    
    Ok(())
}

async fn run_health_check(config: &Config, verbose: bool) -> Result<()> {
    print_header("System Health Check");
    
    let mut issues = 0;
    let mut warnings = 0;
    
    // Check essential files
    println!("{}:", "Essential Files".bold());
    let essential_files = vec![
        ("Config file", Config::default_config_path()),
        ("Scripts directory", Some(shellexpand::tilde("~/scripts").to_string().into())),
        ("LaunchAgent", Some(shellexpand::tilde("~/Library/LaunchAgents/caffeinate.plist").to_string().into())),
    ];
    
    for (name, path_opt) in essential_files {
        if let Some(path) = path_opt {
            if path.exists() {
                print_success(&format!("{}: Present", name));
            } else {
                print_error(&format!("{}: Missing", name));
                issues += 1;
            }
        }
    }
    
    // Check services
    println!("\n{}:", "Services".bold());
    let caffeinate_running = Command::new("pgrep")
        .args(&["-x", "caffeinate"])
        .output()?
        .status.success();
    
    if caffeinate_running {
        print_success("Caffeinate: Running");
    } else {
        print_warning("Caffeinate: Not running");
        warnings += 1;
    }
    
    // Check power settings
    println!("\n{}:", "Power Settings".bold());
    let pmset_output = Command::new("pmset")
        .arg("-g")
        .output()?;
    
    if pmset_output.status.success() {
        let output_str = String::from_utf8_lossy(&pmset_output.stdout);
        let problematic_settings = vec![
            ("hibernatemode", "0"),
            ("standby", "0"),
            ("powernap", "0"),
        ];
        
        for (setting, expected) in problematic_settings {
            if let Some(line) = output_str.lines().find(|l| l.contains(setting)) {
                if line.contains(&format!("{} {}", setting, expected)) {
                    print_success(&format!("{}: Configured correctly", setting));
                } else {
                    print_warning(&format!("{}: May need adjustment", setting));
                    warnings += 1;
                }
            }
        }
    }
    
    // Check disk space
    println!("\n{}:", "Disk Space".bold());
    let df_output = Command::new("df")
        .args(&["-h", "/"])
        .output()?;
    
    if df_output.status.success() {
        let output_str = String::from_utf8_lossy(&df_output.stdout);
        if let Some(line) = output_str.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let usage = parts[4].trim_end_matches('%');
                if let Ok(usage_pct) = usage.parse::<u32>() {
                    if usage_pct > 90 {
                        print_error(&format!("Disk usage: {}% (Critical)", usage_pct));
                        issues += 1;
                    } else if usage_pct > 80 {
                        print_warning(&format!("Disk usage: {}% (High)", usage_pct));
                        warnings += 1;
                    } else {
                        print_success(&format!("Disk usage: {}% (OK)", usage_pct));
                    }
                }
            }
        }
    }
    
    // Summary
    println!("\n{}:", "Health Summary".bold());
    if issues == 0 && warnings == 0 {
        print_success("All systems healthy! 🎉");
    } else {
        if issues > 0 {
            print_error(&format!("Found {} critical issue(s)", issues));
        }
        if warnings > 0 {
            print_warning(&format!("Found {} warning(s)", warnings));
        }
        println!("\nRecommendations:");
        if issues > 0 {
            println!("  • Run 'plan10 server configure' to fix configuration issues");
        }
        if warnings > 0 {
            println!("  • Run 'plan10 monitor power --fixes' for power management recommendations");
        }
    }
    
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}