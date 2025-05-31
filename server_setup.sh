#!/bin/bash

echo "⚙️ Plan 10 Server Setup"
echo "======================"

# Ensure the script is run with root privileges
if [[ "$EUID" -ne 0 ]]; then
  echo "❗ Please run as root: sudo $0"
  exit 1
fi

echo ""
echo "🔋 Configuring power management..."

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

# Basic network connectivity settings
echo "  • Enabling basic network connectivity..."
pmset -a tcpkeepalive 1
pmset -a womp 1

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

echo "  • Starting caffeinate (prevents idle, system, user sleep)..."
nohup caffeinate -imsu > /dev/null 2>&1 &
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
pmset -g | head -15

echo ""
echo "🔋 Battery Status:"
echo "=================="
pmset -g batt

echo ""
echo "🖥️ Plan 10 server setup complete!"
echo ""
echo "✅ Key configurations applied:"
echo "  • Sleep disabled on both AC and battery power"
echo "  • Hibernation and standby disabled"
echo "  • Wake-on-LAN enabled"
echo "  • Caffeinate process running"
echo "  • Auto-restart configured for power loss and system freeze"
echo ""
echo "⚠️  Known limitation:"
echo "  • Network connectivity may be lost when running on battery with lid closed (clamshell mode)"
echo "  • For reliable battery backup, keep lid open or use external display"
echo ""
echo "🔧 Verification commands:"
echo "  • Check power settings: pmset -g"
echo "  • Check caffeinate status: pgrep caffeinate"
echo "  • Check power assertions: pmset -g assertions"
echo "  • Run diagnostics: ~/scripts/power_diagnostics"