#!/bin/bash

set -e

COMPOSE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$COMPOSE_DIR"

echo "=== Gigi P2P Network Startup Script ==="
echo ""

# Force remove old containers using docker directly (bypassing docker-compose cache)
echo "[0/6] Cleaning up old containers and networks..."
docker rm -f $(docker ps -aq --filter "name=gigi-") 2>/dev/null || true

# Remove old docker-compose project networks
docker network rm gigi-node_public-net 2>/dev/null || true
docker network rm gigi-node_nat-net-1 2>/dev/null || true
docker network rm gigi-node_nat-net-2 2>/dev/null || true
docker network rm Gigi-node_nat-net-3 2>/dev/null || true
docker network rm Gigi-node_public-net 2>/dev/null || true

# Step 1: Start bootstrap node
echo "[1/6] Starting bootstrap node..."
docker-compose up -d bootstrap

# Step 2: Wait for bootstrap node and extract peer ID
echo "[2/6] Waiting for bootstrap node and extracting peer ID..."
sleep 3

BOOTSTRAP_PEER_ID=""
for i in {1..30}; do
    BOOTSTRAP_PEER_ID=$(docker logs gigi-bootstrap 2>&1 | grep -E 'Local peer ID:|local_peer_id' | grep -oP '[0-9A-Za-z]{52}' | head -1)
    if [ -n "$BOOTSTRAP_PEER_ID" ]; then
        break
    fi
    echo "  Waiting for bootstrap peer ID... ($i/30)"
    sleep 1
done

if [ -z "$BOOTSTRAP_PEER_ID" ]; then
    echo "ERROR: Could not extract peer ID from bootstrap node"
    docker logs gigi-bootstrap 2>&1 | tail -20
    docker-compose down
    exit 1
fi

echo "  Bootstrap peer ID: $BOOTSTRAP_PEER_ID"
export BOOTSTRAP_PEER_ID

# Save to .env for docker-compose (needed for relay)
echo "BOOTSTRAP_PEER_ID=$BOOTSTRAP_PEER_ID" > "$COMPOSE_DIR/.env"

# Step 3: Start relay node
echo "[3/6] Starting relay node..."
docker-compose up -d relay

# Step 4: Wait for relay node and extract peer ID
echo "[4/6] Waiting for relay node and extracting peer ID..."
sleep 3

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

echo "  Relay peer ID: $RELAY_PEER_ID"
export RELAY_PEER_ID

# Save to .env file (docker-compose reads from this automatically)
echo "BOOTSTRAP_PEER_ID=$BOOTSTRAP_PEER_ID
RELAY_PEER_ID=$RELAY_PEER_ID" > "$COMPOSE_DIR/.env"
echo "  Peer IDs saved to .env"

# Step 5: Start client nodes
echo "[5/6] Starting client nodes (alice, bob, charlie)..."
docker-compose up -d alice bob charlie

echo ""
echo "=== Network started successfully ==="
echo ""
echo "Bootstrap Peer ID: $BOOTSTRAP_PEER_ID"
echo "Relay Peer ID: $RELAY_PEER_ID"
echo ""
docker-compose ps

echo ""
echo "To view logs:"
echo "  docker-compose logs -f          # All logs"
echo "  docker-compose logs -f bootstrap # Bootstrap logs"
echo ""
echo "To stop:"
echo "  docker-compose down"
