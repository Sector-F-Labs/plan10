# System Monitoring Scripts

System monitoring utilities for Plan 10 server management.

## Scripts

### Server Setup Scripts

The main server setup is handled by the consolidated `server_setup.sh` script in the root directory, which includes comprehensive battery power management. The scripts in this directory provide monitoring and diagnostics.

### `temp`
Temperature and thermal monitoring script for macOS systems.

**Usage:**
```sh
temp           # Show formatted temperature status
temp -r        # Show raw temperature data
temp --help    # Show help message
```

**Features:**
- CPU and GPU temperature monitoring (requires sudo for detailed info)
- Thermal state reporting
- CPU usage with thermal indicators
- Fan status monitoring
- Color-coded alerts based on system load

### `battery`
Battery status and health monitoring script for macOS systems.

**Usage:**
```sh
battery        # Show battery status
battery -d     # Show detailed battery health info
battery -r     # Show raw battery data
battery --help # Show help message
```

**Features:**
- Battery charge level and status
- Charging/discharging indicators
- Time remaining estimates
- Battery health and cycle count
- Condition monitoring with alerts

### `power_diagnostics`
Power management diagnostics and troubleshooting script for macOS systems.

**Usage:**
```sh
power_diagnostics        # Basic power status and analysis
power_diagnostics -b     # Battery-focused diagnostics
power_diagnostics -s     # Sleep/wake diagnostics
power_diagnostics -f     # Show recommended fixes
power_diagnostics -a     # Complete power analysis
power_diagnostics --help # Show help message
```

**Features:**
- Analyzes power management settings for server operation
- Identifies issues that could cause shutdowns on battery power
- Provides specific fix commands for identified problems
- Monitors battery health and charging status
- Checks caffeinate and power assertion status

### `setup_aliases.sh`
Automatically configures shell aliases for the monitoring tools.

**What it does:**
- Detects your shell (zsh/bash)
- Backs up existing shell profile
- Adds aliases to your shell configuration
- Creates a `sysmon` help command

## Installation

### Automatic Deployment
From your local machine:
```sh
make push-enhanced     # Deploy server setup with power management + diagnostics
make push-scripts      # Deploy monitoring scripts only
```

### Manual Installation
1. Copy scripts to server:
```sh
scp scripts/* user@server:~/scripts/
```

2. Set up server configuration:
```sh
ssh user@server
sudo ./server_setup.sh
```

3. Set up aliases:
```sh
cd ~/scripts
./setup_aliases.sh
source ~/.zshrc  # or ~/.bashrc
```

## Available Aliases

After installation, these commands are available:

- `temp` - System temperature monitoring
- `battery` - Battery status and health
- `power_diagnostics` - Power management diagnostics and fixes
- `sysmon` - Help and command overview

## Requirements

- macOS system (scripts use macOS-specific tools)
- `sudo` access for detailed temperature monitoring
- Battery-equipped device for battery monitoring

## Examples

### Temperature Monitoring
```sh
$ temp
🌡️  System Temperature Status
================================
CPU die temperature: 45.2°C
CPU Usage: 23%
❄️  Low CPU load - system cool
```

### Battery Status
```sh
$ battery
🔋 Battery Status
==================
Charge Level: 87%
Status: 🔌 Charging (AC Power)
Time to Full: 1:23
🟢 Good Battery Level
```

### Detailed Battery Health
```sh
$ battery -d
🔋 Battery Status
==================
Charge Level: 87%
Status: ✅ Fully Charged
🟢 Good Battery Level

🏥 Battery Health
==================
Cycle Count: 342
✅ Low cycle count - battery in good shape
Condition: Normal
✅ Battery condition is normal
```

## Troubleshooting

### Temperature Script Issues
- Detailed temperature requires `sudo` access
- If `powermetrics` is unavailable, script falls back to CPU usage indicators
- Some features may not work on older macOS versions

### Battery Script Issues
- Desktop Macs without batteries will show appropriate messages
- External battery monitors may not be detected
- Some health metrics require newer macOS versions

### Alias Setup Issues
- Script creates backups of shell profiles before modification
- Supports both zsh and bash
- Run `source ~/.zshrc` or restart terminal after setup

## Security Notes

- Temperature monitoring with `sudo` provides more detailed information
- Scripts only read system information, no modifications are made
- All data is displayed locally, no external network calls