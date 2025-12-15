#!/bin/bash
# Snakepit Dependency Resolver - Shell Integration
# Automatically intercepts pip/pip3 commands and adds conflict resolution
# Source this file in your ~/.bashrc or ~/.zshrc

# Store original pip commands
if ! type __original_pip_command &>/dev/null; then
    # Save reference to original pip
    if command -v pip3 &>/dev/null; then
        __original_pip3() {
            command pip3 "$@"
        }
    fi
    
    if command -v pip &>/dev/null; then
        __original_pip() {
            command pip "$@"
        }
    fi
    
    export -f __original_pip3 2>/dev/null || true
    export -f __original_pip 2>/dev/null || true
fi

# Detect snakepit directory
if [ -z "$SNAKEPIT_DIR" ]; then
    # Try to auto-detect
    if [ -f "$HOME/snakepit/smart-pip" ]; then
        export SNAKEPIT_DIR="$HOME/snakepit"
    elif [ -f "$(dirname "${BASH_SOURCE[0]}")/smart-pip" ]; then
        export SNAKEPIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    else
        echo "⚠️  Warning: Could not locate snakepit directory"
        echo "   Set SNAKEPIT_DIR environment variable manually"
        return 1
    fi
fi

# Smart pip wrapper function for pip3
pip3() {
    local cmd="$1"
    
    # Check if dependency resolver is disabled
    if [ "$SNAKEPIT_RESOLVER_DISABLED" = "1" ]; then
        __original_pip3 "$@"
        return $?
    fi
    
    # Only intercept install commands
    if [ "$cmd" = "install" ]; then
        # Check if smart-pip exists
        if [ -x "$SNAKEPIT_DIR/smart-pip" ]; then
            echo "🐍 Using Snakepit dependency resolver..."
            "$SNAKEPIT_DIR/smart-pip" "$@"
            local result=$?
            
            # If resolver had issues, fall back to regular pip
            if [ $result -ne 0 ]; then
                echo "⚠️  Resolver encountered issues, consider manual resolution"
            fi
            return $result
        else
            echo "⚠️  smart-pip not found, using regular pip"
            __original_pip3 "$@"
        fi
    else
        # Pass through non-install commands
        __original_pip3 "$@"
    fi
}

# Smart pip wrapper function for pip
pip() {
    local cmd="$1"
    
    # Check if dependency resolver is disabled
    if [ "$SNAKEPIT_RESOLVER_DISABLED" = "1" ]; then
        __original_pip "$@"
        return $?
    fi
    
    # Only intercept install commands
    if [ "$cmd" = "install" ]; then
        # Check if smart-pip exists
        if [ -x "$SNAKEPIT_DIR/smart-pip" ]; then
            echo "🐍 Using Snakepit dependency resolver..."
            "$SNAKEPIT_DIR/smart-pip" "$@"
            local result=$?
            
            # If resolver had issues, fall back to regular pip
            if [ $result -ne 0 ]; then
                echo "⚠️  Resolver encountered issues, consider manual resolution"
            fi
            return $result
        else
            echo "⚠️  smart-pip not found, using regular pip"
            __original_pip "$@"
        fi
    else
        # Pass through non-install commands
        __original_pip "$@"
    fi
}

# Utility functions
snakepit-resolver() {
    case "$1" in
        enable)
            unset SNAKEPIT_RESOLVER_DISABLED
            echo "✅ Snakepit dependency resolver enabled"
            ;;
        disable)
            export SNAKEPIT_RESOLVER_DISABLED=1
            echo "🛑 Snakepit dependency resolver disabled"
            ;;
        status)
            if [ "$SNAKEPIT_RESOLVER_DISABLED" = "1" ]; then
                echo "Status: ❌ Disabled"
            else
                echo "Status: ✅ Enabled"
            fi
            echo "Directory: $SNAKEPIT_DIR"
            if [ -x "$SNAKEPIT_DIR/smart-pip" ]; then
                echo "smart-pip: ✅ Available"
            else
                echo "smart-pip: ❌ Not found"
            fi
            ;;
        test)
            echo "🧪 Testing dependency resolver..."
            if [ -f "$SNAKEPIT_DIR/test_resolver.py" ]; then
                cd "$SNAKEPIT_DIR" && python3 test_resolver.py
            else
                echo "❌ Test file not found"
            fi
            ;;
        help|--help|-h)
            cat << 'EOF'
Snakepit Dependency Resolver - Shell Integration

Commands:
  snakepit-resolver enable    Enable automatic conflict resolution
  snakepit-resolver disable   Disable and use regular pip
  snakepit-resolver status    Show current status
  snakepit-resolver test      Run resolver tests
  snakepit-resolver help      Show this help

Environment Variables:
  SNAKEPIT_DIR                Directory containing snakepit tools
  SNAKEPIT_RESOLVER_DISABLED  Set to 1 to disable resolver

Usage:
  # Normal usage - resolver is automatic
  pip install requests
  pip3 install flask numpy

  # Temporarily disable for one command
  SNAKEPIT_RESOLVER_DISABLED=1 pip install package

  # Disable resolver entirely
  snakepit-resolver disable
  pip install package  # Uses regular pip
  
  # Re-enable resolver
  snakepit-resolver enable

  # Check status
  snakepit-resolver status

The resolver automatically detects and resolves dependency conflicts
when you use pip/pip3 install commands.
EOF
            ;;
        *)
            echo "Unknown command: $1"
            echo "Run 'snakepit-resolver help' for usage"
            return 1
            ;;
    esac
}

# Export functions for use in subshells
export -f pip 2>/dev/null || true
export -f pip3 2>/dev/null || true
export -f snakepit-resolver 2>/dev/null || true

# Inform user
if [ -n "$PS1" ]; then
    # Only show in interactive shells
    echo "🐍 Snakepit dependency resolver integration loaded"
    echo "   • pip/pip3 install commands will auto-resolve conflicts"
    echo "   • Run 'snakepit-resolver help' for more info"
    echo "   • Run 'snakepit-resolver status' to check setup"
fi
