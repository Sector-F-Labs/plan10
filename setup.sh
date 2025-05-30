#!/bin/bash

# Plan 10 Local Setup Script
# Sets up the local environment for deploying to a Plan 10 server

set -e

echo "🚀 Plan 10 Local Setup"
echo "====================="

# Default values
DEFAULT_SERVER_USER="$USER"
DEFAULT_SERVER_HOST="plan10"

# Get user input
read -p "Enter server username [$DEFAULT_SERVER_USER]: " SERVER_USER
SERVER_USER=${SERVER_USER:-$DEFAULT_SERVER_USER}

read -p "Enter server hostname [$DEFAULT_SERVER_HOST]: " SERVER_HOST
SERVER_HOST=${SERVER_HOST:-$DEFAULT_SERVER_HOST}

# Create environment file
ENV_FILE=".env"
echo "📝 Creating $ENV_FILE..."

cat > "$ENV_FILE" << EOF
# Plan 10 Configuration
SERVER_USER=$SERVER_USER
SERVER_HOST=$SERVER_HOST
EOF

echo "✅ Configuration saved to $ENV_FILE"

# Test SSH connection
echo "🔍 Testing SSH connection to $SERVER_USER@$SERVER_HOST..."
if ssh -o ConnectTimeout=5 -o BatchMode=yes "$SERVER_USER@$SERVER_HOST" exit 2>/dev/null; then
    echo "✅ SSH connection successful"
else
    echo "⚠️  SSH connection failed. Please ensure:"
    echo "   - SSH keys are set up"
    echo "   - Server is reachable"
    echo "   - Username and hostname are correct"
fi

# Check system requirements
echo "🔍 Checking local requirements..."

if command -v scp &> /dev/null; then
    echo "✅ scp found"
else
    echo "❌ scp not found - required for file deployment"
fi

if command -v ssh &> /dev/null; then
    echo "✅ ssh found"
else
    echo "❌ ssh not found - required for remote commands"
fi

echo ""
echo "🎉 Setup complete!"
echo "📖 Next steps:"
echo "   1. Deploy to server: make push"
echo "   2. SSH to server: ssh $SERVER_USER@$SERVER_HOST"
echo "   3. Run server setup: sudo ./server_setup.sh"
echo "   4. Check apps/ directory for additional applications"