# Plan 10

Transform your MacBook into a dedicated headless server running in clamshell mode.

## Overview

Plan 10 automates the setup of a MacBook as a persistent server with the following core features:
- Prevents system sleep and maintains uptime
- Configures power management for server operation
- Sets up automatic startup services
- Enables remote deployment capabilities
- Supports clamshell (closed lid) operation

## From Scratch Setup

### Prerequisites

For the best server setup experience, start with a clean slate:

1. **Factory Reset (Recommended)**
   - Back up any important data
   - Sign out of all Apple services
   - Go to Apple Menu > System Settings > General > Transfer or Reset
   - Choose "Erase All Content and Settings"
   - Follow the prompts to completely reset your Mac

### Initial macOS Setup

2. **Create Local Account (No Apple ID)**
   - During initial setup, when prompted for Apple ID, click "Set Up Later" or "Skip"
   - Create a local administrator account
   - Choose a strong password you'll remember
   - **Important**: Avoid using Apple ID for server accounts to prevent sync issues

3. **Complete Basic Setup**
   - Set timezone and region
   - Skip Siri, Screen Time, and other optional features
   - Decline analytics sharing for privacy
   - Skip iCloud setup completely

### Server Optimization

4. **Reduce Visual Effects (For Clamshell Mode)**
   - Go to System Settings > Accessibility > Display
   - Check "Reduce motion"
   - Check "Reduce transparency"
   - Go to System Settings > Displays
   - Lower resolution to 1280x800 or similar (reduces GPU load)
   - Set refresh rate to 60Hz

5. **Configure Auto Login (Critical for Servers)**
   - Go to System Settings > General > Login Items & Extensions
   - Click "Automatically log in as:" dropdown
   - Select your user account
   - Enter your password when prompted
   - **Why this matters**: Servers need to boot without manual intervention

6. **Enable SSH Server**
   - Go to System Settings > General > Sharing
   - Turn on "Remote Login"
   - Choose "All users" or specific users as needed
   - Note the SSH command shown (e.g., `ssh username@192.168.1.100`)
   - **Test SSH access** from another machine before proceeding

### Additional Server Preparations

7. **Disable Sleep and Screensaver**
   - Go to System Settings > Displays > Advanced
   - Set "Prevent automatic sleeping when display is off" (if available)
   - Go to System Settings > Lock Screen
   - Set "Start Screen Saver when inactive" to "Never"
   - Set "Turn display off when inactive" to "Never"

8. **Configure Network (Optional but Recommended)**
   - Go to System Settings > Network
   - Configure static IP if desired for consistent SSH access
   - Note your IP address for SSH connections

9. **Install Homebrew (Required for some apps)**
   ```sh
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

10. **Test Remote Access**
    - From another computer, test SSH: `ssh your-username@your-mac-ip`
    - Ensure you can connect and run commands
    - If using SSH keys, set them up now for passwordless access

## Plan 10 Deployment

Once your Mac is prepared with SSH access, deploy Plan 10 from your local machine:

### Configuration

1. **Configure your environment:**
   ```sh
   make setup
   ```

2. **Or manually create a `.env` file:**
   ```sh
   SERVER_USER=your-username
   SERVER_HOST=your-server-hostname-or-ip
   ```

### Deploy Core Server Configuration

3. **Deploy to your server:**
   ```sh
   make push
   ```

4. **Run server setup on target machine:**
   ```sh
   ssh your-user@your-server
   sudo ./server_setup.sh
   ```

### Deploy System Monitoring

5. **Deploy monitoring scripts:**
   ```sh
   make push-scripts
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

### System Monitoring
Built-in monitoring scripts provide real-time server status:
- `temp` - System temperature and thermal status monitoring
- `battery` - Battery level, charging status, and health information
- `sysmon` - Help and overview of monitoring tools

## Clamshell Mode Operation

Once configured, your MacBook can run with the lid closed:

1. **Ensure external power is connected**
2. **Connect external display temporarily** (for any GUI needs)
3. **Test all services work properly**
4. **Close the lid** - system will remain awake and accessible via SSH

**Important**: The system will stay awake indefinitely. Monitor power consumption and ensure adequate cooling.

## Available Commands

```sh
make help           # Show all available commands
make setup          # Interactive environment setup
make check-env      # Display current configuration
make push           # Deploy core server configuration
make push-scripts   # Deploy system monitoring scripts
make apps           # Show available applications
```

## Documentation

- **[Usage Guide](docs/usage.md)** - Complete guide for daily operations, monitoring, and management
- **[Troubleshooting Guide](docs/troubleshooting.md)** - Solutions for common issues and recovery procedures

## Quick Reference

### System Monitoring Commands (After Deployment)
- `temp` - System temperature and thermal status
- `battery` - Battery level, charging status, and health
- `sysmon` - Help and overview of monitoring tools

For detailed usage examples and advanced operations, see the [Usage Guide](docs/usage.md).

### Common Issues
- **SSH Connection Problems** - See [SSH troubleshooting](docs/troubleshooting.md#ssh-connection-issues)
- **Power Management Issues** - See [Power management troubleshooting](docs/troubleshooting.md#power-management-issues)
- **Auto Login Not Working** - See [Auto login troubleshooting](docs/troubleshooting.md#auto-login-not-working)

For complete troubleshooting information, see the [Troubleshooting Guide](docs/troubleshooting.md).

## Security Considerations

- **SSH Key Authentication**: Set up key-based SSH auth and disable password auth
- **Firewall**: Configure macOS firewall for services you plan to run
- **Network Access**: Consider VPN or network restrictions for SSH access
- **Physical Security**: Secure the physical location of your server
- **Regular Updates**: Keep macOS updated for security patches
- **Local Account**: Using local accounts (not Apple ID) improves security for servers

## Applications

Additional applications can be installed and configured:

- `apps/neo4j/` - Neo4j graph database setup
- More applications can be added to the `apps/` directory

Each application directory contains its own README and deployment scripts.

## File Structure

```
plan10/
├── README.md              # This file - complete setup guide
├── Makefile              # Main deployment automation
├── setup.sh              # Interactive configuration setup
├── server_setup.sh       # Server configuration script
├── caffeinate.plist      # LaunchAgent for persistent wake
├── docs/                 # Documentation
│   ├── usage.md          # Complete usage and operations guide
│   └── troubleshooting.md # Troubleshooting and recovery guide
├── scripts/              # System monitoring utilities
│   ├── README.md         # Monitoring scripts documentation
│   ├── temp              # Temperature monitoring script
│   ├── battery           # Battery monitoring script
│   └── setup_aliases.sh  # Alias configuration script
└── apps/                 # Application-specific configurations
    └── neo4j/            # Neo4j database setup
```

## Advanced Configuration

### Custom Environment Variables
Override any configuration:
```sh
make push SERVER_USER=admin SERVER_HOST=macbook-server.local
```

### Manual LaunchAgent Management
```sh
# Install the service manually
cp caffeinate.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/caffeinate.plist

# Check status
launchctl list | grep caffeinate

# Unload if needed
launchctl unload ~/Library/LaunchAgents/caffeinate.plist
```

### Manual Power Management
```sh
# Keep system awake manually
caffeinate -imsu

# Check what's preventing sleep
pmset -g assertions
```

## Contributing

When adding new applications or features:

1. Create a new directory under `apps/` for application-specific configs
2. Include a README.md with setup instructions
3. Add a Makefile for deployment automation if complex
4. Update this main README if core server functionality changes
5. Test thoroughly on a fresh macOS installation

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review logs in `/var/log/` on the server
3. Test individual components in isolation
4. Verify SSH connectivity and permissions