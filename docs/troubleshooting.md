# Plan 10 Troubleshooting Guide

Common issues and solutions for Plan 10 server operation.

## SSH Connection Issues

### Cannot Connect to Server

**Symptoms:**
- `ssh: connect to host [hostname] port 22: Connection refused`
- `ssh: Could not resolve hostname [hostname]`
- Connection timeouts

**Solutions:**

1. **Verify SSH Service is Running**
   ```sh
   # On the server (local access required)
   sudo launchctl list | grep ssh
   sudo systemsetup -getremotelogin
   ```

2. **Check Network Connectivity**
   ```sh
   # Test basic connectivity
   ping your-server-ip
   
   # Test SSH port specifically
   telnet your-server-ip 22
   nc -zv your-server-ip 22
   ```

3. **Verify SSH Configuration**
   ```sh
   # Check SSH daemon configuration
   sudo sshd -T | grep -i port
   sudo sshd -T | grep -i permitrootlogin
   
   # Restart SSH service
   sudo launchctl unload -w /System/Library/LaunchDaemons/ssh.plist
   sudo launchctl load -w /System/Library/LaunchDaemons/ssh.plist
   ```

4. **Firewall Issues**
   ```sh
   # Check firewall status
   sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
   
   # Temporarily disable firewall for testing
   sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate off
   
   # Re-enable after testing
   sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate on
   ```

### Authentication Failures

**Symptoms:**
- `Permission denied (publickey)`
- `Authentication failed`
- Password prompts when expecting key auth

**Solutions:**

1. **SSH Key Issues**
   ```sh
   # Verify key exists on client
   ls -la ~/.ssh/
   
   # Check key permissions
   chmod 600 ~/.ssh/id_rsa
   chmod 644 ~/.ssh/id_rsa.pub
   
   # Test key loading
   ssh-add -l
   ssh-add ~/.ssh/id_rsa
   ```

2. **Server-side Key Configuration**
   ```sh
   # Check authorized_keys file
   ls -la ~/.ssh/authorized_keys
   chmod 600 ~/.ssh/authorized_keys
   chmod 700 ~/.ssh/
   
   # Verify key content
   cat ~/.ssh/authorized_keys
   ```

3. **SSH Client Debugging**
   ```sh
   # Connect with verbose output
   ssh -v your-user@your-server
   ssh -vv your-user@your-server  # More verbose
   ssh -vvv your-user@your-server  # Maximum verbosity
   ```

## Power Management Issues

### System Goes to Sleep

**Symptoms:**
- SSH connections drop after inactivity
- Server becomes unresponsive
- Need to physically wake the machine

**Solutions:**

1. **Check Caffeinate Status**
   ```sh
   # Verify caffeinate is running
   pgrep -x "caffeinate"
   ps aux | grep caffeinate
   
   # Check what's preventing sleep
   pmset -g assertions
   ```

2. **Restart Caffeinate Service**
   ```sh
   # Unload and reload LaunchAgent
   launchctl unload ~/Library/LaunchAgents/caffeinate.plist
   launchctl load ~/Library/LaunchAgents/caffeinate.plist
   
   # Verify service is loaded
   launchctl list | grep caffeinate
   ```

3. **Manual Caffeinate**
   ```sh
   # Start caffeinate manually
   caffeinate -imsu &
   
   # Kill existing caffeinate processes
   sudo killall caffeinate
   ```

4. **Check Power Settings**
   ```sh
   # View current power configuration
   pmset -g
   
   # Check system power settings
   systemsetup -getcomputersleep
   systemsetup -getdisplaysleep
   systemsetup -getharddisksleep
   ```

### Auto Login Not Working

**Symptoms:**
- System requires manual login after reboot
- Server inaccessible after power loss
- Login screen appears on restart

**Solutions:**

1. **Verify Auto Login Settings**
   ```sh
   # Check auto login configuration
   sudo defaults read /Library/Preferences/com.apple.loginwindow
   
   # Enable auto login (replace USERNAME)
   sudo defaults write /Library/Preferences/com.apple.loginwindow autoLoginUser -string "USERNAME"
   ```

2. **FileVault Compatibility**
   ```sh
   # Check if FileVault is enabled (incompatible with auto login)
   fdesetup status
   
   # Disable FileVault if necessary (WARNING: Decrypt disk first)
   sudo fdesetup disable
   ```

3. **User Account Issues**
   ```sh
   # Verify user exists and is admin
   dscl . -read /Users/$(whoami) | grep -A5 -B5 -i admin
   
   # Check user login restrictions
   sudo pwpolicy -u $(whoami) -getpolicy
   ```

## Monitoring Script Issues

### Temperature Script Problems

**Symptoms:**
- `temp` command not found
- Permission denied errors
- Inaccurate temperature readings

**Solutions:**

1. **Script Installation**
   ```sh
   # Verify script exists and is executable
   ls -la ~/scripts/temp
   chmod +x ~/scripts/temp
   
   # Check if aliases are set up
   alias temp
   which temp
   ```

2. **Permission Issues**
   ```sh
   # Run with sudo for detailed temperature data
   sudo ~/scripts/temp
   
   # Check system temperature access
   sudo powermetrics --samplers smc -n 1 -i 1000
   ```

3. **Script Debugging**
   ```sh
   # Run script with debug output
   bash -x ~/scripts/temp
   
   # Check dependencies
   which powermetrics
   which system_profiler
   ```

### Battery Script Problems

**Symptoms:**
- No battery information displayed
- Script reports "No battery found"
- Incorrect battery status

**Solutions:**

1. **Desktop Mac Without Battery**
   ```sh
   # Check if system has a battery
   pmset -g batt
   system_profiler SPPowerDataType | grep -i battery
   ```

2. **Battery Detection Issues**
   ```sh
   # Reset power management
   sudo pmset -a standby 0
   sudo pmset -a standbydelay 0
   
   # Check battery calibration
   pmset -g rawlog | grep -i battery
   ```

## Application Deployment Issues

### Neo4j Setup Problems

**Symptoms:**
- Neo4j won't start
- Connection refused on port 7474/7687
- Permission denied errors

**Solutions:**

1. **Installation Verification**
   ```sh
   # Check Neo4j installation
   brew list neo4j
   neo4j version
   
   # Verify installation path
   brew --prefix neo4j
   ```

2. **Service Management**
   ```sh
   # Check service status
   brew services list | grep neo4j
   
   # Start/stop service
   brew services stop neo4j
   brew services start neo4j
   
   # View logs
   tail -f /opt/homebrew/var/log/neo4j/neo4j.log
   ```

3. **Configuration Issues**
   ```sh
   # Check configuration file
   cat /opt/homebrew/Cellar/neo4j/*/libexec/conf/neo4j.conf
   
   # Verify permissions
   ls -la /opt/homebrew/var/neo4j/
   sudo chown -R $(whoami) /opt/homebrew/var/neo4j/
   ```

## Network Configuration Issues

### IP Address Changes

**Symptoms:**
- Cannot connect after network changes
- SSH connects to wrong machine
- DNS resolution failures

**Solutions:**

1. **Find Current IP Address**
   ```sh
   # On the server
   ifconfig | grep "inet " | grep -v 127.0.0.1
   
   # Check network interface status
   networksetup -listallhardwareports
   networksetup -getinfo "Wi-Fi"
   ```

2. **Update SSH Configuration**
   ```sh
   # Update ~/.ssh/config
   Host plan10
       HostName new-ip-address
       User your-username
   
   # Test new configuration
   ssh plan10
   ```

3. **Static IP Configuration**
   ```sh
   # Set static IP (adjust for your network)
   sudo networksetup -setmanual "Wi-Fi" 192.168.1.100 255.255.255.0 192.168.1.1
   
   # Set DNS servers
   sudo networksetup -setdnsservers "Wi-Fi" 8.8.8.8 8.8.4.4
   ```

## System Performance Issues

### High CPU Usage

**Symptoms:**
- System running hot
- Fan noise increased
- Slow response times

**Solutions:**

1. **Identify Resource-Heavy Processes**
   ```sh
   # Monitor CPU usage
   top -o cpu
   
   # Check for runaway processes
   ps aux --sort=-%cpu | head -10
   
   # Monitor system activity
   sudo fs_usage | head -50
   ```

2. **System Optimization**
   ```sh
   # Disable unnecessary services
   sudo launchctl disable system/com.apple.metadata.mds.scan
   
   # Clear system caches
   sudo purge
   
   # Check disk space
   df -h
   ```

### Memory Issues

**Symptoms:**
- System slowdown
- Frequent disk activity
- "Out of memory" errors

**Solutions:**

1. **Memory Monitoring**
   ```sh
   # Check memory usage
   vm_stat
   
   # Monitor memory pressure
   memory_pressure
   
   # View memory hogs
   top -o mem
   ```

2. **Memory Cleanup**
   ```sh
   # Clear inactive memory
   sudo purge
   
   # Restart memory-heavy services
   sudo launchctl kickstart -k system/com.apple.WindowServer
   ```

## Recovery Procedures

### Complete System Recovery

**When to use:** System completely unresponsive, cannot SSH, caffeinate not working.

1. **Physical Access Recovery**
   ```sh
   # Connect monitor and keyboard
   # Boot into Recovery Mode: Hold Cmd+R during startup
   # Access Terminal and check system
   ```

2. **Reset Power Management**
   ```sh
   # Reset SMC (System Management Controller)
   # Shut down, press Shift+Ctrl+Opt+Power for 10 seconds
   # Release and start normally
   ```

3. **Safe Mode Boot**
   ```sh
   # Boot holding Shift key
   # Check system in safe mode
   # Restart normally
   ```

### Reinstall Plan 10 Components

**When to use:** Partial system failure, some components working.

1. **Redeploy Core Configuration**
   ```sh
   # From local machine
   make push
   
   # SSH to server and run setup
   ssh your-server
   sudo ./server_setup.sh
   ```

2. **Reinstall Monitoring Scripts**
   ```sh
   # From local machine
   make push-scripts
   ```

3. **Reset LaunchAgent**
   ```sh
   # On server
   launchctl unload ~/Library/LaunchAgents/caffeinate.plist
   rm ~/Library/LaunchAgents/caffeinate.plist
   
   # Redeploy from local machine
   make push
   ```

## Getting Help

### Diagnostic Information to Collect

Before seeking help, gather this information:

1. **System Information**
   ```sh
   sw_vers
   uname -a
   uptime
   ```

2. **Network Configuration**
   ```sh
   ifconfig
   netstat -rn
   ```

3. **Service Status**
   ```sh
   launchctl list | grep caffeinate
   pgrep -x "caffeinate"
   pmset -g assertions
   ```

4. **Error Logs**
   ```sh
   log show --last 1h --predicate "category == \"power\""
   dmesg | tail -50
   ```

### Log File Locations

- System logs: `/var/log/system.log`
- SSH logs: `log show --predicate "subsystem == 'com.openssh.sshd'"`
- Power management: `log show --predicate "category == 'power'"`
- LaunchAgent logs: `~/Library/Logs/`

### Common Error Messages

| Error | Meaning | Solution |
|-------|---------|----------|
| `Connection refused` | SSH service not running | Enable Remote Login in System Settings |
| `No route to host` | Network connectivity issue | Check IP address and network settings |
| `Permission denied` | Authentication failure | Check SSH keys and user permissions |
| `caffeinate: command not found` | System command missing | Reinstall Xcode Command Line Tools |
| `launchctl: no such process` | Service not loaded | Reload LaunchAgent configuration |