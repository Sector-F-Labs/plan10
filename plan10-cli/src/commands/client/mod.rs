use anyhow::Result;
use crate::{ClientCommands, ManageActions, Config};
use crate::commands::utils::*;
use crate::ssh::{SshClient, deploy_files, test_connectivity};
use colored::*;
use std::path::PathBuf;

pub mod deploy;
pub mod manage;
pub mod diagnostics;
pub mod servers;

pub async fn execute(cmd: ClientCommands, config: &Config, verbose: bool) -> Result<()> {
    match cmd {
        ClientCommands::Deploy { 
            host, 
            user, 
            port, 
            all, 
            scripts_only, 
            config_only 
        } => {
            deploy::execute_deploy(host, user, port, all, scripts_only, config_only, config, verbose).await
        }
        ClientCommands::Manage { host, action } => {
            manage::execute_manage(host, action, config, verbose).await
        }
        ClientCommands::Diagnose { 
            host, 
            battery, 
            power, 
            fixes 
        } => {
            diagnostics::execute_diagnose(host, battery, power, fixes, config, verbose).await
        }
        ClientCommands::List { detailed } => {
            servers::list_servers(config, detailed, verbose).await
        }
        ClientCommands::Add { 
            name, 
            host, 
            user, 
            port 
        } => {
            servers::add_server(name, host, user, port, config, verbose).await
        }
        ClientCommands::Remove { name } => {
            servers::remove_server(name, config, verbose).await
        }
    }
}

pub async fn test_server_connectivity(host: &str, config: &Config, verbose: bool) -> Result<bool> {
    print_verbose(&format!("Testing connectivity to {}", host), verbose);
    
    if let Some(server) = config.resolve_server(host) {
        test_connectivity(server, config).await
    } else {
        print_error(&format!("Server '{}' not found in configuration", host));
        Ok(false)
    }
}

pub fn get_deployment_files() -> Vec<(PathBuf, String)> {
    vec![
        (PathBuf::from("server_setup.sh"), "~/server_setup.sh".to_string()),
        (PathBuf::from("caffeinate.plist"), "~/Library/LaunchAgents/caffeinate.plist".to_string()),
        (PathBuf::from("scripts/"), "~/scripts/".to_string()),
        (PathBuf::from("docs/"), "~/docs/".to_string()),
    ]
}