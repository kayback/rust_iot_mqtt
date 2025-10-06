#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$PROJECT_ROOT/deploy/docker-compose.yml"

echo "Starting required Docker containers..."

if ! command -v docker-compose &> /dev/null; then
    echo "ERROR: docker-compose not found. Please install docker-compose."
    exit 1
fi

if ! docker info &> /dev/null; then
    echo "ERROR: Docker is not running. Please start Docker daemon."
    exit 1
fi

if [ ! -f "$COMPOSE_FILE" ]; then
    echo "ERROR: docker-compose.yml not found at $COMPOSE_FILE"
    exit 1
fi

echo "Starting PostgreSQL, Mosquitto, Prometheus, and Grafana..."
cd "$PROJECT_ROOT/deploy"
docker-compose up -d postgres mosquitto prometheus grafana

echo "Waiting for services to initialize (5 seconds)..."
sleep 5

echo "Verifying services..."
REQUIRED_SERVICES=("iot-postgres" "iot-mosquitto" "iot-prometheus" "iot-grafana")
ALL_RUNNING=true

for service in "${REQUIRED_SERVICES[@]}"; do
    if docker ps --format '{{.Names}}' | grep -q "^${service}$"; then
        echo " $service is running"
    else
        echo " $service is NOT running"
        ALL_RUNNING=false
    fi
done

if [ "$ALL_RUNNING" = false ]; then
    echo ""
    echo "WARNING: Some services failed to start. Check docker-compose logs:"
    echo "  docker-compose -f $COMPOSE_FILE logs"
    exit 1
fi

echo ""
echo "All required services are running"
echo ""
echo "Service status:"
echo "  PostgreSQL:  localhost:5432"
echo "  Mosquitto:   localhost:1883"
echo "  Prometheus:  http://localhost:9090"
echo "  Grafana:     http://localhost:3000 (admin/admin)"
echo ""

exit 0

