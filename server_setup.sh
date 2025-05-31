#!/bin/bash

echo "⚙️ Plan 10 Server Setup with Battery Power Management"
echo "===================================================="

# Ensure the script is run with root privileges
if [[ "$EUID" -ne 0 ]]; then
  echo "❗ Please run as root: sudo $0"
  exit 1
fi

echo ""
echo "🔋 Configuring battery power management..."

# Prevent system shutdown on battery power
echo "  • Setting battery halt level to 5% (prevents early shutdown)..."
pmset -b haltlevel 5 2>/dev/null || echo "    ℹ️  haltlevel not available on this system"
pmset -b haltafter 0 2>/dev/null || echo "    ℹ️  haltafter not available on this system"

# Keep system awake on both AC and battery power
echo "  • Disabling sleep on AC power..."
pmset -c sleep 0
pmset -c disksleep 0
pmset -c displaysleep 10

echo "  • Disabling sleep on battery power..."
pmset -b sleep 0
pmset -b disksleep 0
pmset -b displaysleep 5

# Disable power management features that can cause shutdowns
echo "  • Disabling hibernation, standby, and power nap..."
pmset -a standby 0
pmset -a hibernatemode 0
pmset -a powernap 0
pmset -a tcpkeepalive 0

# Disable automatic power off
echo "  • Disabling automatic power off..."
pmset -a autopoweroff 0 2>/dev/null || echo "    ℹ️  autopoweroff not available on this system"
pmset -a autopoweroffdelay 0 2>/dev/null || echo "    ℹ️  autopoweroffdelay not available on this system"

# Additional power management settings
echo "  • Applying additional power settings..."
pmset -a sms 0 2>/dev/null || echo "    ℹ️  SMS not available on this system"
pmset -a reducebright 0 2>/dev/null
pmset -a halfdim 0 2>/dev/null

echo ""
echo "⚙️ Configuring system restart and wake settings..."

# Set macOS to auto-restart after freeze or power loss
systemsetup -setrestartfreeze on 2>/dev/null || echo "  ⚠️  Could not set restart on freeze"
systemsetup -setpoweron on 2>/dev/null || echo "  ⚠️  Could not set power on after power loss"
systemsetup -setwakeonnetworkaccess on 2>/dev/null || echo "  ⚠️  Could not set wake on network access"

echo ""
echo "☕ Managing caffeinate process..."

# Check if caffeinate is already running
caffeinate_pids=$(pgrep caffeinate)
if [[ -n $caffeinate_pids ]]; then
  echo "  • Found existing caffeinate processes: $caffeinate_pids"
  echo "  • Killing existing caffeinate processes for clean restart..."
  pkill caffeinate
  sleep 2
fi

echo "  • Starting enhanced caffeinate (prevents idle, system, user, disk sleep)..."
nohup caffeinate -imsud > /dev/null 2>&1 &
sleep 1

# Verify caffeinate started
new_pid=$(pgrep caffeinate)
if [[ -n $new_pid ]]; then
  echo "  ✅ caffeinate started successfully with PID: $new_pid"
else
  echo "  ⚠️  Warning: caffeinate may not have started properly"
fi

echo ""
echo "📊 Current Power Settings Summary:"
echo "=================================="
echo "Sleep settings:"
pmset -g | grep -E "(sleep|disksleep|standby|hibernatemode|powernap|autopoweroff)"

echo ""
echo "Power assertions (what's keeping system awake):"
pmset -g assertions | head -10

echo ""
echo "🔋 Battery Status:"
echo "=================="
pmset -g batt

echo ""
echo "🖥️ Plan 10 server setup complete!"
echo ""
echo "✅ Key configurations applied:"
echo "  • System will NOT shut down when AC power is lost"
echo "  • Battery halt level set to 5% (prevents early shutdown)"
echo "  • Sleep disabled on both AC and battery power"
echo "  • Hibernation, standby, and power nap disabled"
echo "  • Auto power-off disabled"
echo "  • Enhanced caffeinate process running"
echo "  • Auto-restart configured for power loss and system freeze"
echo ""
echo "🔧 Verification commands:"
echo "  • Check power settings: pmset -g"
echo "  • Check caffeinate status: pgrep caffeinate"
echo "  • Check power assertions: pmset -g assertions"
echo "  • Run diagnostics: ~/scripts/power_diagnostics"
echo ""
echo "⚠️  Important notes:"
echo "  • Test power loss in a controlled environment first"
echo "  • Monitor system temperature during extended battery operation"
echo "  • This script is idempotent - safe to run multiple times"
echo "  • Use 'pmset -g assertions' to verify caffeinate is working"