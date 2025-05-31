use anyhow::Result;
use crate::Config;
use crate::commands::utils::*;
use crate::ssh::SshClient;


pub async fn execute_diagnose(
    host: String,
    battery: bool,
    power: bool,
    fixes: bool,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    let server = config.resolve_server(&host)
        .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", host))?;

    print_header(&format!("Diagnostics for: {}", host));
    print_verbose(&format!("Connecting to {}@{}:{}", server.user, server.host, server.port), verbose);

    let client = SshClient::connect(server, config).await?;

    // Test basic connectivity
    match client.test_connection() {
        Ok(_) => print_success("Connection test passed"),
        Err(e) => {
            print_error(&format!("Connection test failed: {}", e));
            return Ok(());
        }
    }

    // Check if Plan 10 scripts are available
    let scripts_available = check_scripts_availability(&client).await?;
    if !scripts_available {
        print_warning("Plan 10 scripts not found. Consider running deployment first.");
        return Ok(());
    }

    // Run appropriate diagnostics based on flags
    if battery {
        run_battery_diagnostics(&client, verbose).await?;
    } else if power {
        run_power_diagnostics(&client, verbose).await?;
    } else if fixes {
        run_comprehensive_diagnostics_with_fixes(&client, verbose).await?;
    } else {
        run_basic_diagnostics(&client, verbose).await?;
    }

    Ok(())
}

async fn check_scripts_availability(client: &SshClient) -> Result<bool> {
    let scripts = vec![
        "~/scripts/temp",
        "~/scripts/battery", 
        "~/scripts/power_diagnostics"
    ];

    let mut all_available = true;
    for script in scripts {
        if !client.file_exists(script)? {
            print_warning(&format!("Missing script: {}", script));
            all_available = false;
        }
    }

    Ok(all_available)
}

async fn run_basic_diagnostics(client: &SshClient, _verbose: bool) -> Result<()> {
    print_info("Running basic diagnostics...");

    // System overview
    let result = client.execute_command("uname -a && uptime")?;
    if result.success {
        println!("\n📊 System Information:");
        println!("{}", result.stdout);
    }

    // Power status
    let power_result = client.execute_command("pmset -g batt | head -1")?;
    if power_result.success {
        println!("\n⚡ Power Status:");
        println!("{}", power_result.stdout.trim());
    }

    // Caffeinate status
    let caffeinate_result = client.execute_command("pgrep -x caffeinate")?;
    println!("\n☕ Caffeinate Status:");
    if caffeinate_result.success && !caffeinate_result.stdout.trim().is_empty() {
        print_success(&format!("Running (PID: {})", caffeinate_result.stdout.trim()));
    } else {
        print_warning("Not running");
    }

    Ok(())
}

async fn run_battery_diagnostics(client: &SshClient, _verbose: bool) -> Result<()> {
    print_info("Running battery-focused diagnostics...");

    let result = client.execute_command("~/scripts/battery -d")?;
    if result.success {
        println!("{}", result.stdout);
    } else {
        print_error(&format!("Battery diagnostics failed: {}", result.stderr));
    }

    Ok(())
}

async fn run_power_diagnostics(client: &SshClient, _verbose: bool) -> Result<()> {
    print_info("Running power management diagnostics...");

    let result = client.execute_command("~/scripts/power_diagnostics")?;
    if result.success {
        println!("{}", result.stdout);
    } else {
        print_error(&format!("Power diagnostics failed: {}", result.stderr));
    }

    Ok(())
}

async fn run_comprehensive_diagnostics_with_fixes(client: &SshClient, _verbose: bool) -> Result<()> {
    print_info("Running comprehensive diagnostics with recommended fixes...");

    let result = client.execute_command("~/scripts/power_diagnostics -f")?;
    if result.success {
        println!("{}", result.stdout);
        
        // Additional checks
        println!("\n🔍 Additional Checks:");
        
        // Check for common issues
        let df_result = client.execute_command("df -h / | tail -1")?;
        if df_result.success {
            println!("📁 Disk usage: {}", df_result.stdout.trim());
        }

        let memory_result = client.execute_command("vm_stat | head -5")?;
        if memory_result.success {
            println!("💾 Memory info:");
            println!("{}", memory_result.stdout);
        }

        println!("\n💡 Deployment Verification:");
        let files_to_check = vec![
            ("Server setup", "~/server_setup.sh"),
            ("Caffeinate plist", "~/Library/LaunchAgents/caffeinate.plist"),
            ("Temp script", "~/scripts/temp"),
            ("Battery script", "~/scripts/battery"),
            ("Power diagnostics", "~/scripts/power_diagnostics"),
        ];

        for (name, path) in files_to_check {
            if client.file_exists(path)? {
                print_success(&format!("{}: Present", name));
            } else {
                print_warning(&format!("{}: Missing", name));
            }
        }

    } else {
        print_error(&format!("Comprehensive diagnostics failed: {}", result.stderr));
    }

    Ok(())
}