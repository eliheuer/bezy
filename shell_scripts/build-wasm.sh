#!/bin/bash

# Build script for Bezy WASM version

echo "🛠️  Building Bezy for WASM..."

# Build for WASM target
cargo build --target wasm32-unknown-unknown --release

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "🌐 Generated files:"
    echo "   - target/wasm32-unknown-unknown/release/bezy.wasm"
    echo "   - index.html (already in root)"
    echo ""
    echo "📦 To serve the WASM build:"
    echo "   1. Copy the generated .wasm file to your web server"
    echo "   2. Also copy the generated .js file (if any)"
    echo "   3. Serve index.html from a web server"
    echo ""
    echo "🔍 For development, run:"
    echo "   cargo run --target wasm32-unknown-unknown"
else
    echo "❌ Build failed! Check the error messages above."
    exit 1
fi 