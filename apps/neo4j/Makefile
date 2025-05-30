# Load environment variables from .env file if it exists
-include ../../.env
export

# Configuration variables (with defaults)
SERVER_USER ?= $(USER)
SERVER_HOST ?= plan10
NEO4J_VERSION ?= 5.15.0

# Derived paths
NEO4J_CONF_PATH = /opt/homebrew/Cellar/neo4j/$(NEO4J_VERSION)/libexec/conf/neo4j.conf
REMOTE_USER_HOST = $(SERVER_USER)@$(SERVER_HOST)

.PHONY: deploy status start stop restart logs help

help:
	@echo "Neo4j Deployment for Plan 10"
	@echo "============================="
	@echo "Available targets:"
	@echo "  deploy   - Deploy Neo4j configuration to server"
	@echo "  start    - Start Neo4j service on server"
	@echo "  stop     - Stop Neo4j service on server"
	@echo "  restart  - Restart Neo4j service on server"
	@echo "  status   - Check Neo4j service status"
	@echo "  logs     - View Neo4j logs"
	@echo "  help     - Show this help message"
	@echo ""
	@echo "Configuration:"
	@echo "  SERVER_USER    = $(SERVER_USER)"
	@echo "  SERVER_HOST    = $(SERVER_HOST)"
	@echo "  NEO4J_VERSION  = $(NEO4J_VERSION)"

deploy:
	@echo "🚀 Deploying Neo4j configuration to $(REMOTE_USER_HOST)..."
	scp neo4j.conf $(REMOTE_USER_HOST):$(NEO4J_CONF_PATH)
	@echo "✅ Neo4j configuration deployed!"

start:
	@echo "▶️  Starting Neo4j service..."
	ssh $(REMOTE_USER_HOST) 'brew services start neo4j'
	@echo "✅ Neo4j service started!"

stop:
	@echo "⏹️  Stopping Neo4j service..."
	ssh $(REMOTE_USER_HOST) 'brew services stop neo4j'
	@echo "✅ Neo4j service stopped!"

restart: stop start

status:
	@echo "📊 Checking Neo4j service status..."
	ssh $(REMOTE_USER_HOST) 'brew services list | grep neo4j'

logs:
	@echo "📋 Viewing Neo4j logs..."
	ssh $(REMOTE_USER_HOST) 'tail -f /opt/homebrew/var/log/neo4j/neo4j.log'