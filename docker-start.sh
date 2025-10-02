#!/bin/bash

echo "Starting Webhook Service with Docker..."
echo "====================================="
echo ""
echo "Building and starting containers..."
docker-compose up --build

echo ""
echo "To stop the service, run: docker-compose down"
echo "To view logs, run: docker-compose logs -f webhook-service"
echo ""
echo "Frontend available at: http://localhost:5050"
echo "Health check: http://localhost:5050/health"
