#!/bin/bash

# Simple hot reloading development script for SnapFileThing
# This script watches for changes in Rust source files and automatically rebuilds and restarts the server

echo "🦀 Starting SnapFileThing with hot reloading..."
echo "🌐 Server will be available at http://localhost:8080"
echo "📁 Static files at http://localhost:8081"
echo "⌨️  Press Ctrl+C to stop"
echo ""

# Use cargo-watch to watch for changes and restart the server
cargo watch \
    --clear \
    --watch src \
    --watch Cargo.toml \
    --exec "run --bin snapfilething"
