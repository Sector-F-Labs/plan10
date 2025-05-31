use clap::{Parser, Subcommand};
use anyhow::Result;
use colored::*;
use std::env;

mod commands;
mod config;
mod ssh;
mod utils;

use commands::{client, server, shared};
use config::Config;

#[derive(Parser)]
#[command(
    name = "plan10",
    about = "Plan 10 - MacBook Server Management CLI",
    long_about = "Transform your MacBook into a dedicated headless server.\nWorks as both client (for remote management) and server (for local operations).",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration file path
    #[arg(short, long, global = true, env = "PLAN10_CONFIG")]
    config: Option<String>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Force server mode (run commands locally)
    #[arg(long, global = true)]
    server_mode: bool,

    /// Force client mode (run commands remotely)
    #[arg(long, global = true)]
    client_mode: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Client commands for managing remote servers
    #[command(subcommand)]
    Client(ClientCommands),

    /// Server commands for local system management
    #[command(subcommand)]
    Server(ServerCommands),

    /// System monitoring (works locally or remotely)
    #[command(subcommand)]
    Monitor(MonitorCommands),

    /// Quick status check
    Status {
        /// Target server (if not specified, runs locally)
        #[arg(short, long)]
        host: Option<String>,
        /// Show detailed status
        #[arg(short, long)]
        detailed: bool,
    },

    /// Interactive setup wizard
    Setup {
        /// Setup mode: client, server, or both
        #[arg(value_enum, default_value = "auto")]
        mode: SetupMode,
    },

    /// Show configuration
    Config {
        /// Show configuration for specific server
        #[arg(short, long)]
        server: Option<String>,
        /// Edit configuration
        #[arg(short, long)]
        edit: bool,
    },
}

#[derive(Subcommand)]
enum ClientCommands {
    /// Deploy Plan 10 to a server
    Deploy {
        /// Target server hostname or IP
        #[arg(short = 'H', long)]
        host: String,
        /// SSH user
        #[arg(short, long)]
        user: Option<String>,
        /// SSH port
        #[arg(short, long, default_value = "22")]
        port: u16,
        /// Deploy everything (scripts, configs, services)
        #[arg(short, long)]
        all: bool,
        /// Deploy only scripts
        #[arg(long)]
        scripts_only: bool,
        /// Deploy only configuration
        #[arg(long)]
        config_only: bool,
    },

    /// Manage remote servers
    Manage {
        /// Target server
        #[arg(short = 'H', long)]
        host: String,
        #[command(subcommand)]
        action: ManageActions,
    },

    /// Run diagnostics on remote server
    Diagnose {
        /// Target server
        #[arg(short = 'H', long)]
        host: String,
        /// Focus on battery diagnostics
        #[arg(short, long)]
        battery: bool,
        /// Focus on power management
        #[arg(short, long)]
        power: bool,
        /// Show recommended fixes
        #[arg(short, long)]
        fixes: bool,
    },

    /// List configured servers
    List {
        /// Show detailed server information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Add a new server configuration
    Add {
        /// Server name
        name: String,
        /// Server hostname or IP
        #[arg(short = 'H', long)]
        host: String,
        /// SSH user
        #[arg(short, long)]
        user: String,
        /// SSH port
        #[arg(short, long, default_value = "22")]
        port: u16,
    },

    /// Remove server configuration
    Remove {
        /// Server name
        name: String,
    },
}

#[derive(Subcommand)]
enum ServerCommands {
    /// Configure this machine as a Plan 10 server
    Configure {
        /// Skip interactive prompts
        #[arg(short, long)]
        yes: bool,
        /// Setup power management
        #[arg(long)]
        power: bool,
        /// Setup monitoring
        #[arg(long)]
        monitoring: bool,
        /// Setup services
        #[arg(long)]
        services: bool,
    },

    /// Start Plan 10 services
    Start {
        /// Start specific service
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Stop Plan 10 services
    Stop {
        /// Stop specific service
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Restart Plan 10 services
    Restart {
        /// Restart specific service
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Show service status
    Services {
        /// Show detailed service information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Power management operations
    Power {
        #[command(subcommand)]
        action: PowerActions,
    },

    /// System maintenance
    Maintenance {
        #[command(subcommand)]
        action: MaintenanceActions,
    },
}

#[derive(Subcommand)]
enum MonitorCommands {
    /// Show temperature status
    Temp {
        /// Show raw temperature data
        #[arg(short, long)]
        raw: bool,
        /// Target server (remote monitoring)
        #[arg(short = 'H', long)]
        host: Option<String>,
    },

    /// Show battery status
    Battery {
        /// Show detailed battery health
        #[arg(short, long)]
        detailed: bool,
        /// Show raw battery data
        #[arg(short, long)]
        raw: bool,
        /// Target server (remote monitoring)
        #[arg(short = 'H', long)]
        host: Option<String>,
    },

    /// Power diagnostics
    Power {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Focus on battery issues
        #[arg(short, long)]
        battery: bool,
        /// Focus on sleep/wake issues
        #[arg(short, long)]
        sleep: bool,
        /// Show all diagnostics
        #[arg(short, long)]
        all: bool,
        /// Show recommended fixes
        #[arg(short, long)]
        fixes: bool,
        /// Target server (remote monitoring)
        #[arg(short = 'H', long)]
        host: Option<String>,
    },

    /// System overview
    System {
        /// Target server (remote monitoring)
        #[arg(short = 'H', long)]
        host: Option<String>,
    },

    /// Continuous monitoring
    Watch {
        /// Update interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,
        /// What to monitor
        #[arg(value_enum, default_value = "all")]
        monitor: WatchType,
        /// Target server (remote monitoring)
        #[arg(short = 'H', long)]
        host: Option<String>,
    },
}

#[derive(Subcommand)]
enum ManageActions {
    /// Start services on remote server
    Start,
    /// Stop services on remote server
    Stop,
    /// Restart services on remote server
    Restart,
    /// Update Plan 10 on remote server
    Update,
    /// Show remote server status
    Status,
    /// Configure remote server
    Configure,
}

#[derive(Subcommand)]
enum PowerActions {
    /// Show current power status
    Status,
    /// Configure power management settings
    Configure {
        /// Disable hibernation
        #[arg(long)]
        no_hibernate: bool,
        /// Disable sleep
        #[arg(long)]
        no_sleep: bool,
        /// Set battery halt level
        #[arg(long)]
        halt_level: Option<u8>,
    },
    /// Reset power settings to defaults
    Reset,
    /// Show power management diagnostics
    Diagnostics,
}

#[derive(Subcommand)]
enum MaintenanceActions {
    /// Update system packages
    Update,
    /// Clean temporary files
    Clean,
    /// Backup configuration
    Backup {
        /// Backup destination
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Restore configuration
    Restore {
        /// Backup file to restore
        input: String,
    },
    /// Health check
    Health,
}

#[derive(clap::ValueEnum, Clone)]
enum SetupMode {
    Auto,
    Client,
    Server,
    Both,
}

#[derive(clap::ValueEnum, Clone)]
enum WatchType {
    All,
    Temp,
    Battery,
    Power,
    System,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        env::set_var("RUST_LOG", "debug");
    }
    
    // Load configuration
    let config = Config::load(cli.config.as_deref())?;
    
    // Determine execution mode
    let execution_mode = determine_execution_mode(&cli);
    
    if cli.verbose {
        eprintln!("{} Running in {:?} mode", "INFO".blue(), execution_mode);
    }
    
    // Execute command
    match cli.command {
        Commands::Client(cmd) => {
            client::execute(cmd, &config, cli.verbose).await
        }
        Commands::Server(cmd) => {
            server::execute(cmd, &config, cli.verbose).await
        }
        Commands::Monitor(cmd) => {
            shared::monitor::execute(cmd, &config, execution_mode, cli.verbose).await
        }
        Commands::Status { host, detailed } => {
            shared::status::execute(host, detailed, &config, execution_mode, cli.verbose).await
        }
        Commands::Setup { mode } => {
            shared::setup::execute(mode, &config, cli.verbose).await
        }
        Commands::Config { server, edit } => {
            shared::config_cmd::execute(server, edit, &config, cli.verbose).await
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Local,
    Remote { host: String },
    Auto,
}

fn determine_execution_mode(cli: &Cli) -> ExecutionMode {
    if cli.server_mode {
        ExecutionMode::Local
    } else if cli.client_mode {
        ExecutionMode::Auto
    } else {
        // Auto-detect based on environment or command context
        ExecutionMode::Auto
    }
}