#!/bin/bash
# Snakepit Shell Integration - Routes all pip/python installations through smart backend
# Source this file in your ~/.bashrc to enable snakepit routing

# Get the directory where this script is located
SNAKEPIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Add snakepit to PATH if not already there
if [[ ":$PATH:" != *":$SNAKEPIT_DIR:"* ]]; then
    export PATH="$SNAKEPIT_DIR:$PATH"
fi

# Export snakepit directory for other scripts
export SNAKEPIT_HOME="$SNAKEPIT_DIR"

# Make wrapper executable
chmod +x "$SNAKEPIT_DIR/snakepit-pip-wrapper.sh" 2>/dev/null

# Source universal wrappers
source "$SNAKEPIT_DIR/snakepit-universal-wrapper.sh"

# Create pip wrapper function
pip() {
    _snakepit_pip_wrapper "$@"
}

# Also wrap pip3
pip3() {
    _snakepit_pip_wrapper "$@"
}

# Wrap python/python3 for -m pip
python() {
    _snakepit_python_wrapper "$@"
}

python3() {
    _snakepit_python_wrapper "$@"
}

# Wrap easy_install (legacy)
if command -v easy_install &> /dev/null; then
    easy_install() {
        _snakepit_easy_install_wrapper "$@"
    }
fi

# Wrap poetry
if command -v poetry &> /dev/null; then
    poetry() {
        _snakepit_poetry_wrapper "$@"
    }
fi

# Wrap pipenv
if command -v pipenv &> /dev/null; then
    pipenv() {
        _snakepit_pipenv_wrapper "$@"
    }
fi

# Wrap conda/mamba
if command -v conda &> /dev/null; then
    conda() {
        _snakepit_conda_wrapper "$@"
    }
fi

if command -v mamba &> /dev/null; then
    mamba() {
        _snakepit_conda_wrapper "$@"
    }
fi

# Wrap pdm
if command -v pdm &> /dev/null; then
    pdm() {
        _snakepit_pdm_wrapper "$@"
    }
fi

# Wrap flit
if command -v flit &> /dev/null; then
    flit() {
        _snakepit_flit_wrapper "$@"
    }
fi

# Wrap hatch
if command -v hatch &> /dev/null; then
    hatch() {
        _snakepit_hatch_wrapper "$@"
    }
fi

# Wrap pip-sync
if command -v pip-sync &> /dev/null; then
    pip-sync() {
        _snakepit_pip_sync_wrapper "$@"
    }
fi

# Export functions so they're available in subshells
export -f pip
export -f pip3
export -f python
export -f python3

# Alias for those who use python -m pip
alias python-pip='pip'
alias python3-pip='pip3'

# Function to bypass snakepit temporarily
pip-direct() {
    SNAKEPIT_BYPASS=1 python3 -m pip "$@"
}

# Function to enable/disable snakepit routing
snakepit-enable() {
    unset SNAKEPIT_BYPASS
    export SNAKEPIT_INTERCEPT=1
    echo "✅ Snakepit routing enabled"
    echo "   All pip installations will be validated through snakepit"
}

snakepit-disable() {
    export SNAKEPIT_BYPASS=1
    export SNAKEPIT_INTERCEPT=0
    echo "⏸️  Snakepit routing disabled"
    echo "   Using system pip directly"
}

# Function to install snakepit sitecustomize.py
snakepit-install-hooks() {
    echo "🔧 Installing snakepit Python hooks..."
    
    # Get site-packages directory
    SITE_PACKAGES=$(python3 -c "import site; print(site.getsitepackages()[0])" 2>/dev/null)
    
    if [[ -z "$SITE_PACKAGES" ]]; then
        echo "❌ Could not locate site-packages directory" >&2
        return 1
    fi
    
    echo "   Site-packages: $SITE_PACKAGES"
    
    # Check if we have write permission
    if [[ ! -w "$SITE_PACKAGES" ]]; then
        echo "⚠️  No write permission to site-packages"
        echo "   Trying with sudo..."
        sudo ln -sf "$SNAKEPIT_DIR/snakepit_sitecustomize.py" "$SITE_PACKAGES/sitecustomize.py"
    else
        ln -sf "$SNAKEPIT_DIR/snakepit_sitecustomize.py" "$SITE_PACKAGES/sitecustomize.py"
    fi
    
    if [[ $? -eq 0 ]]; then
        echo "✅ Snakepit hooks installed successfully"
        echo "   Python will now intercept pip calls automatically"
        return 0
    else
        echo "❌ Failed to install hooks" >&2
        return 1
    fi
}

# Function to uninstall snakepit sitecustomize.py
snakepit-uninstall-hooks() {
    echo "🔧 Uninstalling snakepit Python hooks..."
    
    SITE_PACKAGES=$(python3 -c "import site; print(site.getsitepackages()[0])" 2>/dev/null)
    
    if [[ -z "$SITE_PACKAGES" ]]; then
        echo "❌ Could not locate site-packages directory" >&2
        return 1
    fi
    
    SITECUSTOMIZE="$SITE_PACKAGES/sitecustomize.py"
    
    if [[ -L "$SITECUSTOMIZE" ]] && [[ "$(readlink "$SITECUSTOMIZE")" == *"snakepit"* ]]; then
        if [[ ! -w "$SITE_PACKAGES" ]]; then
            sudo rm "$SITECUSTOMIZE"
        else
            rm "$SITECUSTOMIZE"
        fi
        echo "✅ Snakepit hooks uninstalled"
    else
        echo "⚠️  Snakepit hooks not found or not installed"
    fi
}

# Function to show snakepit status
snakepit-status() {
    echo "🐍 Snakepit Status"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Home Directory: $SNAKEPIT_DIR"
    echo "  Wrapper Script: $(which pip 2>/dev/null || echo 'not in PATH')"
    
    # Check if routing is active
    if [[ "$SNAKEPIT_BYPASS" == "1" ]] || [[ "$SNAKEPIT_INTERCEPT" == "0" ]]; then
        echo "  Routing Status: ⏸️  DISABLED"
    else
        echo "  Routing Status: ✅ ENABLED"
    fi
    
    # Check if handler exists
    if [[ -f "$SNAKEPIT_DIR/snakepit_handler.py" ]]; then
        echo "  Handler: ✅ Found"
    else
        echo "  Handler: ❌ Not found"
    fi
    
    # Check if CLI exists
    if [[ -f "$SNAKEPIT_DIR/snakepit_cli.py" ]]; then
        echo "  CLI: ✅ Found"
    else
        echo "  CLI: ❌ Not found"
    fi
    
    # Check if sitecustomize is installed
    SITE_PACKAGES=$(python3 -c "import site; print(site.getsitepackages()[0])" 2>/dev/null)
    if [[ -n "$SITE_PACKAGES" ]] && [[ -f "$SITE_PACKAGES/sitecustomize.py" ]]; then
        if [[ -L "$SITE_PACKAGES/sitecustomize.py" ]] && [[ "$(readlink "$SITE_PACKAGES/sitecustomize.py")" == *"snakepit"* ]]; then
            echo "  Python Hooks: ✅ Installed"
        else
            echo "  Python Hooks: ⚠️  Other sitecustomize.py exists"
        fi
    else
        echo "  Python Hooks: ❌ Not installed"
    fi
    
    # Check Python path
    echo "  Python: $(which python3)"
    
    # Check for container engines
    if command -v podman &> /dev/null; then
        echo "  Container: ✅ Podman available"
    elif command -v docker &> /dev/null; then
        echo "  Container: ✅ Docker available"
    else
        echo "  Container: ⚠️  None (using venv sandbox)"
    fi
    
    # Environment variables
    echo ""
    echo "Environment Variables:"
    echo "  SNAKEPIT_BYPASS: ${SNAKEPIT_BYPASS:-not set}"
    echo "  SNAKEPIT_INTERCEPT: ${SNAKEPIT_INTERCEPT:-not set}"
    echo "  SNAKEPIT_AUTO_TEST: ${SNAKEPIT_AUTO_TEST:-1 (enabled)}"
    echo "  SNAKEPIT_VERBOSE: ${SNAKEPIT_VERBOSE:-not set}"
}

# Function to test snakepit installation
snakepit-test() {
    echo "🧪 Testing snakepit installation..."
    echo ""
    
    # Test 1: Check if wrapper is accessible
    echo "Test 1: Wrapper accessibility"
    if type pip | grep -q "function"; then
        echo "  ✅ pip is a function (wrapper active)"
    else
        echo "  ❌ pip is not wrapped"
    fi
    
    # Test 2: Dry run installation
    echo ""
    echo "Test 2: Dry run package validation"
    echo "  Testing with 'requests' package..."
    SNAKEPIT_AUTO_TEST=0 pip install requests 2>&1 | head -n 5
    
    echo ""
    echo "Test 3: Handler availability"
    if python3 -c "import sys; sys.path.insert(0, '$SNAKEPIT_DIR'); from snakepit_handler import SnakepitHandler; print('✅ Handler imports successfully')" 2>/dev/null; then
        echo "  ✅ Handler can be imported"
    else
        echo "  ❌ Handler import failed"
    fi
    
    echo ""
    echo "🏁 Test complete"
}

# Auto-install helper
snakepit-setup() {
    echo "🚀 Setting up Snakepit..."
    echo ""
    
    # Check dependencies
    echo "Checking dependencies..."
    
    if ! command -v python3 &> /dev/null; then
        echo "❌ Python 3 not found. Please install Python 3."
        return 1
    fi
    
    # Check for required Python packages
    python3 -c "import toml" 2>/dev/null || {
        echo "⚠️  toml module not found. Installing..."
        python3 -m pip install toml
    }
    
    echo "✅ Dependencies OK"
    echo ""
    
    # Offer to install hooks
    echo "Do you want to install Python hooks (sitecustomize.py)?"
    echo "This will intercept pip calls from within Python programs."
    read -p "Install hooks? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        snakepit-install-hooks
    fi
    
    echo ""
    echo "✅ Snakepit setup complete!"
    echo ""
    echo "Available commands:"
    echo "  pip install <package>      - Install with validation"
    echo "  pip-direct <args>          - Bypass snakepit"
    echo "  snakepit-enable            - Enable routing"
    echo "  snakepit-disable           - Disable routing"
    echo "  snakepit-status            - Show status"
    echo "  snakepit-test              - Run tests"
}

# Welcome message (only on interactive shells)
if [[ $- == *i* ]] && [[ "$SNAKEPIT_QUIET" != "1" ]]; then
    echo "🐍 Snakepit loaded - Smart Python package management active"
    echo "   Use 'snakepit-status' to see configuration"
    echo "   Use 'snakepit-setup' for first-time setup"
fi
