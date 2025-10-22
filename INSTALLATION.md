# Installation Guide

## Prerequisites

- **Rust**: 1.70 or later (install from https://rustup.rs/)
- **Python**: 3.7 or later
- **Git**: For cloning the repository
- **Cargo**: Comes with Rust

## Installation Methods

### Method 1: From Source (Recommended for Development)

```bash
# Clone the repository
git clone https://github.com/adminx/snakepit.git
cd snakepit

# Build in release mode
cargo build --release

# Install to /usr/local/bin (optional)
sudo cp target/release/snakepit /usr/local/bin/

# Verify installation
snakepit --version
```

### Method 2: Using Cargo (Simple)

```bash
# Install directly from crates.io (when published)
cargo install snakepit

# Verify installation
snakepit --version
```

### Method 3: Download Pre-built Binary

Visit the [Releases](https://github.com/adminx/snakepit/releases) page and download the binary for your platform.

```bash
# Linux/macOS
tar xzf snakepit-*.tar.gz
sudo mv snakepit /usr/local/bin/

# Windows
# Extract snakepit.exe and add to PATH
```

## Shell Integration Setup

### Bash (Recommended)

Add the following to your `~/.bashrc`:

```bash
# Snakepit auto-install and helper functions
if [ -f /usr/local/bin/snakepit ]; then
    # Auto-install on import errors
    python() {
        /usr/bin/python3 "$@" 2>&1 | grep -q "ModuleNotFoundError\|ImportError" && snakepit install "${1##*import }" && /usr/bin/python3 "$@" || /usr/bin/python3 "$@"
    }
    
    python3() {
        /usr/bin/python3 "$@" 2>&1 | grep -q "ModuleNotFoundError\|ImportError" && snakepit install "${1##*import }" && /usr/bin/python3 "$@" || /usr/bin/python3 "$@"
    }
fi
```

Or copy the full integration from `~/.bashrc` after running:

```bash
source ~/.bashrc
```

### Zsh

Add to `~/.zshrc`:

```bash
# Similar to bash integration above
if [ -f /usr/local/bin/snakepit ]; then
    alias python="python3"
fi
```

### Fish

Add to `~/.config/fish/config.fish`:

```fish
# Fish shell integration coming soon
```

## Post-Installation

### 1. Verify Installation

```bash
snakepit --version
snakepit --help
```

### 2. Configure Global Settings (Optional)

```bash
# Create config directory
mkdir -p ~/.config/snakepit

# Create config file
cat > ~/.config/snakepit/config.toml << 'EOF'
default_backend = "pip"
default_venv_backend = "venv"
venv_path = "~/.snakepit/venvs"
python_version = "3.10"
cache_enabled = true
timeout = 30
retries = 3

[mirrors]
mirrors = ["https://pypi.org/simple/"]
EOF
```

### 3. Test Installation

```bash
# Create a test virtual environment
snakepit venv create test-env

# List virtual environments
snakepit venv list

# Delete test environment
snakepit venv delete test-env

# Test package installation (requires active venv or uses system Python)
python3 -c "import sys; print(sys.version)"
```

## Uninstallation

### From Source/Cargo

```bash
# If installed from source
rm /usr/local/bin/snakepit

# If installed via cargo
cargo uninstall snakepit

# Remove configuration
rm -rf ~/.config/snakepit
rm -rf ~/.snakepit
```

### Shell Integration Cleanup

Remove the Snakepit section from:
- `~/.bashrc` (for Bash)
- `~/.zshrc` (for Zsh)
- `~/.config/fish/config.fish` (for Fish)

## Troubleshooting

### Command Not Found

If `snakepit` is not found after installation:

```bash
# Check if it's in PATH
which snakepit

# Add to PATH if missing
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Permission Denied

```bash
# Make binary executable
chmod +x /usr/local/bin/snakepit

# Set proper permissions
chmod 755 ~/.config/snakepit
```

### Cargo Build Fails

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release

# Check for specific errors
cargo build --verbose
```

### Shell Integration Not Working

1. Verify snakepit is installed: `which snakepit`
2. Check shell configuration: `echo $SHELL`
3. Reload shell: `exec $SHELL` or `source ~/.bashrc`
4. Test directly: `snakepit --help`

## Updating

### From Source

```bash
cd /path/to/snakepit
git pull origin main
cargo build --release
sudo cp target/release/snakepit /usr/local/bin/
```

### From Cargo

```bash
cargo install snakepit --force
```

## Getting Help

- Check the [README.md](README.md)
- Review [CONTRIBUTING.md](CONTRIBUTING.md)
- Open an issue on GitHub
- Check existing issues for similar problems

## Next Steps

1. [Read the Quick Start](README.md#quick-start)
2. [Set up Shell Integration](README.md#shell-integration)
3. [Check Examples](examples/)
4. [Configure Global Settings](#configure-global-settings-optional)

---

For the latest installation instructions, visit: https://github.com/adminx/snakepit
