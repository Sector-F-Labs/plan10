# Plan 10

Transform your MacBook into a dedicated headless server running in clamshell mode.

## Why Plan 10?

There are many MacBooks out there, especially the Intel kind, that have become almost unusable with fans spinning up constantly. Most of this whirring is due to their inability to handle the graphical tasks of modern macOS workloads. However, these machines are powerful Unix systems under the hood that are much more capable than a Raspberry Pi and have the potential to live for much longer in use as a home server (Plus they come with built in UPS and battery backup during power outages).

So the Plan 10 project was born, with a name loosely inspired by [Plan 9](https://en.wikipedia.org/wiki/Plan_9_from_Bell_Labs), since you can quite easily configure networked storage between Macs and the clean, minimal approach kind of hints at Plan 9-iness.

**Benefits of repurposing your MacBook as a server:**
- **More powerful than typical home servers** - Intel MacBooks have substantial CPU and RAM
- **Excellent build quality** - Apple hardware is designed to last
- **Built-in UPS** - Battery backup during power outages (with limitations, see below)
- **Low power consumption** - Especially when running headless
- **Familiar Unix environment** - Full macOS with Terminal access
- **Cost effective** - Repurpose existing hardware instead of buying new

## Overview

Plan 10 automates the setup of a MacBook as a persistent server with the following core features:
- Prevents system sleep and maintains uptime
- Configures power management for server operation
- Sets up automatic startup services
- Enables remote deployment capabilities
- Works best with external display or lid open

## Known Limitations & Safety Considerations

⚠️ **Clamshell Mode Battery Operation**: Currently, Plan 10 cannot maintain network connectivity when running on battery power with the lid closed (clamshell mode). This is a macOS system limitation that affects the "Built-in UPS" functionality.

🌡️ **Thermal Safety Warning**: Running on battery power with the lid closed can create thermal hazards as heat builds up inside the closed laptop without proper ventilation. This combination is **NOT RECOMMENDED** for server operation.

**Current Status:**
- ✅ **AC Power + Lid Closed**: Works perfectly (with external power, thermal management is adequate)
- ✅ **Battery Power + Lid Open**: Works perfectly (excellent thermal ventilation)
- ❌ **Battery Power + Lid Closed**: Network connectivity lost + thermal hazard risk

**Safety & Performance Benefits of Lid-Open Operation:**
- 🌡️ **Superior thermal management** - Direct airflow to internal components
- 🔥 **Prevents heat buildup** - Critical for sustained server workloads
- 🖥️ **Visual status indicators** - Can see power/activity LEDs
- 🔋 **Safer battery operation** - Better heat dissipation during charging/discharging
- 📊 **Performance benefits** - CPU can maintain higher clock speeds with better cooling

**Recommended Configurations:**
1. **Primary**: AC Power + External Display + Lid Open (best thermals)
2. **Alternative**: AC Power + Lid Closed (acceptable for light workloads)
3. **Emergency**: Battery Power + Lid Open (safe for temporary operation)
4. **AVOID**: Battery Power + Lid Closed (network + thermal issues)

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

## Plan 10 Installation & Deployment

### Option 1: Quick Install (Recommended)

Install the Plan 10 CLI with a single command:

```bash
# Install Plan 10 CLI (installs Rust if needed)
curl -sSf https://raw.githubusercontent.com/plan10/plan10/main/install.sh | sh
```

This installer will:
- Check for Rust/Cargo and install if missing
- Install Plan 10 CLI from GitHub
- Configure your PATH automatically
- Work on both macOS and Linux

After installation:
```bash
# Quick setup
plan10 setup

# Deploy to your server
plan10 client add myserver --host 192.168.1.100 --user admin
plan10 client deploy --host myserver --all
```

### Option 2: Manual Deployment (Legacy Method)

If you prefer the traditional approach, once your Mac is prepared with SSH access, deploy Plan 10 from your local machine:

### Deployment Workflow

```
   Local Machine                   Remote Server
┌─────────────────┐             ┌─────────────────┐
│ 1. make setup   │             │                 │
│    (configure)  │             │                 │
│                 │             │                 │
│ 2. make deploy  │────────────▶│ Files copied    │
│    (deploy all) │             │                 │
│                 │             │ 3. SSH login    │
│                 │◀────────────│ sudo setup.sh   │
│                 │             │                 │
│ 4. Verify       │             │ ✅ Server ready │
│ make diagnose   │◀────────────│                 │
└─────────────────┘             └─────────────────┘
```

### Step-by-Step Instructions

1. **Configure your environment:**
   ```sh
   make setup
   ```
   *Or manually create a `.env` file with SERVER_USER and SERVER_HOST*

2. **Deploy everything to your server:**
   ```sh
   make deploy
   ```
   *This copies all files: server setup, monitoring scripts, LaunchAgent, and aliases*

3. **Complete setup on target machine:**
   ```sh
   ssh your-user@your-server
   sudo ./server_setup.sh
   ```
   *This configures power management including network connectivity fixes for battery operation*

4. **Verify deployment:**
   ```sh
   make diagnose-remote
   ```
   *Check that all power management issues are resolved*

**Why the two-step process?** The server setup script requires `sudo` privileges to modify system power settings, which needs an interactive terminal session on the target machine.

**Important:** After setup, test your specific use case. The current limitation with clamshell mode on battery power may affect your deployment if you require true "closed lid + battery backup" operation.

## Core Components

### Power Management
- **caffeinate**: Prevents system sleep during idle, disk, and user activity
- **LaunchAgent**: Ensures caffeinate runs automatically at startup
- **System settings**: Configures auto-restart after power loss or freeze
- **Wake-on-LAN**: Enables remote wake capabilities

### Server Setup Script
The `server_setup.sh` script configures:
- Disables all sleep modes for continuous operation
- Automatic restart after system freeze
- Power-on after power loss
- Wake-on-network access
- Persistent caffeinate process

### System Monitoring
Built-in monitoring scripts provide real-time server status:
- `temp` - System temperature and thermal status monitoring
- `battery` - Battery level, charging status, and health information
- `power_diagnostics` - Power management diagnostics and troubleshooting
- `sysmon` - Help and overview of monitoring tools

## Server Operation Modes

### Recommended: Lid Open Operation
**Best for server workloads** - provides optimal thermal performance:

1. **Connect external power**
2. **Optional: Connect external display**
3. **Keep lid open** for best thermal management
4. **Position for good airflow** around vents

### Alternative: Clamshell Mode (AC Power Only)
Once configured, your MacBook can run with the lid closed **when connected to AC power**:

1. **Ensure external power is connected**
2. **Connect external display** (recommended for best compatibility)
3. **Monitor temperatures closely** - use `temp` command
4. **Test all services work properly**
5. **Close the lid** - system will remain awake and accessible via SSH

**Important Safety & Limitations:**
- ✅ **AC Power + Lid Closed**: Works but monitor temperatures
- ❌ **Battery Power + Lid Closed**: Network fails + thermal hazard
- 🌡️ **Thermal Management**: Lid open provides much better cooling
- 💡 **Best Practice**: Keep lid open for sustained server workloads

**Monitoring**: The system will stay awake indefinitely on AC power. **Always monitor temperatures** and ensure adequate cooling, especially in clamshell mode.

## Available Commands

```sh
make help           # Show all available commands
make setup          # Interactive environment setup
make check-env      # Display current configuration
make deploy         # Deploy complete server configuration
make diagnose-remote # Run power diagnostics on remote server
make apps           # Show available applications
```

## Documentation

- **[Usage Guide](docs/usage.md)** - Complete guide for daily operations, monitoring, and management
- **[Troubleshooting Guide](docs/troubleshooting.md)** - Solutions for common issues and recovery procedures

## Quick Reference

### Complete Deployment Process
1. **Deploy to server**: `make deploy`
2. **Complete setup**: SSH to server and run `sudo ./server_setup.sh`
3. **Verify**: `make diagnose-remote`

### System Monitoring Commands (After Deployment)
- `temp` - System temperature and thermal status
- `battery` - Battery level, charging status, and health
- `power_diagnostics` - Power management diagnostics and fixes
- `sysmon` - Help and overview of monitoring tools

For detailed usage examples and advanced operations, see the [Usage Guide](docs/usage.md).

### Common Issues
- **SSH Connection Problems** - See [SSH troubleshooting](docs/troubleshooting.md#ssh-connection-issues)
- **Power Management Issues** - See [Power management troubleshooting](docs/troubleshooting.md#power-management-issues)
- **Auto Login Not Working** - See [Auto login troubleshooting](docs/troubleshooting.md#auto-login-not-working)
- **Server Setup Requires SSH** - The `sudo ./server_setup.sh` command must be run interactively on the target machine, not remotely, due to password requirements
- **Clamshell Battery Networking** - Network connectivity is lost when running on battery with lid closed. This is a known macOS limitation.

For complete troubleshooting information, see the [Troubleshooting Guide](docs/troubleshooting.md).

## Use Cases and Limitations

**Ideal Use Cases:**
- Development server with lid open (best thermals)
- Home lab server with consistent AC power and good ventilation
- Media server with external display and open lid
- CI/CD server in controlled environment with temperature monitoring

**Current Limitations:**
- Battery backup requires lid open (for connectivity AND thermal safety)
- True "closed lid + battery only" operation not supported (network + thermal issues)
- Clamshell mode generates more heat - monitor temperatures closely
- Best suited for AC-powered server deployments with proper cooling

**Thermal Considerations:**
- ⭐ **Lid Open**: Excellent thermal performance, recommended for all server workloads
- ⚠️ **Lid Closed**: Acceptable on AC power for light workloads, monitor temperatures
- 🚫 **Battery + Lid Closed**: Not recommended due to thermal and connectivity issues

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
├── LICENSE               # BSD 3-Clause License
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
make deploy SERVER_USER=admin SERVER_HOST=macbook-server.local
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

## License

Plan 10 is released under the BSD 3-Clause License. See [LICENSE](LICENSE) for details.

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review logs in `/var/log/` on the server
3. Test individual components in isolation
4. Verify SSH connectivity and permissions
