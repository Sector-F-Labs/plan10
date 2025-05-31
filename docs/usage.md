# Plan 10 Usage Guide

Complete guide for operating and managing your Plan 10 server.

## Daily Operations

### Basic Commands

```sh
# Check current configuration
make check-env

# Deploy updates to server
make push

# Deploy monitoring scripts
make push-scripts

# View available applications
make apps
```

### System Monitoring

Once deployed, these commands are available on your server:

```sh
# Temperature monitoring
temp                    # Show formatted temperature status
temp -r                 # Show raw temperature data
temp --help            # Show help

# Battery monitoring
battery                # Show battery status
battery -d             # Show detailed health info
battery -r             # Show raw battery data
battery --help         # Show help

# System overview
sysmon                 # Show monitoring tools help
```

### Example Monitoring Output

**Temperature Status:**
```
🌡️  System Temperature Status
================================
CPU die temperature: 45.2°C
CPU Usage: 23%
❄️  Low CPU load - system cool

💨 Fan Status:
Fan 0 output: 1200 RPM
Fan 1 output: 1180 RPM
```

**Battery Status:**
```
🔋 Battery Status
==================
Charge Level: 87%
Status: 🔌 Charging (AC Power)
Time to Full: 1:23
🟢 Good Battery Level

🏥 Battery Health
==================
Cycle Count: 342
✅ Low cycle count - battery in good shape
Condition: Normal
✅ Battery condition is normal
```

## Clamshell Mode Operation

### Prerequisites
- External power adapter connected
- Auto login configured
- SSH server enabled and tested
- Plan 10 services deployed and running

### Entering Clamshell Mode

1. **Verify Services Are Running**
   ```sh
   ssh your-user@your-server
   pgrep -x "caffeinate"  # Should return a process ID
   launchctl list | grep caffeinate  # Should show loaded service
   ```

2. **Test System Response**
   ```sh
   # Run a quick system check
   temp && battery && echo "System ready for clamshell mode"
   ```

3. **Close the Lid**
   - Ensure external power is connected
   - Close the MacBook lid
   - System remains awake and accessible via SSH

### Verifying Clamshell Operation

```sh
# From another machine, test connectivity
ssh your-user@your-server 'date && uptime'

# Check power management status
ssh your-user@your-server 'pmset -g assertions'

# Monitor system resources
ssh your-user@your-server 'top -l 1 | head -10'
```

## Power Management

### Checking Power Status

```sh
# View current power settings
pmset -g

# Check what's preventing sleep
pmset -g assertions

# View power adapter status
pmset -g adapter

# Check battery status (if applicable)
pmset -g batt
```

### Manual Power Control

```sh
# Manually prevent sleep (if caffeinate fails)
caffeinate -imsu &

# Allow system to sleep (temporary)
sudo killall caffeinate

# Restart power management
launchctl unload ~/Library/LaunchAgents/caffeinate.plist
launchctl load ~/Library/LaunchAgents/caffeinate.plist
```

## Service Management

### LaunchAgent Control

```sh
# Check service status
launchctl list | grep caffeinate

# Load the service
launchctl load ~/Library/LaunchAgents/caffeinate.plist

# Unload the service
launchctl unload ~/Library/LaunchAgents/caffeinate.plist

# View service details
launchctl print gui/$(id -u)/caffeinate
```

### System Services

```sh
# Check SSH service
sudo launchctl list | grep ssh

# View system startup items
launchctl list | grep -v "^-"

# Check system power settings
systemsetup -getrestartfreeze
systemsetup -getpoweron
systemsetup -getwakeonnetworkaccess
```

## Configuration Management

### Environment Variables

Override configuration for specific deployments:

```sh
# Deploy to different server
make push SERVER_USER=admin SERVER_HOST=production-server

# Deploy with custom settings
SERVER_USER=myuser SERVER_HOST=192.168.1.100 make push

# Check current settings
make check-env
```

### SSH Configuration

```sh
# Test SSH connectivity
ssh -v your-user@your-server

# Copy SSH keys for passwordless access
ssh-copy-id your-user@your-server

# Configure SSH client (in ~/.ssh/config)
Host plan10
    HostName your-server-ip
    User your-username
    IdentityFile ~/.ssh/id_rsa
```

## Application Management

### Available Applications

```sh
# List available applications
make apps

# Deploy specific application (example: Neo4j)
make -C apps/neo4j push

# Check application status
make -C apps/neo4j help
```

### Adding New Applications

1. Create application directory:
   ```sh
   mkdir -p apps/my-app
   ```

2. Add configuration files and README
3. Create Makefile for deployment automation
4. Test deployment process

## Monitoring and Maintenance

### System Health Checks

```sh
# Quick system overview
ssh your-server 'sysmon && echo "=== UPTIME ===" && uptime'

# Check disk space
ssh your-server 'df -h'

# Monitor system load
ssh your-server 'top -l 1 | head -20'

# Check memory usage
ssh your-server 'vm_stat'
```

### Log Monitoring

```sh
# View system logs
ssh your-server 'log show --last 1h --predicate "category == \"power\""'

# Check SSH logs
ssh your-server 'log show --last 1h --predicate "subsystem == \"com.openssh.sshd\""'

# Monitor caffeinate
ssh your-server 'ps aux | grep caffeinate'
```

### Regular Maintenance

**Daily:**
- Check system temperature and battery status
- Verify SSH connectivity
- Monitor system load

**Weekly:**
- Review system logs for errors
- Check available disk space
- Verify backup systems (if configured)

**Monthly:**
- Update macOS system
- Review security settings
- Clean up log files and temporary data

## Remote Access Patterns

### SSH Tunneling

```sh
# Create tunnel for web services
ssh -L 8080:localhost:8080 your-server

# Create tunnel for databases
ssh -L 5432:localhost:5432 your-server

# Create SOCKS proxy
ssh -D 1080 your-server
```

### File Transfer

```sh
# Copy files to server
scp file.txt your-server:~/

# Copy files from server
scp your-server:~/remote-file.txt ./

# Sync directories
rsync -av ./local-dir/ your-server:~/remote-dir/
```

### Remote Command Execution

```sh
# Run single command
ssh your-server 'command'

# Run multiple commands
ssh your-server 'command1 && command2'

# Run script remotely
ssh your-server 'bash -s' < local-script.sh

# Interactive session with specific command
ssh -t your-server 'htop'
```

## Performance Optimization

### System Tuning for Server Use

```sh
# Reduce visual effects (if not done in initial setup)
defaults write com.apple.universalaccess reduceTransparency -bool true
defaults write com.apple.universalaccess reduceMotion -bool true

# Disable unnecessary services
sudo launchctl disable system/com.apple.metadata.mds.scan

# Optimize network settings
sudo sysctl -w net.inet.tcp.delayed_ack=0
```

### Resource Monitoring

```sh
# Monitor CPU usage over time
ssh your-server 'sar -u 1 10'

# Monitor memory usage
ssh your-server 'vm_stat 1'

# Monitor disk I/O
ssh your-server 'iostat -w 1'

# Monitor network traffic
ssh your-server 'nettop -m tcp'
```

## Security Best Practices

### SSH Hardening

1. **Use SSH keys instead of passwords**
2. **Disable root login**
3. **Change default SSH port** (optional)
4. **Use fail2ban or similar** (if available)

### System Security

```sh
# Check firewall status
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

# Enable firewall
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate on

# Check for unauthorized login attempts
ssh your-server 'last | head -20'

# Monitor system integrity
ssh your-server 'sudo fs_usage | head -50'
```

### Regular Security Checks

- Review SSH access logs regularly
- Monitor unusual network activity
- Keep system updated with security patches
- Verify only necessary services are running
- Check for unauthorized user accounts