use anyhow::Result;
use crate::{ManageActions, Config};
use crate::commands::utils::*;
use crate::ssh::SshClient;
use colored::*;

pub async fn execute_manage(
    host: String,
    action: ManageActions,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    let server = config.resolve_server(&host)
        .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", host))?;

    print_header(&format!("Managing Server: {}", host));
    print_verbose(&format!("Connecting to {}@{}:{}", server.user, server.host, server.port), verbose);

    let client = SshClient::connect(server, config).await?;

    match action {
        ManageActions::Start => {
            print_info("Starting Plan 10 services...");
            let result = client.execute_command("launchctl load ~/Library/LaunchAgents/caffeinate.plist")?;
            if result.success {
                print_success("Services started successfully");
            } else {
                print_error(&format!("Failed to start services: {}", result.stderr));
            }
        }
        ManageActions::Stop => {
            print_info("Stopping Plan 10 services...");
            let result = client.execute_command("launchctl unload ~/Library/LaunchAgents/caffeinate.plist; pkill caffeinate")?;
            if result.success {
                print_success("Services stopped successfully");
            } else {
                print_warning("Some services may still be running");
            }
        }
        ManageActions::Restart => {
            print_info("Restarting Plan 10 services...");
            let _ = client.execute_command("launchctl unload ~/Library/LaunchAgents/caffeinate.plist; pkill caffeinate");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let result = client.execute_command("launchctl load ~/Library/LaunchAgents/caffeinate.plist")?;
            if result.success {
                print_success("Services restarted successfully");
            } else {
                print_error(&format!("Failed to restart services: {}", result.stderr));
            }
        }
        ManageActions::Update => {
            print_info("Updating Plan 10 installation...");
            // Re-deploy the latest files
            crate::commands::client::deploy::execute_deploy(
                host.clone(), None, 22, true, false, false, config, verbose
            ).await?;
            print_success("Plan 10 updated successfully");
        }
        ManageActions::Status => {
            print_info("Checking server status...");
            
            // Check caffeinate status
            let caffeinate_result = client.execute_command("pgrep -x caffeinate")?;
            if caffeinate_result.success && !caffeinate_result.stdout.trim().is_empty() {
                print_success(&format!("Caffeinate running (PID: {})", caffeinate_result.stdout.trim()));
            } else {
                print_warning("Caffeinate not running");
            }
            
            // Check power source
            let power_result = client.execute_command("pmset -g batt | head -1")?;
            if power_result.success {
                let power_info = power_result.stdout.trim();
                if power_info.contains("AC Power") {
                    print_success("Power source: AC Power");
                } else if power_info.contains("Battery Power") {
                    print_warning("Power source: Battery Power");
                } else {
                    print_info(&format!("Power source: {}", power_info));
                }
            }
            
            // Check system uptime
            let uptime_result = client.execute_command("uptime")?;
            if uptime_result.success {
                print_info(&format!("Uptime: {}", uptime_result.stdout.trim()));
            }
        }
        ManageActions::Configure => {
            print_info("Running server configuration...");
            let result = client.execute_command("sudo ./server_setup.sh")?;
            if result.success {
                print_success("Server configuration completed");
                println!("{}", result.stdout);
            } else {
                print_error(&format!("Configuration failed: {}", result.stderr));
            }
        }
    }

    Ok(())
}