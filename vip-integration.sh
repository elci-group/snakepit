#!/bin/bash
# VIP (Visual Install for Python) - Shell Integration
# Source this file in your .bashrc to enable visual pip installations

# Get the directory where this script is located
VIP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Add vip to PATH if not already there
if [[ ":$PATH:" != *":$VIP_DIR:"* ]]; then
    export PATH="$VIP_DIR:$PATH"
fi

# Alias pip to vip for visual installations
alias pip='vip'
alias pip3='vip'
alias python3-pip='vip'

# Optional: Create a 'pip-classic' alias for original pip
alias pip-classic='python3 -m pip'

# Environment variable to disable GUI if needed
# export VIP_NO_GUI=1  # Uncomment to disable GUI

# Function to install with visualization
vinstall() {
    vip install "$@"
}

# Function to quickly disable/enable visual mode
vip-gui-on() {
    unset VIP_NO_GUI
    echo "✓ Visual pip mode enabled"
}

vip-gui-off() {
    export VIP_NO_GUI=1
    echo "✗ Visual pip mode disabled (using classic pip)"
}

# Status function
vip-status() {
    echo "VIP (Visual Install for Python) Status:"
    echo "  VIP Directory: $VIP_DIR"
    echo "  VIP in PATH: $(which vip 2>/dev/null || echo 'not found')"
    if [ -n "$VIP_NO_GUI" ]; then
        echo "  GUI Mode: disabled"
    else
        echo "  GUI Mode: enabled"
    fi
    echo "  Pygame available: $(python3 -c 'import pygame; print("yes")' 2>/dev/null || echo 'no')"
}

#echo "🐍 VIP (Visual Install for Python) loaded!"
#echo "   Use 'pip install <package>' for visual installations"
#echo "   Use 'vip-status' to check configuration"
#echo "   Use 'vip-gui-off' to disable visualization"
