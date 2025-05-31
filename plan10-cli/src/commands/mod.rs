pub mod client;
pub mod server;
pub mod shared;

pub use client::execute as execute_client;
pub use server::execute as execute_server;

// Re-export shared command modules for easier access
pub use shared::monitor;
pub use shared::status;
pub use shared::setup;
pub use shared::config_cmd;

use crate::{ExecutionMode, Config};
use anyhow::Result;

pub trait CommandExecutor {
    type Args;
    
    async fn execute(
        args: Self::Args,
        config: &Config,
        execution_mode: ExecutionMode,
        verbose: bool,
    ) -> Result<()>;
}

pub trait LocalCommand {
    type Args;
    
    async fn execute_local(
        args: Self::Args,
        config: &Config,
        verbose: bool,
    ) -> Result<()>;
}

pub trait RemoteCommand {
    type Args;
    
    async fn execute_remote(
        args: Self::Args,
        server: &str,
        config: &Config,
        verbose: bool,
    ) -> Result<()>;
}

// Common utilities for all commands
pub mod utils {
    use colored::*;
    
    pub fn print_header(title: &str) {
        println!("{}", format!("🔧 {}", title).bold().blue());
        println!("{}", "=".repeat(title.len() + 3).dimmed());
    }
    
    pub fn print_success(message: &str) {
        println!("{} {}", "✅".green(), message);
    }
    
    pub fn print_warning(message: &str) {
        println!("{} {}", "⚠️".yellow(), message);
    }
    
    pub fn print_error(message: &str) {
        println!("{} {}", "❌".red(), message);
    }
    
    pub fn print_info(message: &str) {
        println!("{} {}", "ℹ️".blue(), message);
    }
    
    pub fn print_verbose(message: &str, verbose: bool) {
        if verbose {
            println!("{} {}", "🔍".dimmed(), message.dimmed());
        }
    }
}