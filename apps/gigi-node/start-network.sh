#!/bin/bash

set -e

COMPOSE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$COMPOSE_DIR"

echo "=== Gigi P2P Network Startup Script ==="
echo ""

# Check if bootstrap peer ID already exists
if [ -f "$COMPOSE_DIR/.bootstrap_peer_id" ]; then
    PEER_ID=$(cat "$COMPOSE_DIR/.bootstrap_peer_id")
    echo "[1/5] Bootstrap peer ID found: $PEER_ID"
else
    # Step 1: Start only the bootstrap node
    echo "[1/5] Starting bootstrap node..."
    docker-compose up -d bootstrap

    # Step 2: Wait for bootstrap node to be ready
    echo "[2/5] Waiting for bootstrap node to be ready..."
    sleep 3

    # Step 3: Extract peer ID from bootstrap node
    echo "[3/5] Extracting peer ID from bootstrap node..."
    PEER_ID=""
    for i in {1..30}; do
        PEER_ID=$(docker logs gigi-bootstrap 2>&1 | grep -E 'Local peer ID:|local_peer_id' | grep -oP '[0-9A-Za-z]{52}' | head -1)
        if [ -n "$PEER_ID" ]; then
            break
        fi
        echo "  Waiting for peer ID... ($i/30)"
        sleep 1
    done

    if [ -z "$PEER_ID" ]; then
        echo "ERROR: Could not extract peer ID from bootstrap node"
        docker logs gigi-bootstrap 2>&1 | tail -20
        docker-compose down
        exit 1
    fi

    echo "  Found peer ID: $PEER_ID"

    # Save peer ID for future runs
    echo "$PEER_ID" > "$COMPOSE_DIR/.bootstrap_peer_id"
    echo "  Peer ID saved to .bootstrap_peer_id"
fi

# Export for docker-compose
export BOOTSTRAP_PEER_ID="$PEER_ID"

# Step 4: Start relay nodes
echo "[4/5] Starting relay nodes..."
docker-compose up -d relay

# Step 5: Wait for relay nodes to connect to bootstrap
echo "[5/5] Waiting for relay nodes to connect..."
sleep 3

# Extract relay peer ID
echo "[5/5] Extracting relay peer ID..."
RELAY_PEER_ID=""
for i in {1..30}; do
    RELAY_PEER_ID=$(docker logs gigi-relay 2>&1 | grep -E 'Local peer ID:|local_peer_id' | grep -oP '[0-9A-Za-z]{52}' | head -1)
    if [ -n "$RELAY_PEER_ID" ]; then
        break
    fi
    echo "  Waiting for relay peer ID... ($i/30)"
    sleep 1
done

if [ -z "$RELAY_PEER_ID" ]; then
    echo "ERROR: Could not extract peer ID from relay node"
    docker logs gigi-relay 2>&1 | tail -20
    docker-compose down
    exit 1
fi

echo "  Found relay peer ID: $RELAY_PEER_ID"

# Export for docker-compose
export BOOTSTRAP_PEER_ID="$PEER_ID"
export RELAY_PEER_ID="$RELAY_PEER_ID"

# Stop relay and restart with env variable
echo "[5/5] Restarting relay with peer ID..."
docker-compose stop relay
docker-compose rm -f relay
docker-compose up -d relay

sleep 2

# Step 6: Start receiver and sender nodes
echo "[6/6] Starting receiver and sender nodes..."
docker-compose up -d bob charlie alice

echo ""
echo "=== Network started successfully ==="
echo ""
echo "Bootstrap Peer ID: $PEER_ID"
echo ""
docker-compose ps

echo ""
echo "To view logs:"
echo "  docker-compose logs -f          # All logs"
echo "  docker-compose logs -f bootstrap # Bootstrap logs"
echo ""
echo "To stop:"
echo "  docker-compose down"
echo ""
echo "To reset (delete bootstrap identity):"
echo "  rm .bootstrap_peer_id"
echo "  docker-compose down -v"
