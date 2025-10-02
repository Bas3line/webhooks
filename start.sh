#!/bin/bash

echo "Starting Webhook Service..."
echo "=========================="

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo "Error: .env file not found!"
    echo "Please create a .env file with required environment variables:"
    echo "  DATABASE_URL=postgresql://username:password@host:port/database"
    echo "  BIND_ADDRESS=0.0.0.0:5050"
    echo "  BASE_URL=http://localhost:5050"
    echo "  RUST_LOG=info"
    echo "  SQLX_OFFLINE=true"
    exit 1
fi

# Load environment variables
set -a
source .env
set +a

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo/Rust not found!"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Building application..."
if ! cargo build --release; then
    echo "Error: Build failed!"
    echo "Trying to build in offline mode..."
    export SQLX_OFFLINE=true
    if ! cargo build --release; then
        echo "Error: Build failed even in offline mode!"
        exit 1
    fi
fi

echo "Starting webhook service on port 5050..."
echo "Frontend available at: http://localhost:5050"
echo "Health check: http://localhost:5050/health"
echo "Create webhooks at: http://localhost:5050"
echo ""
echo "API Endpoints:"
echo "  POST /webhook/{endpoint} - Receive webhooks"
echo "  GET  /api/webhooks       - List webhooks"
echo "  POST /api/webhooks       - Create webhook"
echo ""
echo "Press Ctrl+C to stop the service"
echo "=========================="

# Run the application
cargo run --release