# Plan 10 CLI

A powerful command-line interface for Plan 10 MacBook server management, written in Rust. Transform your MacBook into a dedicated headless server with comprehensive monitoring, power management, and remote deployment capabilities.

## Features

- **Dual-Mode Operation**: Works as both client (for remote management) and server (for local operations)
- **Remote Server Management**: Deploy, configure, and monitor multiple Plan 10 servers
- **Real-time Monitoring**: Temperature, battery, power management, and system diagnostics
- **Power Management**: Advanced macOS power settings optimization for server workloads
- **SSH Integration**: Secure remote operations with key-based authentication
- **Interactive Setup**: Guided configuration wizard for both client and server modes
- **Continuous Monitoring**: Watch mode for real-time system status updates
- **Configuration Management**: Centralized configuration with validation

## Installation

### Quick Install (Recommended)

Install Plan 10 CLI with a single command that handles Rust dependencies:

```bash
# One-liner installer (installs Rust if needed)
curl -sSf https://raw.githubusercontent.com/plan10/plan10/main/install.sh | sh

# Or use the compact version
curl -sSf https://raw.githubusercontent.com/plan10/plan10/main/install-one-liner.sh | sh
```

This installer will:
- Detect your operating system (macOS/Linux)
- Install Rust and Cargo if not present
- Install Plan 10 CLI directly from GitHub
- Configure your PATH automatically

After installation, restart your terminal or run:
```bash
source ~/.cargo/env
```

### Manual Installation

#### From Source

```bash
# Install Rust first (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Plan 10 CLI from GitHub
cargo install --git https://github.com/plan10/plan10 plan10-cli

# Or clone and build locally
git clone https://github.com/plan10/plan10.git
cd plan10/plan10-cli
cargo build --release
cargo install --path .
```

#### Pre-built Binaries

Download the latest release from the [releases page](https://github.com/plan10/plan10/releases).

## Quick Start

### Initial Setup

Run the interactive setup wizard to configure your environment:

```bash
plan10 setup
```

### Client Mode (Managing Remote Servers)

```bash
# Add a server
plan10 client add myserver --host 192.168.1.100 --user admin

# Deploy Plan 10 to server
plan10 client deploy --host myserver --all

# Monitor remote server
plan10 monitor system --host myserver
plan10 monitor temp --host myserver
plan10 monitor battery --host myserver --detailed
```

### Server Mode (Local Operations)

```bash
# Configure this machine as a server
plan10 server configure

# Check system status
plan10 status --detailed

# Monitor local system
plan10 monitor temp
plan10 monitor battery
plan10 monitor power --fixes

# Watch continuous monitoring
plan10 monitor watch --interval 10 --monitor all
```

## Command Reference

### Global Options

- `-c, --config <FILE>`: Configuration file path
- `-v, --verbose`: Verbose output
- `--server-mode`: Force server mode (local operations)
- `--client-mode`: Force client mode (remote operations)

### Client Commands

#### Server Management

```bash
# List configured servers
plan10 client list [--detailed]

# Add new server
plan10 client add <name> --host <host> --user <user> [--port <port>]

# Remove server
plan10 client remove <name>

# Deploy to server
plan10 client deploy --host <host> [--all|--scripts-only|--config-only]

# Manage remote server
plan10 client manage --host <host> <start|stop|restart|update|status|configure>

# Remote diagnostics
plan10 client diagnose --host <host> [--battery|--power|--fixes]
```

### Server Commands

#### Service Management

```bash
# Configure server
plan10 server configure [--yes] [--power] [--monitoring] [--services]

# Service control
plan10 server start [--service <name>]
plan10 server stop [--service <name>]
plan10 server restart [--service <name>]
plan10 server services [--detailed]
```

#### Power Management

```bash
# Power status
plan10 server power status

# Configure power settings
plan10 server power configure [--no-hibernate] [--no-sleep] [--halt-level <level>]

# Reset power settings
plan10 server power reset

# Power diagnostics
plan10 server power diagnostics
```

#### Maintenance

```bash
# System updates
plan10 server maintenance update

# Clean temporary files
plan10 server maintenance clean

# Backup configuration
plan10 server maintenance backup [--output <file>]

# Restore configuration
plan10 server maintenance restore <file>

# Health check
plan10 server maintenance health
```

### Monitoring Commands

#### Temperature Monitoring

```bash
# Basic temperature status
plan10 monitor temp

# Raw temperature data
plan10 monitor temp --raw

# Remote temperature monitoring
plan10 monitor temp --host <server>
```

#### Battery Monitoring

```bash
# Basic battery status
plan10 monitor battery

# Detailed battery health
plan10 monitor battery --detailed

# Raw battery data
plan10 monitor battery --raw

# Remote battery monitoring
plan10 monitor battery --host <server>
```

#### Power Diagnostics

```bash
# Basic power diagnostics
plan10 monitor power

# Battery-focused diagnostics
plan10 monitor power --battery

# Sleep/wake diagnostics
plan10 monitor power --sleep

# All diagnostics
plan10 monitor power --all

# Show recommended fixes
plan10 monitor power --fixes

# Remote power diagnostics
plan10 monitor power --host <server> --all
```

#### System Monitoring

```bash
# System overview
plan10 monitor system

# Remote system monitoring
plan10 monitor system --host <server>
```

#### Continuous Monitoring

```bash
# Watch all metrics (5-second interval)
plan10 monitor watch

# Watch specific metric
plan10 monitor watch --monitor temp --interval 10

# Watch remote server
plan10 monitor watch --host <server> --monitor all
```

### Status and Configuration

```bash
# Quick status check
plan10 status

# Detailed status
plan10 status --detailed

# Remote status check
plan10 status --host <server>

# Show configuration
plan10 config

# Show server-specific configuration
plan10 config --server <name>

# Edit configuration
plan10 config --edit

# Interactive setup
plan10 setup [auto|client|server|both]
```

## Configuration

### Configuration File

The CLI uses a TOML configuration file located at:
- macOS: `~/Library/Application Support/plan10/config.toml`
- Linux: `~/.config/plan10/config.toml`

### Environment Variables

- `PLAN10_CONFIG`: Override config file path
- `PLAN10_HOST`: Default server host
- `PLAN10_USER`: Default SSH user
- `PLAN10_PORT`: Default SSH port
- `PLAN10_SSH_KEY`: Default SSH key path
- `PLAN10_LOG_LEVEL`: Log level (debug, info, warn, error)

### Sample Configuration

```toml
[client]
default_server = "macbook-server"
deployment_timeout = 300
concurrent_operations = 4
auto_backup = true

[server]
name = "my-macbook-server"
monitoring_interval = 30
temp_threshold = 80.0
battery_warning_level = 20
auto_restart_services = true
log_level = "info"
services = ["caffeinate", "plan10-monitor"]

[ssh]
connect_timeout = 30
command_timeout = 60
compression = true
keep_alive = true

[servers]
[servers.macbook-server]
name = "macbook-server"
host = "192.168.1.100"
user = "admin"
port = 22
tags = ["production"]
enabled = true
```

## Advanced Usage

### SSH Key Authentication

```bash
# Generate SSH key for Plan 10
ssh-keygen -t ed25519 -f ~/.ssh/plan10_key -C "plan10-client"

# Add to server
ssh-copy-id -i ~/.ssh/plan10_key admin@192.168.1.100

# Configure in Plan 10
plan10 config --edit
# Set ssh.key_path = "~/.ssh/plan10_key"
```

### Multiple Server Management

```bash
# Add multiple servers
plan10 client add server1 --host 192.168.1.100 --user admin
plan10 client add server2 --host 192.168.1.101 --user admin
plan10 client add server3 --host 192.168.1.102 --user admin

# Deploy to all servers
for server in server1 server2 server3; do
    plan10 client deploy --host $server --all
done

# Monitor all servers
plan10 client list --detailed
```

### Custom Monitoring Scripts

The CLI can execute custom monitoring scripts on remote servers:

```bash
# Deploy custom script
scp my_monitor.sh admin@server:~/scripts/

# Execute remotely
plan10 client manage --host server status
```

### Automation and Scripting

```bash
#!/bin/bash
# Health check script

SERVERS=("server1" "server2" "server3")

for server in "${SERVERS[@]}"; do
    echo "Checking $server..."
    plan10 monitor power --host $server --fixes > /tmp/plan10-$server.log
    
    if grep -q "ISSUE" /tmp/plan10-$server.log; then
        echo "WARNING: Issues found on $server"
        cat /tmp/plan10-$server.log
    else
        echo "OK: $server is healthy"
    fi
done
```

## Troubleshooting

### Common Issues

#### SSH Connection Problems

```bash
# Test SSH connectivity
ssh admin@192.168.1.100

# Check SSH key permissions
chmod 600 ~/.ssh/plan10_key

# Verify server configuration
plan10 config --server myserver

# Test Plan 10 connectivity
plan10 client diagnose --host myserver
```

#### Permission Issues

```bash
# Ensure proper permissions for LaunchAgents
chmod 644 ~/Library/LaunchAgents/caffeinate.plist

# Check sudo access for power management
sudo pmset -g

# Verify script permissions
ls -la ~/scripts/
```

#### Power Management Issues

```bash
# Check current power settings
pmset -g

# Run power diagnostics
plan10 monitor power --all --fixes

# Reset power settings
sudo pmset -a restoredefaults
```

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
# Set environment variable
export PLAN10_LOG_LEVEL=debug

# Or use --verbose flag
plan10 --verbose monitor system

# Check configuration
plan10 config --verbose
```

### Log Files

- Client logs: `~/.local/share/plan10/client.log`
- Server logs: `/var/log/plan10.log`
- SSH logs: Use `-v` flag with SSH commands

## Development

### Building from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/plan10/plan10.git
cd plan10/plan10-cli

# Run tests
cargo test

# Build debug binary
cargo build

# Build release binary
cargo build --release

# Run locally
cargo run -- --help
```

### Code Structure

```
src/
├── main.rs              # CLI entry point
├── config.rs            # Configuration management
├── ssh.rs               # SSH client implementation
├── commands/            # Command implementations
│   ├── client/          # Client-side commands
│   ├── server/          # Server-side commands
│   └── shared/          # Shared commands (monitoring)
└── utils/               # Utilities and helpers
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

Plan 10 CLI is released under the BSD 3-Clause License. See [LICENSE](../LICENSE) for details.

## Support

- **Documentation**: [Plan 10 Docs](../docs/)
- **Issues**: [GitHub Issues](https://github.com/plan10/plan10/issues)
- **Discussions**: [GitHub Discussions](https://github.com/plan10/plan10/discussions)

## Related Projects

- **Plan 10 Core**: The main Plan 10 shell scripts and configurations
- **Plan 10 Web**: Web interface for Plan 10 server management
- **Plan 10 Mobile**: Mobile app for monitoring Plan 10 servers