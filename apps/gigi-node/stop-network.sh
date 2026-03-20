#!/bin/bash

set -e

COMPOSE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$COMPOSE_DIR"

echo "=== Gigi P2P Network Stop Script ==="
echo ""

echo "Stopping all services..."
docker-compose down

echo ""
echo "=== Network stopped successfully ==="