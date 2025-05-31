use anyhow::Result;
use crate::Config;
use crate::commands::utils::*;
use crate::ssh::{SshClient, CommandResult};
use crate::{ExecutionMode, MonitorCommands};
use colored::*;
use sysinfo::{System, SystemExt, CpuExt};
use std::process::Command;

pub struct TempMonitor {
    execution_mode: ExecutionMode,
    config: Config,
}

impl TempMonitor {
    pub fn new(execution_mode: ExecutionMode, config: Config) -> Self {
        Self {
            execution_mode,
            config,
        }
    }

    pub async fn execute(&self, raw: bool, host: Option<String>, verbose: bool) -> Result<()> {
        match &self.execution_mode {
            ExecutionMode::Local => {
                self.execute_local(raw, verbose).await
            }
            ExecutionMode::Remote { host: default_host } => {
                let target_host = host.unwrap_or_else(|| default_host.clone());
                self.execute_remote(&target_host, raw, verbose).await
            }
            ExecutionMode::Auto => {
                if let Some(target_host) = host {
                    self.execute_remote(&target_host, raw, verbose).await
                } else {
                    self.execute_local(raw, verbose).await
                }
            }
        }
    }

    async fn execute_local(&self, raw: bool, verbose: bool) -> Result<()> {
        if raw {
            self.display_raw_temp().await
        } else {
            self.display_formatted_temp(verbose).await
        }
    }

    async fn execute_remote(&self, host: &str, raw: bool, verbose: bool) -> Result<()> {
        let server = self.config.resolve_server(host)
            .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", host))?;

        let client = SshClient::connect(server, &self.config).await?;
        
        let command = if raw {
            "~/scripts/temp -r"
        } else {
            "~/scripts/temp"
        };

        let result = client.execute_command(command)?;
        
        if result.success {
            println!("{}", result.stdout);
        } else {
            print_error(&format!("Remote command failed: {}", result.stderr));
        }

        Ok(())
    }

    async fn display_formatted_temp(&self, verbose: bool) -> Result<()> {
        print_header("System Temperature Status");

        // Try to get detailed temperature using powermetrics (requires sudo)
        if let Ok(temp_data) = self.get_powermetrics_temp().await {
            if !temp_data.is_empty() {
                println!("{}", temp_data);
            } else {
                print_warning("Unable to get detailed temperature (requires sudo)");
            }
        }

        // Get thermal state using system_profiler
        if let Ok(thermal_state) = self.get_thermal_state().await {
            if !thermal_state.is_empty() {
                println!("{}", thermal_state);
            }
        }

        // Get CPU usage as thermal indicator
        let cpu_usage = self.get_cpu_usage().await?;
        println!("CPU Usage: {:.1}%", cpu_usage);

        // Color code based on usage
        if cpu_usage > 80.0 {
            println!("{} High CPU load - system may be hot", "🔥".red());
        } else if cpu_usage > 50.0 {
            println!("{} Moderate CPU load", "🔶".yellow());
        } else {
            println!("{} Low CPU load - system cool", "❄️".blue());
        }

        // Show fan status if available
        if let Ok(fan_info) = self.get_fan_status().await {
            if !fan_info.is_empty() {
                println!("\n{} Fan Status:", "💨".cyan());
                println!("{}", fan_info);
            }
        }

        Ok(())
    }

    async fn display_raw_temp(&self) -> Result<()> {
        if let Ok(output) = self.get_powermetrics_temp().await {
            println!("{}", output);
        } else {
            println!("Unable to get raw temperature data");
        }
        Ok(())
    }

    async fn get_powermetrics_temp(&self) -> Result<String> {
        let output = Command::new("sudo")
            .args(&["powermetrics", "--samplers", "smc", "-n", "1", "-i", "1000"])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let temp_lines: Vec<&str> = stdout
                .lines()
                .filter(|line| {
                    line.contains("CPU die temperature") || 
                    line.contains("GPU die temperature")
                })
                .take(2)
                .collect();
            
            Ok(temp_lines.join("\n"))
        } else {
            Err(anyhow::anyhow!("Failed to run powermetrics"))
        }
    }

    async fn get_thermal_state(&self) -> Result<String> {
        let output = Command::new("system_profiler")
            .arg("SPHardwareDataType")
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let thermal_line = stdout
                .lines()
                .find(|line| line.contains("Thermal State"))
                .unwrap_or("")
                .trim();
            
            Ok(thermal_line.to_string())
        } else {
            Ok(String::new())
        }
    }

    async fn get_cpu_usage(&self) -> Result<f32> {
        // Try using top command for CPU usage
        let output = Command::new("top")
            .args(&["-l", "1"])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            for line in stdout.lines() {
                if line.contains("CPU usage") {
                    if let Some(usage_str) = line.split_whitespace().nth(2) {
                        if let Ok(usage) = usage_str.trim_end_matches('%').parse::<f32>() {
                            return Ok(usage);
                        }
                    }
                }
            }
        }

        // Fallback to sysinfo
        let mut system = System::new_all();
        system.refresh_all();
        
        let cpu_usage = system.global_cpu_info().cpu_usage();
        Ok(cpu_usage)
    }

    async fn get_fan_status(&self) -> Result<String> {
        let output = Command::new("sudo")
            .args(&["powermetrics", "--samplers", "smc", "-n", "1", "-i", "500"])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let fan_lines: Vec<&str> = stdout
                .lines()
                .filter(|line| line.to_lowercase().contains("fan"))
                .take(3)
                .collect();
            
            Ok(fan_lines.join("\n"))
        } else {
            Ok(String::new())
        }
    }
}

pub async fn execute_temp_command(
    raw: bool,
    host: Option<String>,
    config: &Config,
    execution_mode: ExecutionMode,
    verbose: bool,
) -> Result<()> {
    let monitor = TempMonitor::new(execution_mode, config.clone());
    monitor.execute(raw, host, verbose).await
}

// Helper function for showing help
pub fn show_help() {
    println!("Usage: plan10 monitor temp [options]");
    println!();
    println!("Options:");
    println!("  -r, --raw         Show raw temperature data");
    println!("  -H, --host <HOST> Target server (remote monitoring)");
    println!("  -v, --verbose     Verbose output");
    println!("  -h, --help        Show this help message");
    println!();
    println!("Examples:");
    println!("  plan10 monitor temp                    # Local temperature");
    println!("  plan10 monitor temp --raw              # Raw temperature data");
    println!("  plan10 monitor temp --host myserver    # Remote temperature");
}