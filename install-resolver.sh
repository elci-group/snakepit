#!/bin/bash
# Install Snakepit Dependency Resolver
# Quick setup script to enable automatic conflict resolution

set -e

SNAKEPIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo "Snakepit Dependency Resolver Installer"
echo "=========================================="
echo ""

# Check Python version
echo "🐍 Checking Python installation..."
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 not found. Please install Python 3.8 or higher."
    exit 1
fi

PYTHON_VERSION=$(python3 -c 'import sys; print(".".join(map(str, sys.version_info[:2])))')
echo "✅ Found Python $PYTHON_VERSION"

# Make smart-pip executable
echo ""
echo "📝 Making smart-pip executable..."
chmod +x "$SNAKEPIT_DIR/smart-pip"
echo "✅ Done"

# Test the resolver
echo ""
echo "🧪 Testing dependency resolver..."
cd "$SNAKEPIT_DIR"
if python3 test_resolver.py > /dev/null 2>&1; then
    echo "✅ Resolver tests passed"
else
    echo "⚠️  Warning: Some tests failed, but resolver may still work"
fi

# Offer to create alias
echo ""
echo "=========================================="
echo "Installation Complete!"
echo "=========================================="
echo ""
echo "The dependency resolver is now installed in:"
echo "  $SNAKEPIT_DIR"
echo ""
echo "Usage options:"
echo ""
echo "1. Use smart-pip directly:"
echo "   $SNAKEPIT_DIR/smart-pip install <package>"
echo ""
echo "2. Add to PATH (add to ~/.bashrc or ~/.zshrc):"
echo "   export PATH=\"$SNAKEPIT_DIR:\$PATH\""
echo "   # Then use: smart-pip install <package>"
echo ""
echo "3. Create an alias (add to ~/.bashrc or ~/.zshrc):"
echo "   alias smart-pip='$SNAKEPIT_DIR/smart-pip'"
echo ""
echo "4. Replace pip entirely (ADVANCED):"
echo "   alias pip='$SNAKEPIT_DIR/smart-pip'"
echo ""
echo "The resolver is automatically integrated into snakepit_handler.py"
echo ""

# Ask about shell integration
echo "========================================="
echo "Shell Integration Options"
echo "========================================="
echo ""
echo "Choose integration level:"
echo "  1) Full integration - pip/pip3 auto-resolve (recommended)"
echo "  2) Alias only - use 'smart-pip' command"
echo "  3) Manual - no automatic setup"
echo ""
read -p "Select option (1-3): " -n 1 -r
echo
echo ""

case $REPLY in
    1)
        # Full shell integration
        # Detect shell
        if [ -n "$BASH_VERSION" ]; then
            SHELL_RC="$HOME/.bashrc"
        elif [ -n "$ZSH_VERSION" ]; then
            SHELL_RC="$HOME/.zshrc"
        else
            SHELL_RC="$HOME/.profile"
        fi
        
        # Add shell integration if not already present
        if ! grep -q "snakepit-resolver-integration" "$SHELL_RC" 2>/dev/null; then
            echo "" >> "$SHELL_RC"
            echo "# Snakepit dependency resolver - full shell integration" >> "$SHELL_RC"
            echo "export SNAKEPIT_DIR='$SNAKEPIT_DIR'" >> "$SHELL_RC"
            echo "[ -f \"$SNAKEPIT_DIR/snakepit-resolver-integration.sh\" ] && source \"$SNAKEPIT_DIR/snakepit-resolver-integration.sh\"" >> "$SHELL_RC"
            echo "✅ Full integration added to $SHELL_RC"
            echo "   • pip/pip3 will automatically resolve conflicts"
            echo "   • Run 'source $SHELL_RC' or restart your terminal"
            echo "   • Use 'snakepit-resolver' command to manage"
        else
            echo "ℹ️  Shell integration already exists in $SHELL_RC"
        fi
        ;;
    
    2)
        # Alias only
        # Detect shell
        if [ -n "$BASH_VERSION" ]; then
            SHELL_RC="$HOME/.bashrc"
        elif [ -n "$ZSH_VERSION" ]; then
            SHELL_RC="$HOME/.zshrc"
        else
            SHELL_RC="$HOME/.profile"
        fi
        
        # Add alias if not already present
        if ! grep -q "smart-pip" "$SHELL_RC" 2>/dev/null; then
            echo "" >> "$SHELL_RC"
            echo "# Snakepit smart-pip with automatic dependency resolution" >> "$SHELL_RC"
            echo "alias smart-pip='$SNAKEPIT_DIR/smart-pip'" >> "$SHELL_RC"
            echo "✅ Alias added to $SHELL_RC"
            echo "   • Use 'smart-pip install <package>' for auto-resolution"
            echo "   • Run 'source $SHELL_RC' or restart your terminal"
        else
            echo "ℹ️  Alias already exists in $SHELL_RC"
        fi
        ;;
    
    3)
        # Manual setup
        echo "ℹ️  No automatic shell configuration"
        echo ""
        echo "To use manually:"
        echo "  $SNAKEPIT_DIR/smart-pip install <package>"
        echo ""
        echo "To enable shell integration later:"
        echo "  source $SNAKEPIT_DIR/snakepit-resolver-integration.sh"
        ;;
    
    *)
        echo "❌ Invalid option, skipping shell integration"
        ;;
esac

echo ""
echo "🎉 Setup complete! Try it out:"
echo "   smart-pip install requests"
echo ""
