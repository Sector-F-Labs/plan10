use anyhow::Result;
use crate::Config;
use crate::commands::utils::*;
use crate::ssh::SshClient;
use crate::ExecutionMode;
use colored::*;
use std::process::Command;
use std::collections::HashMap;

pub struct PowerDiagnostics {
    execution_mode: ExecutionMode,
    config: Config,
}

impl PowerDiagnostics {
    pub fn new(execution_mode: ExecutionMode, config: Config) -> Self {
        Self {
            execution_mode,
            config,
        }
    }

    pub async fn execute(
        &self,
        verbose: bool,
        battery: bool,
        sleep: bool,
        all: bool,
        fixes: bool,
        host: Option<String>,
        _verbose_flag: bool,
    ) -> Result<()> {
        match &self.execution_mode {
            ExecutionMode::Local => {
                self.execute_local(verbose, battery, sleep, all, fixes).await
            }
            ExecutionMode::Remote { host: default_host } => {
                let target_host = host.unwrap_or_else(|| default_host.clone());
                self.execute_remote(&target_host, verbose, battery, sleep, all, fixes).await
            }
            ExecutionMode::Auto => {
                if let Some(target_host) = host {
                    self.execute_remote(&target_host, verbose, battery, sleep, all, fixes).await
                } else {
                    self.execute_local(verbose, battery, sleep, all, fixes).await
                }
            }
        }
    }

    async fn execute_local(&self, verbose: bool, battery: bool, sleep: bool, all: bool, fixes: bool) -> Result<()> {
        if all {
            self.show_all_diagnostics().await
        } else if fixes {
            self.show_basic_status().await?;
            self.analyze_power_issues().await?;
            self.show_recommended_fixes().await
        } else if battery {
            self.show_basic_status().await?;
            self.show_battery_diagnostics().await
        } else if sleep {
            self.show_basic_status().await?;
            self.show_sleep_diagnostics().await
        } else if verbose {
            self.show_basic_status().await?;
            self.show_verbose_info().await
        } else {
            self.show_basic_status().await?;
            self.analyze_power_issues().await
        }
    }

    async fn execute_remote(&self, host: &str, verbose: bool, battery: bool, sleep: bool, all: bool, fixes: bool) -> Result<()> {
        let server = self.config.resolve_server(host)
            .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", host))?;

        let client = SshClient::connect(server, &self.config).await?;
        
        let mut args = Vec::new();
        if verbose { args.push("-v"); }
        if battery { args.push("-b"); }
        if sleep { args.push("-s"); }
        if all { args.push("-a"); }
        if fixes { args.push("-f"); }

        let command = if args.is_empty() {
            "~/scripts/power_diagnostics".to_string()
        } else {
            format!("~/scripts/power_diagnostics {}", args.join(" "))
        };

        let result = client.execute_command(&command)?;
        
        if result.success {
            println!("{}", result.stdout);
        } else {
            print_error(&format!("Remote command failed: {}", result.stderr));
        }

        Ok(())
    }

    async fn show_basic_status(&self) -> Result<()> {
        println!("{} Basic Power Status", "⚡".yellow());
        println!("{}", "=".repeat(20));

        // Check power source
        let battery_info = self.get_pmset_battery().await?;
        if battery_info.contains("Battery Power") {
            println!("{} Currently running on: Battery Power", "🔋".yellow());
        } else if battery_info.contains("AC Power") {
            println!("{} Currently running on: AC Power", "🔌".green());
        } else {
            println!("{} Power source: Unknown", "❓".red());
        }

        // Extract battery percentage
        if let Some(percentage) = self.extract_battery_percentage(&battery_info) {
            let pct_num = percentage.trim_end_matches('%').parse::<u8>().unwrap_or(0);
            let (icon, status) = match pct_num {
                81..=100 => ("🟢", "Good"),
                51..=80 => ("🟡", "Medium"),
                21..=50 => ("🟠", "Low"),
                _ => ("🔴", "Critical"),
            };
            println!("{} Battery Level: {}% ({})", icon, pct_num, status);
        }

        // Check caffeinate status
        if self.is_caffeinate_running().await? {
            let pid = self.get_caffeinate_pid().await?;
            println!("{} Caffeinate: ✅ Running (PID: {})", "☕".cyan(), pid);
        } else {
            println!("{} Caffeinate: ❌ Not running", "☕".cyan());
        }

        println!();
        Ok(())
    }

    async fn analyze_power_issues(&self) -> Result<()> {
        println!("{} Power Management Analysis", "🔍".blue());
        println!("{}", "=".repeat(29));

        let pmset_output = self.get_pmset_settings().await?;
        let settings = self.parse_pmset_settings(&pmset_output);
        let mut issues_found = 0;

        // Check hibernation mode
        if let Some(hibernate_mode) = settings.get("hibernatemode") {
            if hibernate_mode != "0" {
                println!("{} ISSUE: hibernatemode is {} (should be 0 for servers)", "⚠️".yellow(), hibernate_mode);
                issues_found += 1;
            } else {
                println!("{} hibernatemode: {} (good)", "✅".green(), hibernate_mode);
            }
        }

        // Check standby
        if let Some(standby) = settings.get("standby") {
            if standby == "1" {
                println!("{} ISSUE: standby is enabled (should be 0 for servers)", "⚠️".yellow());
                issues_found += 1;
            } else {
                println!("{} standby: {} (good)", "✅".green(), standby);
            }
        }

        // Check powernap
        if let Some(powernap) = settings.get("powernap") {
            if powernap == "1" {
                println!("{} ISSUE: powernap is enabled (should be 0 for servers)", "⚠️".yellow());
                issues_found += 1;
            } else {
                println!("{} powernap: {} (good)", "✅".green(), powernap);
            }
        }

        // Check sleep settings
        if let Some(sleep) = settings.get("sleep") {
            if sleep != "0" {
                println!("{} ISSUE: sleep is enabled ({} minutes)", "⚠️".yellow(), sleep);
                issues_found += 1;
            } else {
                println!("{} sleep: {} (good)", "✅".green(), sleep);
            }
        }

        // Check disksleep
        if let Some(disksleep) = settings.get("disksleep") {
            if disksleep != "0" {
                println!("{} ISSUE: disksleep is enabled ({} minutes)", "⚠️".yellow(), disksleep);
                issues_found += 1;
            } else {
                println!("{} disksleep: {} (good)", "✅".green(), disksleep);
            }
        }

        // Check halt level
        if let Some(haltlevel) = settings.get("haltlevel") {
            if let Ok(level) = haltlevel.parse::<u8>() {
                if level > 10 {
                    println!("{} ISSUE: haltlevel is {}% (should be 5% or lower)", "⚠️".yellow(), level);
                    issues_found += 1;
                } else {
                    println!("{} haltlevel: {}% (good)", "✅".green(), level);
                }
            }
        }

        println!();
        if issues_found == 0 {
            println!("{} No power management issues found!", "🎉".green());
        } else {
            println!("{} Found {} power management issue(s) that could cause shutdowns", "❌".red(), issues_found);
            println!("   Use --fixes flag to see recommended fixes");
        }
        println!();

        Ok(())
    }

    async fn show_battery_diagnostics(&self) -> Result<()> {
        println!("{} Battery Diagnostics", "🔋".green());
        println!("{}", "=".repeat(21));

        let battery_output = self.get_pmset_battery().await?;
        println!("{}", battery_output);

        // Battery health information
        let health_output = self.get_battery_health().await?;
        if !health_output.is_empty() {
            println!();
            println!("{} Battery Health Information:", "🏥".blue());
            println!("{}", "=".repeat(30));
            println!("{}", health_output);
        }

        // Critical power settings
        println!();
        println!("{} Critical Battery Settings:", "⚠️".yellow());
        println!("{}", "=".repeat(30));
        
        let pmset_output = self.get_pmset_settings().await?;
        let settings = self.parse_pmset_settings(&pmset_output);

        let halt_level = settings.get("haltlevel").map(|s| s.as_str()).unwrap_or("Not set");
        let halt_after = settings.get("haltafter").map(|s| s.as_str()).unwrap_or("Not set");
        let autopoweroff = settings.get("autopoweroff").map(|s| s.as_str()).unwrap_or("Not set");

        println!("Halt Level: {}", halt_level);
        println!("Halt After: {}", halt_after);
        println!("Auto Power Off: {}", autopoweroff);

        if let Ok(level) = halt_level.parse::<u8>() {
            if level > 10 {
                println!("{} WARNING: Halt level is high ({}%). System may shut down early on battery.", "⚠️".yellow(), level);
            }
        }

        if autopoweroff == "1" {
            println!("{} WARNING: Auto power off is enabled. System may shut down automatically.", "⚠️".yellow());
        }

        println!();
        Ok(())
    }

    async fn show_sleep_diagnostics(&self) -> Result<()> {
        println!("{} Sleep/Wake Diagnostics", "😴".blue());
        println!("{}", "=".repeat(25));

        // Current sleep settings
        println!("Current Sleep Settings:");
        println!("{}", "=".repeat(23));
        let custom_output = self.get_pmset_custom().await?;
        println!("{}", custom_output);

        // Power assertions
        println!();
        println!("{} Power Assertions (what's keeping system awake):", "🔒".cyan());
        println!("{}", "=".repeat(52));
        let assertions_output = self.get_power_assertions().await?;
        let lines: Vec<&str> = assertions_output.lines().take(20).collect();
        println!("{}", lines.join("\n"));

        // Recent wake/sleep log
        println!();
        println!("{} Recent Sleep/Wake Events:", "📝".yellow());
        println!("{}", "=".repeat(29));
        let log_output = self.get_pmset_log().await?;
        let wake_events: Vec<&str> = log_output
            .lines()
            .filter(|line| line.contains("Sleep") || line.contains("Wake") || line.contains("DarkWake"))
            .rev()
            .take(10)
            .collect();
        for event in wake_events.iter().rev() {
            println!("{}", event);
        }

        println!();
        Ok(())
    }

    async fn show_verbose_info(&self) -> Result<()> {
        println!("{} Detailed Power Management Settings", "🔍".blue());
        println!("{}", "=".repeat(37));

        let pmset_output = self.get_pmset_settings().await?;
        println!("{}", pmset_output);

        println!();
        println!("{} System Power Information:", "⚙️".cyan());
        println!("{}", "=".repeat(29));
        let system_info = self.get_system_power_info().await?;
        println!("{}", system_info);

        println!();
        Ok(())
    }

    async fn show_recommended_fixes(&self) -> Result<()> {
        println!("{} Recommended Fixes for Power Issues", "🔧".green());
        println!("{}", "=".repeat(37));
        println!();
        println!("Based on your current settings, here are the recommended fixes:");
        println!();

        let pmset_output = self.get_pmset_settings().await?;
        let settings = self.parse_pmset_settings(&pmset_output);

        println!("{} Quick Fix Commands (run these in order):", "1️⃣".blue());
        println!("{}", "=".repeat(42));
        println!();

        // Generate specific fix commands based on current settings
        if settings.get("hibernatemode").unwrap_or(&"0".to_string()) != "0" {
            println!("# Disable hibernation (prevents unexpected shutdowns)");
            println!("sudo pmset -a hibernatemode 0");
            println!();
        }

        if settings.get("standby").unwrap_or(&"0".to_string()) == "1" {
            println!("# Disable standby mode");
            println!("sudo pmset -a standby 0");
            println!();
        }

        if settings.get("powernap").unwrap_or(&"0".to_string()) == "1" {
            println!("# Disable power nap");
            println!("sudo pmset -a powernap 0");
            println!();
        }

        if settings.get("sleep").unwrap_or(&"0".to_string()) != "0" {
            println!("# Disable system sleep completely");
            println!("sudo pmset -a sleep 0");
            println!();
        }

        if settings.get("disksleep").unwrap_or(&"0".to_string()) != "0" {
            println!("# Disable disk sleep");
            println!("sudo pmset -a disksleep 0");
            println!();
        }

        if let Some(halt_level) = settings.get("haltlevel") {
            if let Ok(level) = halt_level.parse::<u8>() {
                if level > 5 {
                    println!("# Set battery halt level to 5% (prevents early shutdown)");
                    println!("sudo pmset -b haltlevel 5");
                    println!("sudo pmset -b haltafter 0");
                    println!();
                }
            }
        }

        println!("# Disable auto power off");
        println!("sudo pmset -a autopoweroff 0");
        println!();

        println!("# Restart caffeinate if needed");
        println!("pkill caffeinate 2>/dev/null");
        println!("nohup caffeinate -imsud > /dev/null 2>&1 &");
        println!();

        println!("{} Complete Server Setup (recommended):", "2️⃣".blue());
        println!("{}", "=".repeat(40));
        println!();
        println!("# Use the Plan 10 server setup:");
        println!("sudo ./server_setup.sh");
        println!();

        println!("{} Verification Commands:", "3️⃣".blue());
        println!("{}", "=".repeat(25));
        println!();
        println!("# Check that settings were applied:");
        println!("pmset -g");
        println!();
        println!("# Verify caffeinate is running:");
        println!("pgrep caffeinate");
        println!();
        println!("# Check power assertions:");
        println!("pmset -g assertions");
        println!();

        Ok(())
    }

    async fn show_all_diagnostics(&self) -> Result<()> {
        self.show_basic_status().await?;
        self.analyze_power_issues().await?;
        self.show_battery_diagnostics().await?;
        self.show_sleep_diagnostics().await?;
        self.show_verbose_info().await?;

        println!("{} Troubleshooting Tips", "🔧".green());
        println!("{}", "=".repeat(22));
        println!("• If system shuts down on battery, check halt level: pmset -b haltlevel 5");
        println!("• If system sleeps unexpectedly, ensure caffeinate is running");
        println!("• For sleep issues, check assertions: pmset -g assertions");
        println!("• To prevent all sleep: sudo pmset -a sleep 0");
        println!("• To check what woke the system: pmset -g log");
        println!();
        println!("{} Emergency Commands", "🆘".red());
        println!("{}", "=".repeat(19));
        println!("• Kill all sleep: sudo pmset -a sleep 0 disksleep 0 standby 0");
        println!("• Restart caffeinate: pkill caffeinate && caffeinate -imsud &");
        println!("• Reset power settings: sudo pmset -a restoredefaults");
        println!();

        Ok(())
    }

    // Helper methods
    async fn get_pmset_battery(&self) -> Result<String> {
        let output = Command::new("pmset")
            .args(&["-g", "batt"])
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn get_pmset_settings(&self) -> Result<String> {
        let output = Command::new("pmset")
            .arg("-g")
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn get_pmset_custom(&self) -> Result<String> {
        let output = Command::new("pmset")
            .args(&["-g", "custom"])
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn get_power_assertions(&self) -> Result<String> {
        let output = Command::new("pmset")
            .args(&["-g", "assertions"])
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn get_pmset_log(&self) -> Result<String> {
        let output = Command::new("pmset")
            .args(&["-g", "log"])
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
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

    async fn get_system_power_info(&self) -> Result<String> {
        let output = Command::new("system_profiler")
            .arg("SPPowerDataType")
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn is_caffeinate_running(&self) -> Result<bool> {
        let output = Command::new("pgrep")
            .args(&["-x", "caffeinate"])
            .output()?;
        Ok(output.status.success() && !output.stdout.is_empty())
    }

    async fn get_caffeinate_pid(&self) -> Result<String> {
        let output = Command::new("pgrep")
            .arg("caffeinate")
            .output()?;
        let stdout_string = String::from_utf8_lossy(&output.stdout);
        Ok(stdout_string.trim().to_string())
    }

    fn extract_battery_percentage(&self, battery_info: &str) -> Option<String> {
        for line in battery_info.lines() {
            if let Some(start) = line.find(char::is_numeric) {
                if let Some(end) = line[start..].find('%') {
                    return Some(line[start..start + end + 1].to_string());
                }
            }
        }
        None
    }

    fn parse_pmset_settings(&self, output: &str) -> HashMap<String, String> {
        let mut settings = HashMap::new();
        
        for line in output.lines() {
            let trimmed = line.trim();
            if let Some(space_pos) = trimmed.find(' ') {
                let key = trimmed[..space_pos].trim();
                let value = trimmed[space_pos..].trim();
                if !key.is_empty() && !value.is_empty() {
                    settings.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        settings
    }
}

pub async fn execute_power_diagnostics_command(
    verbose: bool,
    battery: bool,
    sleep: bool,
    all: bool,
    fixes: bool,
    host: Option<String>,
    config: &Config,
    execution_mode: ExecutionMode,
    verbose_flag: bool,
) -> Result<()> {
    let diagnostics = PowerDiagnostics::new(execution_mode, config.clone());
    diagnostics.execute(verbose, battery, sleep, all, fixes, host, verbose_flag).await
}

pub fn show_help() {
    println!("Usage: plan10 monitor power [options]");
    println!();
    println!("Options:");
    println!("  -v, --verbose     Show verbose output");
    println!("  -b, --battery     Focus on battery issues");
    println!("  -s, --sleep       Focus on sleep/wake issues");
    println!("  -a, --all         Show all diagnostics");
    println!("  -f, --fixes       Show recommended fixes");
    println!("  -H, --host <HOST> Target server (remote monitoring)");
    println!("  --help            Show this help message");
    println!();
    println!("Examples:");
    println!("  plan10 monitor power                    # Basic power diagnostics");
    println!("  plan10 monitor power --battery          # Battery-focused diagnostics");
    println!("  plan10 monitor power --fixes            # Show recommended fixes");
    println!("  plan10 monitor power --host myserver    # Remote power diagnostics");
}