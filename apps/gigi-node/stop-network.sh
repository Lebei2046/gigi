#!/bin/bash

set -e

COMPOSE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$COMPOSE_DIR"

echo "=== Gigi P2P Network Stop Script ==="
echo ""

# Restore original docker-compose.yml
echo "Restoring original docker-compose.yml..."
git checkout docker-compose.yml 2>/dev/null || echo "No git repository found, docker-compose.yml may have been modified"

echo "Stopping all services..."
docker-compose down

echo ""
echo "=== Network stopped successfully ==="