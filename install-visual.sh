#!/bin/bash
# Quick installer for Visual Install for Python (VIP)

set -e

SNAKEPIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASHRC="$HOME/.bashrc"

echo "🐍 Installing Visual Install for Python (VIP)"
echo "=============================================="
echo ""

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Error: python3 is required but not found"
    exit 1
fi
echo "✓ Python3 found: $(python3 --version)"

# Check pip
if ! python3 -m pip --version &> /dev/null; then
    echo "❌ Error: pip is required but not found"
    exit 1
fi
echo "✓ pip found"

# Install pygame if not already installed
echo ""
echo "Checking for pygame..."
if ! python3 -c "import pygame" &> /dev/null; then
    echo "📦 Installing pygame..."
    python3 -m pip install pygame --user
    if [ $? -eq 0 ]; then
        echo "✓ pygame installed successfully"
    else
        echo "⚠️  Warning: pygame installation failed"
        echo "   Visual mode will not work, but vip will fallback to classic mode"
    fi
else
    echo "✓ pygame already installed"
fi

# Make scripts executable
echo ""
echo "Setting up scripts..."
chmod +x "$SNAKEPIT_DIR/vip"
chmod +x "$SNAKEPIT_DIR/snake_monitor.py"
chmod +x "$SNAKEPIT_DIR/snake_gui.py"
chmod +x "$SNAKEPIT_DIR/vip-integration.sh"
echo "✓ Scripts marked as executable"

# Add to bashrc
echo ""
echo "Configuring shell integration..."

# Check if already integrated
if grep -q "vip-integration.sh" "$BASHRC" 2>/dev/null; then
    echo "✓ Shell integration already configured"
else
    echo "" >> "$BASHRC"
    echo "# Visual Install for Python (VIP)" >> "$BASHRC"
    echo "source $SNAKEPIT_DIR/vip-integration.sh" >> "$BASHRC"
    echo "✓ Added to $BASHRC"
fi

# Add to PATH temporarily for this session
export PATH="$SNAKEPIT_DIR:$PATH"

echo ""
echo "=============================================="
echo "✓ Installation complete!"
echo ""
echo "Next steps:"
echo "  1. Reload your shell:"
echo "     source ~/.bashrc"
echo ""
echo "  2. Test with demo mode:"
echo "     cd $SNAKEPIT_DIR"
echo "     python3 snake_monitor.py"
echo ""
echo "  3. Try a real installation:"
echo "     pip install colorama"
echo ""
echo "Useful commands:"
echo "  vip-status       - Check configuration"
echo "  vip-gui-off      - Disable visual mode"
echo "  vip-gui-on       - Enable visual mode"
echo "  pip-classic      - Use original pip"
echo ""
echo "See VISUAL_SETUP.md for full documentation"
echo "🐍 Happy installing!"
