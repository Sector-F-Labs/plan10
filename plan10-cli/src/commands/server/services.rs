use anyhow::Result;
use crate::Config;
use crate::commands::utils::*;
use colored::*;
use std::process::Command;

pub async fn start_services(service: Option<String>, config: &Config, verbose: bool) -> Result<()> {
    print_header("Starting Plan 10 Services");
    
    match service {
        Some(name) => start_specific_service(&name, verbose).await,
        None => start_all_services(config, verbose).await,
    }
}

pub async fn stop_services(service: Option<String>, config: &Config, verbose: bool) -> Result<()> {
    print_header("Stopping Plan 10 Services");
    
    match service {
        Some(name) => stop_specific_service(&name, verbose).await,
        None => stop_all_services(config, verbose).await,
    }
}

pub async fn restart_services(service: Option<String>, config: &Config, verbose: bool) -> Result<()> {
    print_header("Restarting Plan 10 Services");
    
    match service {
        Some(name) => {
            stop_specific_service(&name, verbose).await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            start_specific_service(&name, verbose).await
        },
        None => {
            stop_all_services(config, verbose).await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            start_all_services(config, verbose).await
        }
    }
}

pub async fn show_services(detailed: bool, config: &Config, verbose: bool) -> Result<()> {
    print_header("Plan 10 Services Status");
    
    let services = &config.server.services;
    
    for service_name in services {
        let running = super::is_service_running(service_name)?;
        let status_icon = if running { "🟢" } else { "🔴" };
        
        println!("{} {}: {}", status_icon, service_name, 
                 if running { "Running".green() } else { "Stopped".red() });
        
        if detailed && running {
            if let Ok(Some(pid)) = super::get_service_pid(service_name) {
                println!("  PID: {}", pid);
            }
        }
    }
    
    // Check LaunchAgents
    println!("\n{}:", "LaunchAgents".bold());
    let launch_agents = vec![
        "com.plan10.caffeinate",
        "caffeinate",
    ];
    
    for agent in launch_agents {
        let loaded = super::is_launchagent_loaded(agent)?;
        let status_icon = if loaded { "🟢" } else { "🔴" };
        println!("{} {}: {}", status_icon, agent,
                 if loaded { "Loaded".green() } else { "Not loaded".red() });
    }
    
    Ok(())
}

async fn start_all_services(config: &Config, verbose: bool) -> Result<()> {
    for service in &config.server.services {
        start_specific_service(service, verbose).await?;
    }
    Ok(())
}

async fn stop_all_services(config: &Config, verbose: bool) -> Result<()> {
    for service in &config.server.services {
        stop_specific_service(service, verbose).await?;
    }
    Ok(())
}

async fn start_specific_service(service: &str, verbose: bool) -> Result<()> {
    print_verbose(&format!("Starting service: {}", service), verbose);
    
    match service {
        "caffeinate" => {
            if !super::is_service_running("caffeinate")? {
                let _result = Command::new("caffeinate")
                    .args(&["-imsud"])
                    .spawn()?;
                print_success("Caffeinate started");
            } else {
                print_info("Caffeinate already running");
            }
        },
        _ => {
            print_warning(&format!("Unknown service: {}", service));
        }
    }
    
    Ok(())
}

async fn stop_specific_service(service: &str, verbose: bool) -> Result<()> {
    print_verbose(&format!("Stopping service: {}", service), verbose);
    
    match service {
        "caffeinate" => {
            let _result = Command::new("pkill")
                .arg("caffeinate")
                .output()?;
            print_success("Caffeinate stopped");
        },
        _ => {
            print_warning(&format!("Unknown service: {}", service));
        }
    }
    
    Ok(())
}