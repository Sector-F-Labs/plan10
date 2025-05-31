use anyhow::{Context, Result};
use ssh2::Session;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;
use tokio::time::timeout;
use tokio::net::TcpStream;

use crate::config::{Config, ServerDefinition};

pub struct SshClient {
    session: Session,
    server: ServerDefinition,
}

impl SshClient {
    pub async fn connect(server: &ServerDefinition, config: &Config) -> Result<Self> {
        let tcp = timeout(
            Duration::from_secs(config.ssh.connect_timeout),
            TcpStream::connect(format!("{}:{}", server.host, server.port))
        ).await
        .context("Connection timeout")?
        .context("Failed to connect to server")?;
        
        let std_tcp = tcp.into_std()?;

        let mut session = Session::new()?;
        session.set_tcp_stream(std_tcp);
        session.handshake()
            .context("SSH handshake failed")?;

        // Try key authentication first
        if let Some(key_path) = &server.ssh_key.as_ref().or(config.ssh.key_path.as_ref()) {
            let key_path = shellexpand::tilde(key_path);
            if Path::new(&*key_path).exists() {
                session.userauth_pubkey_file(
                    &server.user,
                    None,
                    Path::new(&*key_path),
                    None
                ).context("SSH key authentication failed")?;
            }
        }

        // Fall back to SSH agent if key auth didn't work
        if !session.authenticated() {
            session.userauth_agent(&server.user)
                .context("SSH agent authentication failed")?;
        }

        // Final fallback to interactive auth (will fail in non-interactive mode)
        if !session.authenticated() {
            anyhow::bail!("Authentication failed for user {} on {}", server.user, server.host);
        }

        Ok(Self {
            session,
            server: server.clone(),
        })
    }

    pub fn execute_command(&self, command: &str) -> Result<CommandResult> {
        let mut channel = self.session.channel_session()?;
        channel.exec(command)?;

        let mut stdout = String::new();
        let mut stderr = String::new();
        
        channel.read_to_string(&mut stdout)?;
        channel.stderr().read_to_string(&mut stderr)?;
        
        channel.wait_close()?;
        let exit_status = channel.exit_status()?;

        Ok(CommandResult {
            stdout,
            stderr,
            exit_code: exit_status,
            success: exit_status == 0,
        })
    }

    pub fn execute_command_with_timeout(&self, command: &str, _timeout_secs: u64) -> Result<CommandResult> {
        // For now, just use the regular execute_command
        // In a real implementation, you'd want to handle timeouts properly
        self.execute_command(command)
    }

    pub fn copy_file(&self, local_path: &Path, remote_path: &str) -> Result<()> {
        let local_content = std::fs::read(local_path)
            .context(format!("Failed to read local file: {}", local_path.display()))?;

        let mut remote_file = self.session.scp_send(
            Path::new(remote_path),
            0o644,
            local_content.len() as u64,
            None
        )?;

        remote_file.write_all(&local_content)?;
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        Ok(())
    }

    pub fn copy_directory(&self, local_dir: &Path, remote_dir: &str) -> Result<()> {
        use walkdir::WalkDir;

        // Create remote directory
        self.execute_command(&format!("mkdir -p {}", remote_dir))?;

        for entry in WalkDir::new(local_dir) {
            let entry = entry?;
            let local_path = entry.path();
            
            if local_path.is_file() {
                let relative_path = local_path.strip_prefix(local_dir)?;
                let remote_path = format!("{}/{}", remote_dir, relative_path.display());
                
                // Create parent directory if needed
                if let Some(parent) = Path::new(&remote_path).parent() {
                    self.execute_command(&format!("mkdir -p {}", parent.display()))?;
                }
                
                self.copy_file(local_path, &remote_path)?;
            }
        }

        Ok(())
    }

    pub fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<()> {
        let (mut remote_file, _stat) = self.session.scp_recv(Path::new(remote_path))?;
        
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        std::fs::write(local_path, contents)
            .context(format!("Failed to write to local file: {}", local_path.display()))?;

        Ok(())
    }

    pub fn file_exists(&self, remote_path: &str) -> Result<bool> {
        let result = self.execute_command(&format!("test -f {}", remote_path));
        Ok(result.map(|r| r.success).unwrap_or(false))
    }

    pub fn directory_exists(&self, remote_path: &str) -> Result<bool> {
        let result = self.execute_command(&format!("test -d {}", remote_path));
        Ok(result.map(|r| r.success).unwrap_or(false))
    }

    pub fn ensure_directory(&self, remote_path: &str) -> Result<()> {
        self.execute_command(&format!("mkdir -p {}", remote_path))?;
        Ok(())
    }

    pub fn get_server_info(&self) -> &ServerDefinition {
        &self.server
    }

    pub fn test_connection(&self) -> Result<()> {
        let result = self.execute_command("echo 'connection test'")?;
        if result.success && result.stdout.trim() == "connection test" {
            Ok(())
        } else {
            anyhow::bail!("Connection test failed")
        }
    }

    pub fn get_system_info(&self) -> Result<SystemInfo> {
        let uname_result = self.execute_command("uname -a")?;
        let uptime_result = self.execute_command("uptime")?;
        let df_result = self.execute_command("df -h /")?;
        let whoami_result = self.execute_command("whoami")?;

        Ok(SystemInfo {
            hostname: self.server.host.clone(),
            uname: uname_result.stdout.trim().to_string(),
            uptime: uptime_result.stdout.trim().to_string(),
            disk_usage: df_result.stdout.trim().to_string(),
            current_user: whoami_result.stdout.trim().to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

impl CommandResult {
    pub fn ensure_success(self) -> Result<Self> {
        if self.success {
            Ok(self)
        } else {
            anyhow::bail!(
                "Command failed with exit code {}: {}",
                self.exit_code,
                self.stderr.trim()
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub uname: String,
    pub uptime: String,
    pub disk_usage: String,
    pub current_user: String,
}

pub async fn test_connectivity(server: &ServerDefinition, config: &Config) -> Result<bool> {
    match SshClient::connect(server, config).await {
        Ok(client) => {
            client.test_connection().map(|_| true)
        }
        Err(_) => Ok(false),
    }
}

pub async fn execute_remote_command(
    server: &ServerDefinition,
    config: &Config,
    command: &str,
) -> Result<CommandResult> {
    let client = SshClient::connect(server, config).await?;
    client.execute_command(command)
}

pub async fn deploy_files(
    server: &ServerDefinition,
    config: &Config,
    local_files: &[(std::path::PathBuf, String)],
) -> Result<()> {
    let client = SshClient::connect(server, config).await?;
    
    for (local_path, remote_path) in local_files {
        if local_path.is_file() {
            client.copy_file(local_path, remote_path)?;
        } else if local_path.is_dir() {
            client.copy_directory(local_path, remote_path)?;
        }
    }
    
    Ok(())
}

// SSH connection pool for managing multiple concurrent connections
pub struct SshPool {
    connections: std::collections::HashMap<String, SshClient>,
    config: Config,
}

impl SshPool {
    pub fn new(config: Config) -> Self {
        Self {
            connections: std::collections::HashMap::new(),
            config,
        }
    }

    pub async fn get_connection(&mut self, server: &ServerDefinition) -> Result<&SshClient> {
        let key = format!("{}@{}:{}", server.user, server.host, server.port);
        
        if !self.connections.contains_key(&key) {
            let client = SshClient::connect(server, &self.config).await?;
            self.connections.insert(key.clone(), client);
        }
        
        Ok(self.connections.get(&key).unwrap())
    }

    pub fn disconnect(&mut self, server: &ServerDefinition) {
        let key = format!("{}@{}:{}", server.user, server.host, server.port);
        self.connections.remove(&key);
    }

    pub fn disconnect_all(&mut self) {
        self.connections.clear();
    }
}