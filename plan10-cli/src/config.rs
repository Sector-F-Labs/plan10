use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client: ClientConfig,
    pub server: ServerConfig,
    pub servers: HashMap<String, ServerDefinition>,
    pub ssh: SshConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub default_server: Option<String>,
    pub deployment_timeout: u64,
    pub concurrent_operations: usize,
    pub auto_backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub monitoring_interval: u64,
    pub temp_threshold: f32,
    pub battery_warning_level: u8,
    pub auto_restart_services: bool,
    pub log_level: String,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDefinition {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub ssh_key: Option<String>,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub connect_timeout: u64,
    pub command_timeout: u64,
    pub key_path: Option<String>,
    pub known_hosts_file: Option<String>,
    pub compression: bool,
    pub keep_alive: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            client: ClientConfig {
                default_server: None,
                deployment_timeout: 300,
                concurrent_operations: 4,
                auto_backup: true,
            },
            server: ServerConfig {
                name: hostname::get()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                monitoring_interval: 30,
                temp_threshold: 80.0,
                battery_warning_level: 20,
                auto_restart_services: true,
                log_level: "info".to_string(),
                services: vec![
                    "caffeinate".to_string(),
                    "plan10-monitor".to_string(),
                ],
            },
            servers: HashMap::new(),
            ssh: SshConfig {
                connect_timeout: 30,
                command_timeout: 60,
                key_path: None,
                known_hosts_file: None,
                compression: true,
                keep_alive: true,
            },
        }
    }
}

impl Config {
    pub fn load(config_path: Option<&str>) -> Result<Self> {
        let path = config_path
            .map(PathBuf::from)
            .or_else(|| Self::default_config_path())
            .context("Could not determine config file path")?;

        if path.exists() {
            let content = fs::read_to_string(&path)
                .context(format!("Failed to read config file: {}", path.display()))?;
            
            let config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save(Some(&path))?;
            Ok(config)
        }
    }

    pub fn save(&self, config_path: Option<&Path>) -> Result<()> {
        let path = config_path
            .map(PathBuf::from)
            .or_else(|| Self::default_config_path())
            .context("Could not determine config file path")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&path, content)
            .context(format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    pub fn default_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("plan10").join("config.toml"))
    }

    pub fn add_server(&mut self, server: ServerDefinition) -> Result<()> {
        if self.servers.contains_key(&server.name) {
            anyhow::bail!("Server '{}' already exists", server.name);
        }
        
        self.servers.insert(server.name.clone(), server);
        Ok(())
    }

    pub fn remove_server(&mut self, name: &str) -> Result<()> {
        if !self.servers.contains_key(name) {
            anyhow::bail!("Server '{}' not found", name);
        }
        
        self.servers.remove(name);
        
        // Clear default server if it was the removed one
        if self.client.default_server.as_ref() == Some(&name.to_string()) {
            self.client.default_server = None;
        }
        
        Ok(())
    }

    pub fn get_server(&self, name: &str) -> Option<&ServerDefinition> {
        self.servers.get(name)
    }

    pub fn get_default_server(&self) -> Option<&ServerDefinition> {
        self.client.default_server
            .as_ref()
            .and_then(|name| self.servers.get(name))
    }

    pub fn list_servers(&self) -> Vec<&ServerDefinition> {
        self.servers.values().collect()
    }

    pub fn update_server_last_seen(&mut self, name: &str) -> Result<()> {
        if let Some(server) = self.servers.get_mut(name) {
            server.last_seen = Some(chrono::Utc::now());
            Ok(())
        } else {
            anyhow::bail!("Server '{}' not found", name);
        }
    }

    pub fn resolve_server(&self, name_or_host: &str) -> Option<&ServerDefinition> {
        // First try exact name match
        if let Some(server) = self.servers.get(name_or_host) {
            return Some(server);
        }

        // Then try host match
        self.servers.values().find(|server| server.host == name_or_host)
    }

    pub fn get_ssh_key_path(&self) -> Option<PathBuf> {
        self.ssh.key_path
            .as_ref()
            .map(|path| shellexpand::tilde(path).into_owned().into())
            .or_else(|| {
                dirs::home_dir().map(|home| home.join(".ssh").join("id_rsa"))
            })
    }

    pub fn get_known_hosts_path(&self) -> Option<PathBuf> {
        self.ssh.known_hosts_file
            .as_ref()
            .map(|path| shellexpand::tilde(path).into_owned().into())
            .or_else(|| {
                dirs::home_dir().map(|home| home.join(".ssh").join("known_hosts"))
            })
    }

    pub fn validate(&self) -> Result<()> {
        // Validate server configurations
        for (name, server) in &self.servers {
            if server.name != *name {
                anyhow::bail!("Server name mismatch: key '{}' vs name '{}'", name, server.name);
            }
            
            if server.host.is_empty() {
                anyhow::bail!("Server '{}' has empty host", name);
            }
            
            if server.user.is_empty() {
                anyhow::bail!("Server '{}' has empty user", name);
            }
            
            if server.port == 0 || server.port > 65535 {
                anyhow::bail!("Server '{}' has invalid port: {}", name, server.port);
            }
        }

        // Validate default server exists
        if let Some(default_server) = &self.client.default_server {
            if !self.servers.contains_key(default_server) {
                anyhow::bail!("Default server '{}' not found in servers list", default_server);
            }
        }

        // Validate thresholds
        if self.server.temp_threshold < 0.0 || self.server.temp_threshold > 150.0 {
            anyhow::bail!("Invalid temperature threshold: {}", self.server.temp_threshold);
        }
        
        if self.server.battery_warning_level > 100 {
            anyhow::bail!("Invalid battery warning level: {}", self.server.battery_warning_level);
        }

        Ok(())
    }

    pub fn merge_env_vars(&mut self) {
        // Override with environment variables
        if let Ok(host) = std::env::var("PLAN10_HOST") {
            if let Some(user) = std::env::var("PLAN10_USER").ok() {
                let port = std::env::var("PLAN10_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(22);

                let server = ServerDefinition {
                    name: "env".to_string(),
                    host,
                    user,
                    port,
                    ssh_key: std::env::var("PLAN10_SSH_KEY").ok(),
                    tags: vec!["env".to_string()],
                    enabled: true,
                    last_seen: None,
                };

                self.servers.insert("env".to_string(), server);
                self.client.default_server = Some("env".to_string());
            }
        }

        if let Ok(key_path) = std::env::var("PLAN10_SSH_KEY") {
            self.ssh.key_path = Some(key_path);
        }

        if let Ok(log_level) = std::env::var("PLAN10_LOG_LEVEL") {
            self.server.log_level = log_level;
        }
    }
}