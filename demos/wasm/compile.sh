#!/bin/bash
# Compile WAT (WebAssembly Text) files to binary WASM
#
# Requires: wabt tools (install with: sudo apt-get install wabt)
#
# This script compiles all .wat files to .wasm binaries for embedding in JerichoOS

set -e

echo "🔨 Compiling WASM demos..."
echo "================================"

# Check if wat2wasm is available
if ! command -v wat2wasm &> /dev/null; then
    echo "❌ Error: wat2wasm not found"
    echo "Install WABT tools: sudo apt-get install wabt"
    exit 1
fi

# Compile each WAT file
for wat_file in *.wat; do
    if [ -f "$wat_file" ]; then
        wasm_file="${wat_file%.wat}.wasm"
        echo "Compiling: $wat_file -> $wasm_file"
        wat2wasm "$wat_file" -o "$wasm_file"

        # Show file size
        size=$(stat -c%s "$wasm_file" 2>/dev/null || stat -f%z "$wasm_file" 2>/dev/null)
        echo "  ✓ Generated $wasm_file ($size bytes)"
    fi
done

echo ""
echo "✅ All WASM demos compiled!"
echo ""
echo "Generated binaries:"
ls -lh *.wasm 2>/dev/null || echo "No .wasm files found"
