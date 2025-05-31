#!/bin/bash
# Run Bezy with debug logging enabled
# Usage: ./debug.sh [additional cargo arguments]

echo "Running Bezy with detailed logging enabled..."
# Suppress macOS IMK messages while keeping app logs
export OS_ACTIVITY_MODE=disable
BEZY_LOG=info cargo run --release "$@" 