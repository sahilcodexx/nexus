#!/bin/bash
# Nexus Panel Build Script
# Builds all crates including the GPUI application

set -e

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT/nexus"

echo "🔧 Building Nexus Panel (Rust + GPUI)..."
echo ""

# Build all core crates
echo "📦 Building core crates..."
cargo build -p nexus-core
cargo build -p nexus-storage
cargo build -p nexus-search
cargo build -p nexus-emoji
cargo build -p nexus-symbols
cargo build -p nexus-gif
cargo build -p nexus-config

echo ""
echo "🚀 Building GPUI UI application..."
cargo build -p nexus-ui

echo ""
echo "✅ All crates and Nexus UI built successfully!"
echo "   Run with: ./run.sh"
echo "   Or from nexus directory: cargo run -p nexus-ui"
