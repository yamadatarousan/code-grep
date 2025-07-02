#!/bin/bash

# code-grep installation script
# Builds and installs the code-grep CLI tool

set -e

echo "🔧 Building code-grep..."
cargo build --release

echo "📦 Installing binary..."
mkdir -p ~/.local/bin
cp target/release/cg ~/.local/bin/

echo "✅ Installation complete!"
echo ""
echo "To use code-grep, make sure ~/.local/bin is in your PATH:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
echo "Add this to your shell profile (.bashrc, .zshrc, etc.) to make it permanent."
echo ""
echo "Usage examples:"
echo "  cg \"pattern\" --type rust"
echo "  cg \"TODO\" --comments-only"
echo "  cg --help"