# Load environment variables from .env file if it exists
-include .env
export

# Configuration variables (with defaults)
SERVER_USER ?= $(USER)
SERVER_HOST ?= plan10

# Remote connection
REMOTE_USER_HOST = $(SERVER_USER)@$(SERVER_HOST)

.brew install --cask keepingyouawakePHONY: all clean deploy help setup check-env apps diagnose-remote

help:
	@echo "Plan 10 Server Setup"
	@echo "===================="
	@echo "Available targets:"
	@echo "  setup         - Run interactive setup to create .env file"
	@echo "  deploy        - Deploy complete Plan 10 server configuration"
	@echo "  diagnose-remote - Run power diagnostics on remote server"
	@echo "  check-env     - Show current configuration"
	@echo "  apps          - Show available applications"
	@echo "  clean         - Clean temporary files"
	@echo "  help          - Show this help message"
	@echo ""
	@echo "Configuration variables:"
	@echo "  SERVER_USER    = $(SERVER_USER)"
	@echo "  SERVER_HOST    = $(SERVER_HOST)"
	@echo ""
	@echo "Override variables like: make push SERVER_USER=myuser SERVER_HOST=myserver"
	@echo "Or run 'make setup' to create a .env file with your settings"

setup:
	./setup.sh

check-env:
	@echo "Current configuration:"
	@echo "  SERVER_USER    = $(SERVER_USER)"
	@echo "  SERVER_HOST    = $(SERVER_HOST)"
	@echo "  Remote target  = $(REMOTE_USER_HOST)"

apps:
	@echo "Available applications:"
	@echo "======================"
	@if [ -d "apps" ]; then \
		for app in apps/*/; do \
			if [ -d "$$app" ]; then \
				app_name=$$(basename "$$app"); \
				echo "  $$app_name"; \
				if [ -f "$$app/README.md" ]; then \
					echo "    📖 See apps/$$app_name/README.md for setup instructions"; \
				fi; \
				if [ -f "$$app/Makefile" ]; then \
					echo "    🔧 Run 'make -C apps/$$app_name help' for commands"; \
				fi; \
				echo ""; \
			fi; \
		done; \
	else \
		echo "  No applications directory found"; \
	fi

deploy:
	@echo "🚀 Deploying complete Plan 10 server configuration to $(REMOTE_USER_HOST)..."
	@echo ""
	@echo "📁 Copying server setup script..."
	scp server_setup.sh $(REMOTE_USER_HOST):~/
	@echo "📁 Copying caffeinate LaunchAgent..."
	scp caffeinate.plist $(REMOTE_USER_HOST):~/Library/LaunchAgents/
	@echo "📁 Creating scripts directory and copying monitoring tools..."
	ssh $(REMOTE_USER_HOST) 'mkdir -p ~/scripts'
	scp scripts/power_diagnostics $(REMOTE_USER_HOST):~/scripts/
	scp scripts/temp $(REMOTE_USER_HOST):~/scripts/
	scp scripts/battery $(REMOTE_USER_HOST):~/scripts/
	scp scripts/setup_aliases.sh $(REMOTE_USER_HOST):~/scripts/
	@echo "🔧 Setting up permissions and aliases..."
	ssh $(REMOTE_USER_HOST) 'cd ~/scripts && chmod +x * && ./setup_aliases.sh'
	@echo "🔧 Loading LaunchAgent..."
	ssh $(REMOTE_USER_HOST) 'launchctl load ~/Library/LaunchAgents/caffeinate.plist' || echo "  ℹ️  LaunchAgent may already be loaded"
	@echo ""
	@echo "✅ Complete Plan 10 deployment finished!"
	@echo ""
	@echo "🔧 Final setup step - SSH to your server and run:"
	@echo "   ssh $(REMOTE_USER_HOST)"
	@echo "   sudo ./server_setup.sh"
	@echo ""
	@echo "📊 Available monitoring commands on server:"
	@echo "   temp              - System temperature and thermal status"
	@echo "   battery           - Battery level, charging status, and health"
	@echo "   power_diagnostics - Power management diagnostics and fixes"
	@echo "   sysmon            - Help for system monitoring tools"
	@echo ""
	@echo "🔍 Verify deployment:"
	@echo "   make diagnose-remote"
	@echo ""
	@echo "⚠️  Note: Network connectivity may be lost in clamshell mode on battery power"

diagnose-remote:
	@echo "🔌 Running power diagnostics on remote server $(REMOTE_USER_HOST)..."
	@echo "📁 Uploading power diagnostics script..."
	ssh $(REMOTE_USER_HOST) 'mkdir -p ~/scripts'
	scp scripts/power_diagnostics $(REMOTE_USER_HOST):~/scripts/
	@echo "🔧 Setting permissions and running diagnostics..."
	ssh $(REMOTE_USER_HOST) 'chmod +x ~/scripts/power_diagnostics'
	@echo "📊 Executing power diagnostics on $(REMOTE_USER_HOST)..."
	@echo ""
	ssh $(REMOTE_USER_HOST) '~/scripts/power_diagnostics -a'
	@echo ""
	@echo "✅ Remote diagnostics complete!"
	@echo ""
	@echo "💡 To see fixes for identified issues:"
	@echo "   ssh $(REMOTE_USER_HOST) '~/scripts/power_diagnostics -f'"



clean:
	@echo "🧹 Cleaning up temporary files..."
	@echo "Nothing to clean yet."
