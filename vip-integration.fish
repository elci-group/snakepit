#!/usr/bin/env fish
# VIP (Visual Install for Python) - Fish Integration
# Source this file in your config.fish to enable visual pip installations

# Get the directory where this script is located
set -l VIP_DIR (dirname (status --current-filename))

# Add vip to PATH if not already there
if not contains $VIP_DIR $PATH
    set -gx PATH $VIP_DIR $PATH
end

# Alias pip to vip for visual installations
alias pip='vip'
alias pip3='vip'
alias python3-pip='vip'

# Optional: Create a 'pip-classic' alias for original pip
alias pip-classic='python3 -m pip'

# Environment variable to disable GUI if needed
# set -gx VIP_NO_GUI 1  # Uncomment to disable GUI

# Function to install with visualization
function vinstall
    vip install $argv
end

# Function to quickly disable/enable visual mode
function vip-gui-on
    set -e VIP_NO_GUI
    echo "✓ Visual pip mode enabled"
end

function vip-gui-off
    set -gx VIP_NO_GUI 1
    echo "✗ Visual pip mode disabled (using classic pip)"
end

# Status function
function vip-status
    echo "VIP (Visual Install for Python) Status:"
    echo "  VIP Directory: $VIP_DIR"
    if type -q vip
        echo "  VIP in PATH: "(which vip)
    else
        echo "  VIP in PATH: not found"
    end
    if set -q VIP_NO_GUI
        echo "  GUI Mode: disabled"
    else
        echo "  GUI Mode: enabled"
    end
    python3 -c 'import pygame; print("  Pygame available: yes")' 2>/dev/null; or echo "  Pygame available: no"
end

# Completion for vip command
complete -c vip -f
complete -c vip -n '__fish_use_subcommand' -a 'install' -d 'Install packages'
complete -c vip -n '__fish_use_subcommand' -a 'uninstall' -d 'Uninstall packages'
complete -c vip -n '__fish_use_subcommand' -a 'list' -d 'List installed packages'
complete -c vip -n '__fish_use_subcommand' -a 'show' -d 'Show package information'
complete -c vip -n '__fish_use_subcommand' -a 'freeze' -d 'Output installed packages'
complete -c vip -n '__fish_use_subcommand' -a 'search' -d 'Search PyPI'
complete -c vip -n '__fish_use_subcommand' -a 'download' -d 'Download packages'
complete -c vip -n '__fish_use_subcommand' -a 'wheel' -d 'Build wheels'
complete -c vip -n '__fish_use_subcommand' -a 'hash' -d 'Compute hashes'
complete -c vip -n '__fish_use_subcommand' -a 'check' -d 'Verify installed packages'
complete -c vip -n '__fish_use_subcommand' -a 'config' -d 'Manage configuration'
complete -c vip -n '__fish_use_subcommand' -a 'help' -d 'Show help'

# Common options
complete -c vip -s h -l help -d 'Show help message'
complete -c vip -s v -l verbose -d 'Give more output'
complete -c vip -s q -l quiet -d 'Give less output'
complete -c vip -l version -d 'Show version'
complete -c vip -s V -d 'Show version'

# Install-specific completions
complete -c vip -n '__fish_seen_subcommand_from install' -s U -l upgrade -d 'Upgrade packages'
complete -c vip -n '__fish_seen_subcommand_from install' -l user -d 'Install to user site-packages'
complete -c vip -n '__fish_seen_subcommand_from install' -l no-deps -d 'Don\'t install dependencies'
complete -c vip -n '__fish_seen_subcommand_from install' -s r -l requirement -d 'Install from requirements file' -F

echo "🐍 VIP (Visual Install for Python) loaded!"
echo "   Use 'pip install <package>' for visual installations"
echo "   Use 'vip-status' to check configuration"
echo "   Use 'vip-gui-off' to disable visualization"
