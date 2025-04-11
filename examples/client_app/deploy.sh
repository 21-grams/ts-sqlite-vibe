#!/bin/bash
# Simple deployment script for the sensor monitoring system

set -e

echo "Deploying Sensor Monitoring System..."

# Build and start the API server
echo "Starting API server..."
cd "$(dirname "$0")"
docker-compose up -d

# Wait for the API to start
echo "Waiting for API to start..."
sleep 5

# Deploy the client
echo "Deploying client application..."
cd examples/client_app
docker-compose up -d

echo ""
echo "Deployment complete!"
echo "- API server running at: http://localhost:3000"
echo "- Client app running at: http://localhost:8080"
echo ""
echo "Use the following commands to check logs:"
echo "- API logs: docker-compose logs -f sensor-api"
echo "- Client logs: docker-compose -f examples/client_app/docker-compose.yml logs -f client-app"