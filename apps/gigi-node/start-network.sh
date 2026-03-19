#!/bin/bash

set -e

COMPOSE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$COMPOSE_DIR"

echo "=== Gigi P2P Network Startup Script ==="
echo ""

# Step 1: Start only the bootstrap node
echo "[1/4] Starting bootstrap node..."
docker-compose up -d bootstrap

# Step 2: Wait for bootstrap node to be ready and extract peer ID
echo "[2/4] Waiting for bootstrap node to be ready..."
sleep 5

echo "[3/4] Extracting peer ID from bootstrap node..."
# Wait for peer ID to appear in logs (max 30 seconds)
PEER_ID=""
for i in {1..30}; do
    # Extract peer ID using multiple patterns for robustness
    PEER_ID=$(docker logs gigi-bootstrap 2>&1 | grep -E 'Local peer ID:|local_peer_id' | grep -oP '[0-9A-Za-z]{52}' | head -1)
    if [ -n "$PEER_ID" ]; then
        break
    fi
    echo "  Waiting for peer ID... ($i/30)"
    sleep 1
done

if [ -z "$PEER_ID" ]; then
    echo "ERROR: Could not extract peer ID from bootstrap node"
    echo "Checking bootstrap logs for debugging:"
    docker logs gigi-bootstrap 2>&1 | tail -20
    docker-compose down
    exit 1
fi

echo "  Found peer ID: $PEER_ID"

# Step 3: Update docker-compose.yml with the correct peer ID
echo "[4/4] Updating docker-compose.yml with peer ID..."
sed -i "s/QmPLACEHOLDER/$PEER_ID/g" docker-compose.yml

echo ""
echo "=== Starting all services ==="
docker-compose up -d

echo ""
echo "=== Network started successfully ==="
echo ""
docker-compose ps

echo ""
echo "To view logs:"
echo "  docker-compose logs -f          # All logs"
echo "  docker-compose logs -f bootstrap # Bootstrap logs"
echo ""
echo "To stop:"
echo "  docker-compose down"