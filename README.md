# Plan 10

Transform your MacBook into a dedicated headless server running in clamshell mode.

## Overview

Plan 10 automates the setup of a MacBook as a persistent server with the following core features:
- Prevents system sleep and maintains uptime
- Configures power management for server operation
- Sets up automatic startup services
- Enables remote deployment capabilities
- Supports clamshell (closed lid) operation

## Prerequisites

- macOS system (target server)
- SSH access configured between local and target machines
- Administrative privileges on the target machine

## Quick Start

1. **Configure your environment:**
   ```sh
   make setup
   ```

2. **Deploy to your server:**
   ```sh
   make push
   ```

3. **Run server setup on target machine:**
   ```sh
   sudo ./server_setup.sh
   ```

## Configuration

Set your server details using the interactive setup:

```sh
make setup
```

Or manually create a `.env` file:
```sh
SERVER_USER=your-username
SERVER_HOST=your-server-hostname
```

You can also override variables directly:
```sh
make push SERVER_USER=admin SERVER_HOST=macbook-server
```

## Core Components

### Power Management
- **caffeinate**: Prevents system sleep during idle, disk, and user activity
- **LaunchAgent**: Ensures caffeinate runs automatically at startup
- **System settings**: Configures auto-restart after power loss or freeze

### Server Setup Script
The `server_setup.sh` script configures:
- Automatic restart after system freeze
- Power-on after power loss
- Wake-on-network access
- Persistent caffeinate process

### Deployment
The Makefile automates:
- Configuration file deployment
- Service installation
- Remote command execution

## Manual Operations

### Keep System Awake
```sh
caffeinate -imsu
```

Flags explained:
- `-i`: Prevent system sleep when idle
- `-m`: Prevent disk sleep  
- `-s`: Prevent system sleep
- `-u`: Prevent user activity sleep

### LaunchAgent Management
```sh
# Install the service
cp caffeinate.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/caffeinate.plist

# Check status
launchctl list | grep caffeinate

# Unload if needed
launchctl unload ~/Library/LaunchAgents/caffeinate.plist
```

## Available Commands

```sh
make help           # Show all available commands
make setup          # Interactive environment setup
make check-env      # Display current configuration
make push           # Deploy configuration to server
make push-scripts   # Deploy system monitoring scripts
```

## Clamshell Mode

Once configured, your MacBook can run with the lid closed:

1. Connect external power
2. Connect to external display (temporarily, for initial setup)
3. Close the lid
4. The system will remain awake and accessible via SSH

## Troubleshooting

### Check if caffeinate is running:
```sh
pgrep -x "caffeinate"
```

### View system power settings:
```sh
pmset -g
```

### Test SSH connectivity:
```sh
ssh your-user@your-server 'echo "Connection successful"'
```

### Check LaunchAgent status:
```sh
launchctl list | grep caffeinate
```

## Security Considerations

- Ensure SSH key-based authentication is configured
- Consider firewall rules for services you plan to run
- The server will remain awake indefinitely - monitor power consumption
- Review which services auto-start with the system

## System Monitoring

Built-in system monitoring scripts provide real-time server status:

### Deploy Monitoring Scripts
```sh
make push-scripts
```

### Available Commands (after deployment)
- `temp` - System temperature and thermal status monitoring
- `battery` - Battery level, charging status, and health information
- `sysmon` - Help and overview of monitoring tools

### Features
- **Temperature Monitoring**: CPU/GPU temperatures, thermal state, fan status
- **Battery Monitoring**: Charge level, health metrics, cycle count tracking
- **Shell Integration**: Automatic alias setup for easy access

See `scripts/README.md` for detailed usage and examples.

## Applications

Additional applications can be installed and configured separately:

- `apps/neo4j/` - Neo4j graph database setup
- More applications can be added to the `apps/` directory

Each application directory contains its own README and deployment scripts.

## File Structure

```
plan10/
├── README.md              # This file - core server setup
├── Makefile              # Main deployment automation
├── setup.sh              # Interactive configuration setup
├── server_setup.sh       # Server configuration script
├── caffeinate.plist      # LaunchAgent for persistent wake
├── scripts/              # System monitoring utilities
│   ├── README.md         # Monitoring scripts documentation
│   ├── temp              # Temperature monitoring script
│   ├── battery           # Battery monitoring script
│   └── setup_aliases.sh  # Alias configuration script
└── apps/                 # Application-specific configurations
    └── neo4j/            # Neo4j database setup
```

## Contributing

When adding new applications or features:
1. Create a new directory under `apps/` for application-specific configs
2. Include a README.md with setup instructions
3. Add a Makefile for deployment automation
4. Update this main README if core server functionality changes