#!/bin/bash

# Setup script for system monitoring aliases
# This script adds temperature and battery monitoring aliases to the shell profile

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHELL_PROFILE=""

# Detect shell and profile file
if [[ "$SHELL" == *"zsh"* ]]; then
    SHELL_PROFILE="$HOME/.zshrc"
elif [[ "$SHELL" == *"bash"* ]]; then
    if [[ -f "$HOME/.bash_profile" ]]; then
        SHELL_PROFILE="$HOME/.bash_profile"
    else
        SHELL_PROFILE="$HOME/.bashrc"
    fi
else
    echo "⚠️  Unknown shell: $SHELL"
    echo "Please manually add aliases to your shell profile"
    exit 1
fi

echo "🔧 Setting up system monitoring aliases..."
echo "Shell: $SHELL"
echo "Profile: $SHELL_PROFILE"

# Create backup of shell profile
if [[ -f "$SHELL_PROFILE" ]]; then
    cp "$SHELL_PROFILE" "${SHELL_PROFILE}.backup.$(date +%Y%m%d_%H%M%S)"
    echo "📋 Created backup: ${SHELL_PROFILE}.backup.$(date +%Y%m%d_%H%M%S)"
fi

# Check if aliases already exist
if grep -q "# Plan 10 System Monitoring" "$SHELL_PROFILE" 2>/dev/null; then
    echo "⚠️  Plan 10 aliases already exist in $SHELL_PROFILE"
    echo "Skipping to avoid duplicates"
    exit 0
fi

# Add aliases to shell profile
cat >> "$SHELL_PROFILE" << EOF

# Plan 10 System Monitoring
# =======================
alias temp='$SCRIPT_DIR/temp'
alias battery='$SCRIPT_DIR/battery'
alias sysmon='echo "🖥️  System Monitoring Tools"; echo "========================="; echo "temp     - Show system temperature"; echo "battery  - Show battery status"; echo ""; echo "Use -h flag for help with each command"'

EOF

echo "✅ Aliases added to $SHELL_PROFILE"
echo ""
echo "Available commands:"
echo "  temp     - Show system temperature and thermal status"
echo "  battery  - Show battery level, charging status, and health"
echo "  sysmon   - Show help for system monitoring tools"
echo ""
echo "🔄 To use immediately, run: source $SHELL_PROFILE"
echo "   Or open a new terminal session"