#!/bin/bash
# Snakepit Pip Wrapper - Routes all pip installations through snakepit handler
# This script intercepts pip commands and routes them through the smart snakepit backend

# Get the directory where this script is located
SNAKEPIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}\")\" && pwd)"
SNAKEPIT_HANDLER="$SNAKEPIT_DIR/snakepit_handler.py"
SNAKEPIT_CLI="$SNAKEPIT_DIR/snakepit_cli.py"

# Check if handler is available
if [[ ! -f "$SNAKEPIT_HANDLER" ]]; then
    echo "⚠️  Snakepit handler not found at $SNAKEPIT_HANDLER" >&2
    echo "Falling back to system pip..." >&2
    exec python3 -m pip "$@"
fi

# Parse pip command
COMMAND=""
PACKAGES=()
VERSION=""
TEST_SCRIPT=""
OTHER_ARGS=()
DRY_RUN=false

# Check if user wants to bypass snakepit
if [[ "$SNAKEPIT_BYPASS" == "1" ]] || [[ "$1" == "--snakepit-bypass" ]]; then
    exec python3 -m pip "$@"
fi

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        install)
            COMMAND="install"
            shift
            ;;
        uninstall|remove)
            COMMAND="uninstall"
            shift
            ;;
        list|freeze|show)
            # For list-type commands, pass through to regular pip
            exec python3 -m pip "$@"
            ;;
        --version)
            # Show snakepit version info
            echo "Snakepit Pip Wrapper (routing to smart handler)"
            python3 -m pip --version
            exit 0
            ;;
        --dry-run|-n)
            DRY_RUN=true
            shift
            ;;
        --test-script)
            TEST_SCRIPT="$2"
            shift 2
            ;;
        --help|-h)
            cat << EOF
Snakepit Pip Wrapper - Smart Package Management

This wrapper routes pip commands through snakepit's validation system.

Usage:
  pip install <package> [--version VERSION] [--test-script SCRIPT]
  pip uninstall <package>
  pip list
  
Additional Snakepit Options:
  --dry-run, -n           Validate package without installing
  --test-script SCRIPT    Use custom test script for validation
  --snakepit-bypass       Bypass snakepit and use system pip directly
  
Environment Variables:
  SNAKEPIT_BYPASS=1       Bypass snakepit for this command
  SNAKEPIT_AUTO_TEST=0    Disable automatic package testing (default: enabled)
  
Examples:
  pip install requests                    # Smart install with validation
  pip install numpy --dry-run             # Validate without installing
  pip install pandas --test-script test.py # Use custom validation
  SNAKEPIT_BYPASS=1 pip install flask     # Use system pip directly
  
For regular pip help:
  SNAKEPIT_BYPASS=1 pip --help

EOF
            exit 0
            ;;
        -*)
            # Collect other flags
            OTHER_ARGS+=("$1")
            shift
            ;;
        *)
            # This should be a package name
            PACKAGES+=("$1")
            shift
            ;;
    esac
done

# Handle install command through snakepit
if [[ "$COMMAND" == "install" ]]; then
    if [[ ${#PACKAGES[@]} -eq 0 ]]; then
        echo "❌ Error: No package specified for installation" >&2
        exit 1
    fi
    
    # Check if auto-testing is disabled
    AUTO_TEST="${SNAKEPIT_AUTO_TEST:-1}"
    
    for PACKAGE in "${PACKAGES[@]}"; do
        # Extract version if package has format package==version
        if [[ "$PACKAGE" =~ ^([^=]+)==(.+)$ ]]; then
            PKG_NAME="${BASH_REMATCH[1]}"
            PKG_VERSION="${BASH_REMATCH[2]}"
        else
            PKG_NAME="$PACKAGE"
            PKG_VERSION=""
        fi
        
        echo "🐍 Snakepit: Processing $PKG_NAME through smart handler..."
        
        # Build snakepit command
        SNAKEPIT_CMD="python3 $SNAKEPIT_CLI"
        
        if [[ "$AUTO_TEST" == "0" ]] || [[ "$DRY_RUN" == "true" ]]; then
            # Use validate command (test only, no install)
            SNAKEPIT_CMD="$SNAKEPIT_CMD validate $PKG_NAME"
        else
            # Use install command (test + install)
            SNAKEPIT_CMD="$SNAKEPIT_CMD install $PKG_NAME"
        fi
        
        # Add version if specified
        if [[ -n "$PKG_VERSION" ]]; then
            SNAKEPIT_CMD="$SNAKEPIT_CMD --version $PKG_VERSION"
        fi
        
        # Add test script if specified
        if [[ -n "$TEST_SCRIPT" ]]; then
            SNAKEPIT_CMD="$SNAKEPIT_CMD --test-script $TEST_SCRIPT"
        fi
        
        # Add verbose flag if present
        for arg in "${OTHER_ARGS[@]}"; do
            case "$arg" in
                -v|--verbose)
                    SNAKEPIT_CMD="$SNAKEPIT_CMD --verbose"
                    ;;
            esac
        done
        
        # Execute snakepit handler
        eval "$SNAKEPIT_CMD"
        EXIT_CODE=$?
        
        if [[ $EXIT_CODE -ne 0 ]]; then
            echo "❌ Snakepit: Failed to process $PKG_NAME" >&2
            exit $EXIT_CODE
        fi
    done
    
    exit 0

elif [[ "$COMMAND" == "uninstall" ]]; then
    if [[ ${#PACKAGES[@]} -eq 0 ]]; then
        echo "❌ Error: No package specified for uninstall" >&2
        exit 1
    fi
    
    # For now, uninstall passes through to system pip
    # TODO: Add snakepit tracking for uninstalls
    echo "🐍 Snakepit: Routing uninstall to system pip..."
    exec python3 -m pip uninstall "${OTHER_ARGS[@]}" "${PACKAGES[@]}"

else
    # Unknown or no command - pass through to pip
    exec python3 -m pip "$@"
fi
