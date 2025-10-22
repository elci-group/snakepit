#!/bin/bash
# Quick installation script for Snakepit routing system

set -e

SNAKEPIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║     Snakepit Routing Installation                          ║"
echo "║     Smart Python Package Management                        ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check prerequisites
echo "🔍 Checking prerequisites..."

if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 not found. Please install Python 3 first."
    exit 1
fi

echo "  ✅ Python 3: $(python3 --version)"

# Check for required Python modules
echo ""
echo "🔍 Checking Python dependencies..."

MISSING_DEPS=()

if ! python3 -c "import toml" 2>/dev/null; then
    MISSING_DEPS+=("toml")
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo "⚠️  Missing dependencies: ${MISSING_DEPS[*]}"
    echo "   Installing..."
    python3 -m pip install "${MISSING_DEPS[@]}"
fi

echo "  ✅ All dependencies satisfied"

# Make scripts executable
echo ""
echo "🔧 Making scripts executable..."
chmod +x "$SNAKEPIT_DIR/snakepit-pip-wrapper.sh"
chmod +x "$SNAKEPIT_DIR/snakepit-shell-integration.sh"
chmod +x "$SNAKEPIT_DIR/snakepit_handler.py"
chmod +x "$SNAKEPIT_DIR/snakepit_cli.py"
echo "  ✅ Scripts are executable"

# Check container availability
echo ""
echo "🔍 Checking container engines..."
if command -v podman &> /dev/null; then
    echo "  ✅ Podman found - will use containerized sandboxes"
    CONTAINER_ENGINE="podman"
elif command -v docker &> /dev/null; then
    echo "  ✅ Docker found - will use containerized sandboxes"
    CONTAINER_ENGINE="docker"
else
    echo "  ⚠️  No container engine found - will use venv sandboxes"
    CONTAINER_ENGINE="none"
fi

# Offer to add to bashrc
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
read -p "Add snakepit to ~/.bashrc? (y/N): " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Check if already present
    if grep -q "snakepit-shell-integration.sh" ~/.bashrc; then
        echo "  ⚠️  Already present in ~/.bashrc"
    else
        echo "" >> ~/.bashrc
        echo "# Snakepit - Smart Python Package Management" >> ~/.bashrc
        echo "source $SNAKEPIT_DIR/snakepit-shell-integration.sh" >> ~/.bashrc
        echo "  ✅ Added to ~/.bashrc"
    fi
fi

# Offer to install Python hooks
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Python hooks intercept pip calls from within Python programs."
echo "This provides complete coverage but requires system permissions."
echo ""
read -p "Install Python hooks (sitecustomize.py)? (y/N): " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    SITE_PACKAGES=$(python3 -c "import site; print(site.getsitepackages()[0])" 2>/dev/null)
    
    if [[ -z "$SITE_PACKAGES" ]]; then
        echo "  ❌ Could not locate site-packages directory"
    else
        echo "  📍 Site-packages: $SITE_PACKAGES"
        
        # Check if sitecustomize.py already exists
        if [[ -f "$SITE_PACKAGES/sitecustomize.py" ]] && [[ ! -L "$SITE_PACKAGES/sitecustomize.py" ]]; then
            echo "  ⚠️  Warning: sitecustomize.py already exists"
            echo "     Existing file will be backed up"
            
            if [[ -w "$SITE_PACKAGES" ]]; then
                mv "$SITE_PACKAGES/sitecustomize.py" "$SITE_PACKAGES/sitecustomize.py.backup"
            else
                sudo mv "$SITE_PACKAGES/sitecustomize.py" "$SITE_PACKAGES/sitecustomize.py.backup"
            fi
        fi
        
        # Create symlink
        if [[ -w "$SITE_PACKAGES" ]]; then
            ln -sf "$SNAKEPIT_DIR/snakepit_sitecustomize.py" "$SITE_PACKAGES/sitecustomize.py"
            echo "  ✅ Python hooks installed"
        else
            echo "  🔐 Requesting sudo for system Python..."
            sudo ln -sf "$SNAKEPIT_DIR/snakepit_sitecustomize.py" "$SITE_PACKAGES/sitecustomize.py"
            if [[ $? -eq 0 ]]; then
                echo "  ✅ Python hooks installed (with sudo)"
            else
                echo "  ❌ Failed to install hooks"
            fi
        fi
    fi
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Installation Complete!"
echo ""
echo "📋 Summary:"
echo "   Directory: $SNAKEPIT_DIR"
echo "   Container: $CONTAINER_ENGINE"
echo ""
echo "🚀 Next Steps:"
echo ""
echo "   1. Reload your shell:"
echo "      source ~/.bashrc"
echo ""
echo "   2. Check status:"
echo "      snakepit-status"
echo ""
echo "   3. Run diagnostics:"
echo "      snakepit-test"
echo ""
echo "   4. Try installing a package:"
echo "      pip install requests"
echo ""
echo "📖 Documentation:"
echo "   Full guide: $SNAKEPIT_DIR/ROUTING_SETUP.md"
echo ""
echo "🔧 Useful Commands:"
echo "   snakepit-status            Show current configuration"
echo "   snakepit-enable            Enable routing"
echo "   snakepit-disable           Disable routing"
echo "   pip-direct <args>          Bypass snakepit"
echo "   snakepit-install-hooks     Install Python hooks"
echo "   snakepit-uninstall-hooks   Remove Python hooks"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
