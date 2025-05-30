# Neo4j Setup for Plan 10

Neo4j database server setup and configuration for your Plan 10 server.

## Prerequisites

- Plan 10 server setup completed
- Homebrew installed on the server
- SSH access to the server

## Installation

### On the Server

1. Install Neo4j via Homebrew:
```sh
brew install neo4j
```

2. Create necessary directories and set permissions:
```sh
mkdir -p /usr/local/var/log/neo4j
chmod u+w /usr/local/var/log/neo4j
sudo chown -R $USER /opt/homebrew/var/log/neo4j
sudo chown -R $USER /opt/homebrew/var/neo4j
```

## Configuration

### Deploy Configuration

From your local machine, deploy the Neo4j configuration:

```sh
# Set your Neo4j version
export NEO4J_VERSION="5.15.0"  # Check your installed version

# Deploy configuration
make push-neo4j
```

### Manual Configuration

The `neo4j.conf` file in this directory contains optimized settings for server operation. Key configurations include:

- Import directory restrictions for security
- Memory settings optimized for server use
- Authentication settings (disabled by default for development)

## Service Management

### Start Neo4j
```sh
brew services start neo4j
```

### Stop Neo4j
```sh
brew services stop neo4j
```

### Check Status
```sh
brew services list | grep neo4j
```

### View Logs
```sh
tail -f /opt/homebrew/var/log/neo4j/neo4j.log
```

## Access

Once running, Neo4j will be available at:
- Web Interface: http://your-server:7474
- Bolt Protocol: bolt://your-server:7687

Default credentials (if auth enabled):
- Username: neo4j
- Password: neo4j (change on first login)

## Security

### Enable Authentication
Uncomment this line in `neo4j.conf`:
```
dbms.security.auth_enabled=true
```

### Firewall Configuration
Consider restricting access to Neo4j ports:
```sh
# Example with pfctl (macOS)
sudo pfctl -f /etc/pf.conf
```

### Network Binding
By default, Neo4j only accepts local connections. To allow remote access, configure:
```
server.default_listen_address=0.0.0.0
```

## Troubleshooting

### Check Neo4j Version
```sh
neo4j version
```

### Verify Installation Path
```sh
brew --prefix neo4j
```

### Reset Database
```sh
brew services stop neo4j
rm -rf /opt/homebrew/var/neo4j/data/databases/neo4j
brew services start neo4j
```

### Check Java Version
Neo4j requires Java 17 or later:
```sh
java -version
```

## Configuration Files

- `neo4j.conf`: Main Neo4j configuration file
- Location on server: `/opt/homebrew/Cellar/neo4j/{version}/libexec/conf/neo4j.conf`