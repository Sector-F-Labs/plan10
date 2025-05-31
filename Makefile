# Load environment variables from .env file if it exists
-include .env
export

# Configuration variables (with defaults)
SERVER_USER ?= $(USER)
SERVER_HOST ?= plan10

# Remote connection
REMOTE_USER_HOST = $(SERVER_USER)@$(SERVER_HOST)

.PHONY: all clean push push-scripts push-enhanced help setup check-env apps diagnose-remote

help:
	@echo "Plan 10 Server Setup"
	@echo "===================="
	@echo "Available targets:"
	@echo "  setup         - Run interactive setup to create .env file"
	@echo "  push          - Deploy core server configuration"
	@echo "  push-enhanced - Deploy server setup with comprehensive power management"
	@echo "  push-scripts  - Deploy system monitoring scripts and set up aliases"
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

push:
	@echo "🚀 Deploying core server configuration to $(REMOTE_USER_HOST)..."
	@echo "📁 Copying server setup script..."
	scp server_setup.sh $(REMOTE_USER_HOST):~/
	@echo "📁 Copying caffeinate LaunchAgent..."
	scp caffeinate.plist $(REMOTE_USER_HOST):~/Library/LaunchAgents/
	@echo "🔧 Loading LaunchAgent..."
	ssh $(REMOTE_USER_HOST) 'launchctl load ~/Library/LaunchAgents/caffeinate.plist'
	@echo "✅ Core server deployment complete!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. SSH to your server: ssh $(REMOTE_USER_HOST)"
	@echo "  2. Run server setup: sudo ./server_setup.sh"
	@echo "  3. Deploy monitoring scripts: make push-scripts"
	@echo "  4. Configure applications in apps/ directories as needed"

push-enhanced:
	@echo "🚀 Deploying server configuration with power management to $(REMOTE_USER_HOST)..."
	@echo "📁 Copying server setup script..."
	scp server_setup.sh $(REMOTE_USER_HOST):~/
	@echo "📁 Copying power diagnostics script..."
	ssh $(REMOTE_USER_HOST) 'mkdir -p ~/scripts'
	scp scripts/power_diagnostics $(REMOTE_USER_HOST):~/scripts/
	ssh $(REMOTE_USER_HOST) 'chmod +x ~/scripts/power_diagnostics'
	@echo "📁 Copying caffeinate LaunchAgent..."
	scp caffeinate.plist $(REMOTE_USER_HOST):~/Library/LaunchAgents/
	@echo "🔧 Loading LaunchAgent..."
	ssh $(REMOTE_USER_HOST) 'launchctl load ~/Library/LaunchAgents/caffeinate.plist'
	@echo "✅ Server deployment complete!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. SSH to your server: ssh $(REMOTE_USER_HOST)"
	@echo "  2. Run server setup: sudo ./server_setup.sh"
	@echo "  3. Test power settings: ~/scripts/power_diagnostics -a"
	@echo "  4. Deploy monitoring scripts: make push-scripts"
	@echo ""
	@echo "💡 The setup includes comprehensive battery power management"

push-scripts:
	@echo "📊 Deploying system monitoring scripts to $(REMOTE_USER_HOST)..."
	@echo "📁 Creating scripts directory..."
	ssh $(REMOTE_USER_HOST) 'mkdir -p ~/scripts'
	@echo "📁 Copying monitoring scripts..."
	scp scripts/temp $(REMOTE_USER_HOST):~/scripts/
	scp scripts/battery $(REMOTE_USER_HOST):~/scripts/
	scp scripts/power_diagnostics $(REMOTE_USER_HOST):~/scripts/
	scp scripts/setup_aliases.sh $(REMOTE_USER_HOST):~/scripts/
	@echo "🔧 Setting up aliases and permissions..."
	ssh $(REMOTE_USER_HOST) 'cd ~/scripts && chmod +x * && ./setup_aliases.sh'
	@echo "✅ System monitoring scripts deployed!"
	@echo ""
	@echo "Available commands on server:"
	@echo "  temp              - Show system temperature and thermal status"
	@echo "  battery           - Show battery level, charging status, and health"
	@echo "  power_diagnostics - Show power management diagnostics and issues"
	@echo "  sysmon            - Show help for system monitoring tools"
	@echo ""
	@echo "💡 Use 'ssh $(REMOTE_USER_HOST)' and run 'source ~/.zshrc' or open a new session"

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
