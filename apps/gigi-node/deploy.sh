#!/bin/bash
# Gigi Node Deployment Script
# Usage: ./deploy.sh [bootstrap|relay|full] [host_ip] [identity_path]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
NODE_MODE=${1:-bootstrap}
HOST_IP=${2:-127.0.0.1}
IDENTITY_PATH=${3:-./data/node.key}
BASE_PORT=${BASE_PORT:-4001}

# Validate mode
case $NODE_MODE in
    bootstrap)
        PORT=$BASE_PORT
        echo -e "${GREEN}Deploying Bootstrap Node${NC}"
        ;;
    relay)
        PORT=$((BASE_PORT + 2))
        echo -e "${GREEN}Deploying Relay Node${NC}"
        ;;
    full)
        PORT=$((BASE_PORT + 1))
        echo -e "${GREEN}Deploying Full Node${NC}"
        ;;
    *)
        echo -e "${RED}Invalid mode: $NODE_MODE${NC}"
        echo "Usage: ./deploy.sh [bootstrap|relay|full] [host_ip] [identity_path]"
        exit 1
        ;;
esac

echo -e "${YELLOW}Configuration:${NC}"
echo "  Mode: $NODE_MODE"
echo "  Host IP: $HOST_IP"
echo "  Port: $PORT"
echo "  Identity: $IDENTITY_PATH"

# Create data directory
mkdir -p $(dirname $IDENTITY_PATH)

# Build the node
echo -e "\n${YELLOW}Building gigi-node...${NC}"
cargo build --release -p gigi-node

# Check if binary exists
if [ ! -f "../../target/release/gigi-node" ]; then
    echo -e "${RED}Build failed: binary not found${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful!${NC}"

# Create systemd service file
SERVICE_NAME="gigi-node-$NODE_MODE"
SERVICE_FILE="/tmp/$SERVICE_NAME.service"

cat > $SERVICE_FILE << EOF
[Unit]
Description=Gigi Node ($NODE_MODE)
After=network.target

[Service]
Type=simple
User=gigi
WorkingDirectory=/opt/gigi
ExecStart=/opt/gigi/gigi-node \\
  --mode $NODE_MODE \\
  --listen /ip4/0.0.0.0/tcp/$PORT \\
  --listen /ip4/0.0.0.0/udp/$PORT/quic-v1 \\
  --external /ip4/$HOST_IP/tcp/$PORT \\
  --external /ip4/$HOST_IP/udp/$PORT/quic-v1 \\
  --identity $IDENTITY_PATH
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

echo -e "\n${YELLOW}Systemd service file created: $SERVICE_FILE${NC}"
cat $SERVICE_FILE

# Create Docker deployment
cat > Dockerfile.$NODE_MODE << EOF
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY ../../target/release/gigi-node /usr/local/bin/
RUN chmod +x /usr/local/bin/gigi-node

# Create data directory
RUN mkdir -p /data

EXPOSE $PORT/tcp $PORT/udp

ENTRYPOINT ["gigi-node"]
CMD [
  "--mode", "$NODE_MODE",
  "--listen", "/ip4/0.0.0.0/tcp/$PORT",
  "--listen", "/ip4/0.0.0.0/udp/$PORT/quic-v1",
  "--external", "/ip4/$HOST_IP/tcp/$PORT",
  "--external", "/ip4/$HOST_IP/udp/$PORT/quic-v1",
  "--identity", "$IDENTITY_PATH"
]
EOF

echo -e "\n${YELLOW}Dockerfile created: Dockerfile.$NODE_MODE${NC}"

# Create docker-compose file
cat > docker-compose.$NODE_MODE.yml << EOF
version: '3.8'

services:
  gigi-node-$NODE_MODE:
    build:
      context: ../..
      dockerfile: apps/gigi-node/Dockerfile.$NODE_MODE
    container_name: gigi-node-$NODE_MODE
    ports:
      - "$PORT:$PORT/tcp"
      - "$PORT:$PORT/udp"
    volumes:
      - ./data:/data
    restart: unless-stopped
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
EOF

echo -e "${YELLOW}Docker Compose file created: docker-compose.$NODE_MODE.yml${NC}"

# Create deployment instructions
cat > DEPLOY.md << EOF
# Gigi Node Deployment Guide

## Quick Start

### Option 1: Direct Binary Execution

\`\`\`bash
# Run the node directly
./target/release/gigi-node \\
  --mode $NODE_MODE \\
  --listen /ip4/0.0.0.0/tcp/$PORT \\
  --listen /ip4/0.0.0.0/udp/$PORT/quic-v1 \\
  --external /ip4/$HOST_IP/tcp/$PORT \\
  --identity $IDENTITY_PATH
\`\`\`

### Option 2: Systemd Service

\`\`\`bash
# Copy binary
sudo cp ../../target/release/gigi-node /usr/local/bin/
sudo chmod +x /usr/local/bin/gigi-node

# Install service
sudo cp $SERVICE_FILE /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable $SERVICE_NAME
sudo systemctl start $SERVICE_NAME

# Check status
sudo systemctl status $SERVICE_NAME
\`\`\`

### Option 3: Docker

\`\`\`bash
# Build and run
docker-compose -f docker-compose.$NODE_MODE.yml up -d

# Check logs
docker-compose -f docker-compose.$NODE_MODE.yml logs -f
\`\`\`

## Firewall Configuration

Ensure these ports are open:
- TCP $PORT
- UDP $PORT

\`\`\`bash
# UFW (Ubuntu)
sudo ufw allow $PORT/tcp
sudo ufw allow $PORT/udp

# FirewallD (CentOS/RHEL)
sudo firewall-cmd --permanent --add-port=$PORT/tcp
sudo firewall-cmd --permanent --add-port=$PORT/udp
sudo firewall-cmd --reload
\`\`\`

## Monitoring

### Check Node Status

\`\`\`bash
# If running with systemd
sudo journalctl -u $SERVICE_NAME -f

# If running with Docker
docker-compose -f docker-compose.$NODE_MODE.yml logs -f
\`\`\`

### Get Peer ID

\`\`\`bash
# The peer ID is logged on startup
grep "Local peer ID" /var/log/gigi-node.log
\`\`\`

## Troubleshooting

### Node won't start
- Check if port is already in use: \`netstat -tlnp | grep $PORT\`
- Verify identity file permissions: \`ls -la $IDENTITY_PATH\`

### Can't connect to bootstrap nodes
- Verify network connectivity: \`telnet bootstrap_ip $BASE_PORT\`
- Check firewall rules

### High memory usage
- Limit connections in config
- Enable connection limits in systemd service
EOF

echo -e "\n${GREEN}Deployment files created successfully!${NC}"
echo -e "${YELLOW}Files generated:${NC}"
echo "  - $SERVICE_FILE (systemd service)"
echo "  - Dockerfile.$NODE_MODE (Docker image)"
echo "  - docker-compose.$NODE_MODE.yml (Docker Compose)"
echo "  - DEPLOY.md (deployment guide)"

echo -e "\n${GREEN}Next steps:${NC}"
echo "1. Copy binary to server: scp ../../target/release/gigi-node user@$HOST_IP:/opt/gigi/"
echo "2. Deploy using systemd, Docker, or direct execution"
echo "3. Configure firewall to open port $PORT"
echo "4. Update DNS to point bootstrap.gigi.network to $HOST_IP"
