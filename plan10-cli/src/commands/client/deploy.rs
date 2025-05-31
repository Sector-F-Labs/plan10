use anyhow::Result;
use crate::Config;
use crate::commands::utils::*;
use crate::ssh::{SshClient, deploy_files};
use crate::config::ServerDefinition;
use colored::*;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn execute_deploy(
    host: String,
    user: Option<String>,
    port: u16,
    all: bool,
    scripts_only: bool,
    config_only: bool,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    print_header(&format!("Deploying Plan 10 to {}", host));

    // Resolve server configuration
    let server = resolve_or_create_server(&host, user, port, config)?;
    
    print_verbose(&format!("Connecting to {}@{}:{}", server.user, server.host, server.port), verbose);

    // Test connectivity first
    print_info("Testing connection...");
    let client = SshClient::connect(&server, config).await?;
    client.test_connection()?;
    print_success("Connection established");

    // Determine what to deploy
    let deployment_items = determine_deployment_items(all, scripts_only, config_only)?;
    
    if deployment_items.is_empty() {
        print_warning("No deployment items specified. Use --all, --scripts-only, or --config-only");
        return Ok(());
    }

    // Create progress bar
    let pb = ProgressBar::new(deployment_items.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );

    // Deploy items
    for (category, files) in deployment_items {
        pb.set_message(format!("Deploying {}", category));
        
        match category.as_str() {
            "server-setup" => deploy_server_setup(&client, verbose).await?,
            "scripts" => deploy_scripts(&client, &files, verbose).await?,
            "configs" => deploy_configs(&client, &files, verbose).await?,
            "services" => deploy_services(&client, &files, verbose).await?,
            _ => continue,
        }
        
        pb.inc(1);
    }

    pb.finish_with_message("Deployment complete");
    
    print_success("Plan 10 deployed successfully!");
    print_info("Next steps:");
    println!("  1. SSH to your server: ssh {}@{}", server.user, server.host);
    println!("  2. Run server setup: sudo ./server_setup.sh");
    println!("  3. Verify deployment: plan10 monitor system --host {}", server.host);

    Ok(())
}

fn resolve_or_create_server(
    host: &str,
    user: Option<String>,
    port: u16,
    config: &Config,
) -> Result<ServerDefinition> {
    // Try to find existing server
    if let Some(server) = config.resolve_server(host) {
        return Ok(server.clone());
    }

    // Create temporary server definition
    let user = user.ok_or_else(|| {
        anyhow::anyhow!("User not specified and server '{}' not found in config", host)
    })?;

    Ok(ServerDefinition {
        name: host.to_string(),
        host: host.to_string(),
        user,
        port,
        ssh_key: None,
        tags: vec!["temporary".to_string()],
        enabled: true,
        last_seen: None,
    })
}

fn determine_deployment_items(
    all: bool,
    scripts_only: bool,
    config_only: bool,
) -> Result<Vec<(String, Vec<(PathBuf, String)>)>> {
    let mut items = Vec::new();

    if all || (!scripts_only && !config_only) {
        // Deploy everything
        items.push(("server-setup".to_string(), vec![
            (PathBuf::from("server_setup.sh"), "~/server_setup.sh".to_string()),
        ]));
        
        items.push(("scripts".to_string(), vec![
            (PathBuf::from("scripts/temp"), "~/scripts/temp".to_string()),
            (PathBuf::from("scripts/battery"), "~/scripts/battery".to_string()),
            (PathBuf::from("scripts/power_diagnostics"), "~/scripts/power_diagnostics".to_string()),
            (PathBuf::from("scripts/setup_aliases.sh"), "~/scripts/setup_aliases.sh".to_string()),
        ]));
        
        items.push(("configs".to_string(), vec![
            (PathBuf::from("caffeinate.plist"), "~/Library/LaunchAgents/caffeinate.plist".to_string()),
        ]));
        
        items.push(("services".to_string(), vec![
            (PathBuf::from("docs/"), "~/docs/".to_string()),
        ]));
    } else if scripts_only {
        items.push(("scripts".to_string(), vec![
            (PathBuf::from("scripts/temp"), "~/scripts/temp".to_string()),
            (PathBuf::from("scripts/battery"), "~/scripts/battery".to_string()),
            (PathBuf::from("scripts/power_diagnostics"), "~/scripts/power_diagnostics".to_string()),
            (PathBuf::from("scripts/setup_aliases.sh"), "~/scripts/setup_aliases.sh".to_string()),
        ]));
    } else if config_only {
        items.push(("configs".to_string(), vec![
            (PathBuf::from("caffeinate.plist"), "~/Library/LaunchAgents/caffeinate.plist".to_string()),
            (PathBuf::from("server_setup.sh"), "~/server_setup.sh".to_string()),
        ]));
    }

    Ok(items)
}

async fn deploy_server_setup(client: &SshClient, verbose: bool) -> Result<()> {
    print_verbose("Deploying server setup script", verbose);
    
    let local_path = PathBuf::from("server_setup.sh");
    if !local_path.exists() {
        anyhow::bail!("server_setup.sh not found in current directory");
    }
    
    client.copy_file(&local_path, "~/server_setup.sh")?;
    client.execute_command("chmod +x ~/server_setup.sh")?;
    
    print_verbose("Server setup script deployed and made executable", verbose);
    Ok(())
}

async fn deploy_scripts(client: &SshClient, files: &[(PathBuf, String)], verbose: bool) -> Result<()> {
    print_verbose("Deploying monitoring scripts", verbose);
    
    // Ensure scripts directory exists
    client.ensure_directory("~/scripts")?;
    
    for (local_path, remote_path) in files {
        if !local_path.exists() {
            print_warning(&format!("Local file not found: {}", local_path.display()));
            continue;
        }
        
        client.copy_file(local_path, remote_path)?;
        
        // Make scripts executable
        if local_path.file_name().unwrap().to_str().unwrap() != "setup_aliases.sh" {
            let chmod_cmd = format!("chmod +x {}", remote_path);
            client.execute_command(&chmod_cmd)?;
        }
        
        print_verbose(&format!("Deployed: {}", local_path.display()), verbose);
    }
    
    Ok(())
}

async fn deploy_configs(client: &SshClient, files: &[(PathBuf, String)], verbose: bool) -> Result<()> {
    print_verbose("Deploying configuration files", verbose);
    
    for (local_path, remote_path) in files {
        if !local_path.exists() {
            print_warning(&format!("Local file not found: {}", local_path.display()));
            continue;
        }
        
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(remote_path).parent() {
            let mkdir_cmd = format!("mkdir -p {}", parent.display());
            client.execute_command(&mkdir_cmd)?;
        }
        
        client.copy_file(local_path, remote_path)?;
        print_verbose(&format!("Deployed: {}", local_path.display()), verbose);
    }
    
    Ok(())
}

async fn deploy_services(client: &SshClient, files: &[(PathBuf, String)], verbose: bool) -> Result<()> {
    print_verbose("Deploying service files", verbose);
    
    for (local_path, remote_path) in files {
        if !local_path.exists() {
            print_warning(&format!("Local path not found: {}", local_path.display()));
            continue;
        }
        
        if local_path.is_dir() {
            client.copy_directory(local_path, remote_path)?;
        } else {
            client.copy_file(local_path, remote_path)?;
        }
        
        print_verbose(&format!("Deployed: {}", local_path.display()), verbose);
    }
    
    Ok(())
}

pub async fn verify_deployment(
    server: &ServerDefinition,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    print_header("Verifying Deployment");
    
    let client = SshClient::connect(server, config).await?;
    
    // Check if key files exist
    let files_to_check = vec![
        "~/server_setup.sh",
        "~/scripts/temp",
        "~/scripts/battery",
        "~/scripts/power_diagnostics",
    ];
    
    for file in files_to_check {
        if client.file_exists(file)? {
            print_success(&format!("{} exists", file));
        } else {
            print_error(&format!("{} missing", file));
        }
    }
    
    // Test script execution
    let temp_result = client.execute_command("~/scripts/temp --help");
    match temp_result {
        Ok(result) if result.success => print_success("Scripts are executable"),
        _ => print_warning("Scripts may not be properly configured"),
    }
    
    Ok(())
}