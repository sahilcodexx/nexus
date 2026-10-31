#!/bin/bash
# Nexus Panel Run Script
# Builds and launches the Windows 11-style GPUI application

set -e

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT/nexus"

echo "▶️ Nexus Panel - Windows 11 Fluent UI (Rust + GPUI)"
echo ""

# Build all core crates
echo "📦 Building core crates..."
cargo build -p nexus-core --quiet
cargo build -p nexus-storage --quiet
cargo build -p nexus-search --quiet
cargo build -p nexus-emoji --quiet
cargo build -p nexus-symbols --quiet
cargo build -p nexus-gif --quiet
cargo build -p nexus-config --quiet

echo "✅ All core crates built successfully!"
echo ""

# Build and run UI
echo "🚀 Building Nexus UI..."
cargo build -p nexus-ui

echo ""
echo "▶️ Launching Nexus Panel..."
cargo run -p nexus-ui -- "$@"
