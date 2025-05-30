#!/bin/bash

echo "⚙️ Setting up persistent server power settings..."

# Ensure the script is run with root privileges
if [[ "$EUID" -ne 0 ]]; then
  echo "❗ Please run as root: sudo $0"
  exit 1
fi

# Set macOS to auto-restart after freeze or power loss
systemsetup -setrestartfreeze on
systemsetup -setpoweron on
systemsetup -setwakeonnetworkaccess on

echo "✅ Power management settings applied."

# Check if caffeinate is already running to avoid duplicates
if pgrep -x "caffeinate" > /dev/null; then
  echo "☕ caffeinate is already running."
else
  echo "☕ Starting caffeinate to prevent sleep (idle, disk, user activity)..."
  nohup caffeinate -imsu > /dev/null 2>&1 &
  echo "✅ caffeinate started."
fi

echo "🖥️ Server setup complete."
