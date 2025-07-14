#!/bin/bash

# Build script for GitHub Pages deployment
# This script builds the WASM version and prepares static files for hosting

set -e

echo "🚀 Building Bezy for GitHub Pages deployment..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf dist target/wasm32-unknown-unknown/debug/build

# Create dist directory
mkdir -p dist

# Build the WASM version
echo "🔧 Building WASM..."
cargo build --target wasm32-unknown-unknown

# Get the WASM file name (should be bezy.wasm)
WASM_FILE="target/wasm32-unknown-unknown/debug/bezy.wasm"

if [ ! -f "$WASM_FILE" ]; then
    echo "❌ Error: WASM file not found at $WASM_FILE"
    echo "Available files in target/wasm32-unknown-unknown/debug/:"
    ls -la target/wasm32-unknown-unknown/debug/ || echo "Directory not found"
    exit 1
fi

# Copy WASM file to dist
echo "📦 Copying WASM file..."
cp "$WASM_FILE" dist/

# Generate HTML file for hosting
echo "📝 Generating HTML file..."
cat > dist/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>Bezy Font Editor</title>
    <meta name="description" content="Open-source cross-platform font editor built with Bevy and Rust">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    
    <!-- Favicon -->
    <link rel="icon" type="image/x-icon" href="data:image/x-icon;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==">
    
    <style>
        body {
            margin: 0;
            padding: 0;
            background: #1a1a1a;
            color: #ffffff;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            flex-direction: column;
            min-height: 100vh;
        }
        
        #loading {
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: #1a1a1a;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            z-index: 1000;
        }
        
        #loading h1 {
            margin: 0 0 20px 0;
            font-size: 2.5em;
            font-weight: 300;
        }
        
        #loading p {
            margin: 10px 0;
            opacity: 0.7;
        }
        
        .spinner {
            border: 3px solid #333;
            border-top: 3px solid #ffffff;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin: 20px 0;
        }
        
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
        
        canvas {
            display: block;
            width: 100%;
            height: 100vh;
            background: #1a1a1a;
        }
        
        .hidden {
            display: none !important;
        }
    </style>
</head>
<body>
    <div id="loading">
        <h1>Bezy Font Editor</h1>
        <div class="spinner"></div>
        <p>Loading WebAssembly application...</p>
        <p>Please wait while we initialize the font editor.</p>
    </div>
    
    <canvas id="canvas"></canvas>
    
    <script type="module">
        import init from './bezy.js';
        
        async function run() {
            try {
                await init();
                // Hide loading screen once WASM is loaded
                document.getElementById('loading').classList.add('hidden');
            } catch (error) {
                console.error('Failed to load WASM:', error);
                document.getElementById('loading').innerHTML = `
                    <h1>Loading Error</h1>
                    <p>Failed to load the font editor. Please refresh the page.</p>
                    <p style="font-family: monospace; font-size: 0.8em; color: #ff6b6b;">${error}</p>
                `;
            }
        }
        
        run();
    </script>
</body>
</html>
EOF

# Check if wasm-bindgen is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "📦 Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Generate the JavaScript binding
echo "🔧 Generating JavaScript bindings..."
wasm-bindgen --out-dir dist --target web "$WASM_FILE" --no-typescript

# Add CNAME file for custom domain
echo "🌐 Adding CNAME for custom domain..."
echo "bezy.org" > dist/CNAME

# Copy assets if they exist
if [ -d "assets" ]; then
    echo "📁 Copying assets..."
    cp -r assets dist/
fi

echo "✅ Build complete! Files are ready in the 'dist' directory."
echo ""
echo "📊 Build summary:"
echo "   - WASM file: $(ls -lh dist/*.wasm | awk '{print $5}')"
echo "   - JavaScript bindings: dist/bezy.js"
echo "   - HTML file: dist/index.html"
echo "   - Custom domain: bezy.org (via CNAME)"
echo ""
echo "🚀 Ready for GitHub Pages deployment!" 