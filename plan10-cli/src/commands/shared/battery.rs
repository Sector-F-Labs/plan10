use anyhow::Result;
use crate::Config;
use crate::commands::utils::*;
use crate::ssh::SshClient;
use crate::ExecutionMode;
use colored::*;
use std::process::Command;
use chrono::{DateTime, Utc};

pub struct BatteryMonitor {
    execution_mode: ExecutionMode,
    config: Config,
}

impl BatteryMonitor {
    pub fn new(execution_mode: ExecutionMode, config: Config) -> Self {
        Self {
            execution_mode,
            config,
        }
    }

    pub async fn execute(&self, detailed: bool, raw: bool, host: Option<String>, verbose: bool) -> Result<()> {
        match &self.execution_mode {
            ExecutionMode::Local => {
                self.execute_local(detailed, raw, verbose).await
            }
            ExecutionMode::Remote { host: default_host } => {
                let target_host = host.unwrap_or_else(|| default_host.clone());
                self.execute_remote(&target_host, detailed, raw, verbose).await
            }
            ExecutionMode::Auto => {
                if let Some(target_host) = host {
                    self.execute_remote(&target_host, detailed, raw, verbose).await
                } else {
                    self.execute_local(detailed, raw, verbose).await
                }
            }
        }
    }

    async fn execute_local(&self, detailed: bool, raw: bool, verbose: bool) -> Result<()> {
        if raw {
            self.display_raw_battery().await
        } else if detailed {
            self.display_detailed_battery(verbose).await
        } else {
            self.display_formatted_battery(verbose).await
        }
    }

    async fn execute_remote(&self, host: &str, detailed: bool, raw: bool, verbose: bool) -> Result<()> {
        let server = self.config.resolve_server(host)
            .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", host))?;

        let client = SshClient::connect(server, &self.config).await?;
        
        let command = if raw {
            "~/scripts/battery -r"
        } else if detailed {
            "~/scripts/battery -d"
        } else {
            "~/scripts/battery"
        };

        let result = client.execute_command(command)?;
        
        if result.success {
            println!("{}", result.stdout);
        } else {
            print_error(&format!("Remote command failed: {}", result.stderr));
        }

        Ok(())
    }

    async fn display_formatted_battery(&self, verbose: bool) -> Result<()> {
        println!("{} Battery Status", "🔋".green());
        println!("{}", "=".repeat(18));

        let battery_info = self.get_battery_pmset().await?;
        
        if battery_info.is_empty() {
            println!("{} Unable to get battery information", "❌".red());
            println!("This device may not have a battery or battery monitoring is unavailable");
            return Ok(());
        }

        // Parse battery information
        let (percentage, status, time_remaining) = self.parse_battery_info(&battery_info)?;
        
        println!("Charge Level: {}", percentage);
        println!("Status: {}", status);
        
        if let Some(time) = time_remaining {
            println!("{}", time);
        }

        // Color code percentage
        if let Some(pct_str) = percentage.strip_suffix('%') {
            if let Ok(pct_num) = pct_str.parse::<u8>() {
                match pct_num {
                    0..=20 => println!("{} Low Battery - Consider charging", "🔴".red()),
                    21..=50 => println!("{} Medium Battery", "🟡".yellow()),
                    _ => println!("{} Good Battery Level", "🟢".green()),
                }
            }
        }

        Ok(())
    }

    async fn display_detailed_battery(&self, verbose: bool) -> Result<()> {
        self.display_formatted_battery(verbose).await?;
        
        println!();
        println!("{} Battery Health", "🏥".blue());
        println!("{}", "=".repeat(16));
        
        let health_info = self.get_battery_health().await?;
        
        if health_info.is_empty() {
            println!("{} Unable to get battery health information", "❌".red());
            return Ok(());
        }

        self.parse_and_display_health(&health_info)?;
        
        Ok(())
    }

    async fn display_raw_battery(&self) -> Result<()> {
        println!("Raw Battery Data:");
        println!("{}", "=".repeat(18));
        
        let pmset_output = self.get_battery_pmset().await?;
        println!("{}", pmset_output);
        
        println!();
        let detailed_output = self.get_battery_detailed().await?;
        if !detailed_output.is_empty() {
            println!("{}", detailed_output);
        }
        
        Ok(())
    }

    async fn get_battery_pmset(&self) -> Result<String> {
        let output = Command::new("pmset")
            .args(&["-g", "batt"])
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Ok(String::new())
        }
    }

    async fn get_battery_detailed(&self) -> Result<String> {
        let output = Command::new("system_profiler")
            .arg("SPPowerDataType")
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let battery_section: Vec<&str> = stdout
                .lines()
                .skip_while(|line| !line.to_lowercase().contains("battery"))
                .take(20)
                .collect();
            
            Ok(battery_section.join("\n"))
        } else {
            Ok(String::new())
        }
    }

    async fn get_battery_health(&self) -> Result<String> {
        let output = Command::new("system_profiler")
            .arg("SPPowerDataType")
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let health_lines: Vec<&str> = stdout
                .lines()
                .filter(|line| {
                    line.contains("Cycle Count") ||
                    line.contains("Condition") ||
                    line.contains("Full Charge Capacity") ||
                    line.contains("Maximum Capacity")
                })
                .collect();
            
            Ok(health_lines.join("\n"))
        } else {
            Ok(String::new())
        }
    }

    fn parse_battery_info(&self, battery_info: &str) -> Result<(String, String, Option<String>)> {
        let mut percentage = String::new();
        let mut status = String::new();
        let mut time_remaining = None;

        // Extract percentage
        for line in battery_info.lines() {
            if let Some(pct) = self.extract_percentage(line) {
                percentage = pct;
                break;
            }
        }

        // Extract charging status
        if battery_info.contains("AC Power") {
            status = "🔌 Charging (AC Power)".to_string();
        } else if battery_info.contains("discharging") {
            status = "⚡ Discharging".to_string();
        } else if battery_info.contains("charged") {
            status = "✅ Fully Charged".to_string();
        } else {
            status = "❓ Unknown".to_string();
        }

        // Extract time remaining
        for line in battery_info.lines() {
            if let Some(time) = self.extract_time_remaining(line, &battery_info) {
                time_remaining = Some(time);
                break;
            }
        }

        Ok((percentage, status, time_remaining))
    }

    fn extract_percentage(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find(char::is_numeric) {
            if let Some(end) = line[start..].find('%') {
                return Some(line[start..start + end + 1].to_string());
            }
        }
        None
    }

    fn extract_time_remaining(&self, line: &str, full_info: &str) -> Option<String> {
        // Look for time patterns like "1:23" or "0:45"
        if let Some(time_match) = line.find(|c: char| c.is_ascii_digit()) {
            let remainder = &line[time_match..];
            if let Some(colon_pos) = remainder.find(':') {
                if colon_pos < 3 && remainder.len() > colon_pos + 2 {
                    let time_str = &remainder[..colon_pos + 3];
                    if full_info.contains("discharging") {
                        return Some(format!("Time Remaining: {}", time_str));
                    } else if full_info.contains("charging") {
                        return Some(format!("Time to Full: {}", time_str));
                    }
                }
            }
        }
        None
    }

    fn parse_and_display_health(&self, health_info: &str) -> Result<()> {
        for line in health_info.lines() {
            if line.contains("Cycle Count") {
                if let Some(cycles_str) = line.split(':').nth(1) {
                    let cycles_str = cycles_str.trim();
                    if let Ok(cycles) = cycles_str.parse::<u32>() {
                        println!("Cycle Count: {}", cycles);
                        match cycles {
                            0..=500 => println!("{} Low cycle count - battery in good shape", "✅".green()),
                            501..=1000 => println!("{} Moderate cycle count", "🔶".yellow()),
                            _ => println!("{} High cycle count - battery may need replacement", "⚠️".red()),
                        }
                    }
                }
            } else if line.contains("Condition") {
                if let Some(condition) = line.split(':').nth(1) {
                    let condition = condition.trim();
                    println!("Condition: {}", condition);
                    if condition.to_lowercase().contains("normal") {
                        println!("{} Battery condition is normal", "✅".green());
                    } else {
                        println!("{} Battery condition: {}", "⚠️".yellow(), condition);
                    }
                }
            } else if line.contains("Maximum Capacity") || line.contains("Full Charge Capacity") {
                println!("{}", line.trim());
            }
        }
        Ok(())
    }
}

pub async fn execute_battery_command(
    detailed: bool,
    raw: bool,
    host: Option<String>,
    config: &Config,
    execution_mode: ExecutionMode,
    verbose: bool,
) -> Result<()> {
    let monitor = BatteryMonitor::new(execution_mode, config.clone());
    monitor.execute(detailed, raw, host, verbose).await
}

pub fn show_help() {
    println!("Usage: plan10 monitor battery [options]");
    println!();
    println!("Options:");
    println!("  -d, --detailed    Show detailed battery health info");
    println!("  -r, --raw         Show raw battery data");
    println!("  -H, --host <HOST> Target server (remote monitoring)");
    println!("  -v, --verbose     Verbose output");
    println!("  -h, --help        Show this help message");
    println!();
    println!("Examples:");
    println!("  plan10 monitor battery                    # Basic battery status");
    println!("  plan10 monitor battery --detailed         # Detailed health info");
    println!("  plan10 monitor battery --host myserver    # Remote battery status");
}