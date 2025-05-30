# Load environment variables from .env file if it exists
-include .env
export

# Configuration variables (with defaults)
SERVER_USER ?= $(USER)
SERVER_HOST ?= plan10

# Remote connection
REMOTE_USER_HOST = $(SERVER_USER)@$(SERVER_HOST)

.PHONY: all clean push help setup check-env apps

help:
	@echo "Plan 10 Server Setup"
	@echo "===================="
	@echo "Available targets:"
	@echo "  setup       - Run interactive setup to create .env file"
	@echo "  push        - Deploy core server configuration"
	@echo "  check-env   - Show current configuration"
	@echo "  apps        - Show available applications"
	@echo "  clean       - Clean temporary files"
	@echo "  help        - Show this help message"
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
	@echo "  3. Configure applications in apps/ directories as needed"

clean:
	@echo "🧹 Cleaning up temporary files..."
	@echo "Nothing to clean yet."
