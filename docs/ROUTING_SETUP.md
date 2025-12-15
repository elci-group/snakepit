# Snakepit Routing Setup Guide

This guide explains how to configure snakepit to route all pip and Python installations through the smart validation backend.

## Overview

Snakepit provides multiple layers of interception to ensure all Python package installations are validated before being installed on your system:

1. **Shell Integration** - Wraps `pip` and `pip3` commands in your shell
2. **Python Hooks** - Intercepts pip calls from within Python programs via `sitecustomize.py`
3. **Direct CLI** - Use snakepit CLI directly for explicit control

## Quick Start

### 1. Source the Shell Integration

Add to your `~/.bashrc`:

```bash
source /home/adminx/snakepit/snakepit-shell-integration.sh
```

Then reload your shell:

```bash
source ~/.bashrc
```

### 2. Run Setup

```bash
snakepit-setup
```

This will:
- Check dependencies
- Offer to install Python hooks
- Configure your environment

### 3. Verify Installation

```bash
snakepit-status
```

You should see:
- ✅ Routing Status: ENABLED
- ✅ Handler: Found
- ✅ CLI: Found

## Installation Methods

### Method 1: Shell Integration Only (Recommended for beginners)

This method only intercepts shell commands. Python programs calling pip directly will bypass snakepit.

```bash
# Add to ~/.bashrc
echo 'source /home/adminx/snakepit/snakepit-shell-integration.sh' >> ~/.bashrc
source ~/.bashrc
```

**Pros:**
- Easy to enable/disable
- No system-wide changes
- Can be bypassed with `pip-direct`

**Cons:**
- Doesn't intercept pip calls from Python programs

### Method 2: Python Hooks (Complete interception)

This method intercepts ALL pip calls, including those from Python programs.

```bash
# Install hooks
snakepit-install-hooks
```

This creates a symlink from site-packages to snakepit's sitecustomize.py.

**Pros:**
- Complete interception
- Works for all pip calls
- Transparent to applications

**Cons:**
- System-wide change
- Requires sudo if system Python
- May conflict with existing sitecustomize.py

### Method 3: Manual PATH Configuration

If you want more control, manually configure PATH:

```bash
export PATH="/home/adminx/snakepit:$PATH"
export SNAKEPIT_HOME="/home/adminx/snakepit"

# Create symlinks
mkdir -p ~/bin
ln -sf /home/adminx/snakepit/snakepit-pip-wrapper.sh ~/bin/pip
ln -sf /home/adminx/snakepit/snakepit-pip-wrapper.sh ~/bin/pip3
```

## Usage Examples

### Basic Installation

```bash
# Install with validation
pip install requests

# Output:
# 🐍 Snakepit: Processing requests through smart handler...
# 📥 INGEST: Starting ingestion of requests
# ✅ INGEST: Successfully ingested requests
# 🧪 TEST/COLLABORATE: Validating requests
# ✅ TEST/COLLABORATE: requests approved for installation
# ⚔️ CONSCRIPT: Installing requests
# ✅ Successfully installed requests
```

### Install with Specific Version

```bash
pip install numpy==1.24.0
```

### Dry Run (Validate Only)

```bash
pip install pandas --dry-run
# or
SNAKEPIT_AUTO_TEST=0 pip install pandas
```

### Bypass Snakepit

```bash
# One-time bypass
SNAKEPIT_BYPASS=1 pip install flask

# Or use the helper function
pip-direct install flask
```

### Custom Test Script

```bash
pip install mypackage --test-script ./my_test.py
```

## Environment Variables

### Configuration

- `SNAKEPIT_BYPASS=1` - Bypass snakepit for single command
- `SNAKEPIT_INTERCEPT=0` - Disable Python hooks
- `SNAKEPIT_AUTO_TEST=0` - Disable automatic testing (validate only)
- `SNAKEPIT_VERBOSE=1` - Enable verbose logging
- `SNAKEPIT_QUIET=1` - Suppress welcome messages

### Example Usage

```bash
# Validate but don't install
SNAKEPIT_AUTO_TEST=0 pip install somepackage

# Verbose mode with testing
SNAKEPIT_VERBOSE=1 pip install somepackage

# Quick system install without validation
SNAKEPIT_BYPASS=1 pip install trusted-package
```

## Helper Commands

### Status and Configuration

```bash
# Show current status
snakepit-status

# Run diagnostic tests
snakepit-test

# Enable routing
snakepit-enable

# Disable routing
snakepit-disable
```

### Hook Management

```bash
# Install Python hooks
snakepit-install-hooks

# Uninstall Python hooks
snakepit-uninstall-hooks
```

## Direct CLI Usage

For explicit control, use the snakepit CLI directly:

```bash
# Install with validation
python3 /home/adminx/snakepit/snakepit_cli.py install requests

# Validate without installing
python3 /home/adminx/snakepit/snakepit_cli.py validate requests --verbose

# Check status of package
python3 /home/adminx/snakepit/snakepit_cli.py status --package requests

# View history
python3 /home/adminx/snakepit/snakepit_cli.py history --limit 10

# Cleanup sandboxes
python3 /home/adminx/snakepit/snakepit_cli.py cleanup
```

## Configuration File

Edit `snakepit.toml` to configure behavior:

```toml
[handler]
sandbox_dir = "/tmp/snakepit-sandbox"
container_image = "python:3.11-slim"
validation_timeout = 60
max_retries = 3
auto_install = true
dry_run = false
verbose = true
log_level = "INFO"
```

## Validation Process

When you install a package through snakepit, it follows this workflow:

```
┌─────────────────────────────────────────────────────────┐
│ 1. INGEST                                               │
│    Download package into isolated container/venv        │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ 2. TEST/COLLABORATE                                     │
│    - Import test                                        │
│    - Basic functionality checks                         │
│    - Custom test script (if provided)                   │
└─────────────────────────────────────────────────────────┘
                        ↓
                  Pass / Fail?
                        ↓
        ┌───────────────┴───────────────┐
        │                               │
        ↓ FAIL                          ↓ PASS
┌─────────────────┐         ┌─────────────────────────┐
│ 3. KILL/DESTROY │         │ 4. CONSCRIPT/INSTALL    │
│    Remove sandbox│         │    Install to system    │
└─────────────────┘         └─────────────────────────┘
```

## Troubleshooting

### "Handler not found" error

```bash
# Check if files exist
ls -la /home/adminx/snakepit/snakepit_handler.py
ls -la /home/adminx/snakepit/snakepit_cli.py

# Make sure Python path is correct
python3 --version
```

### Python hooks not working

```bash
# Check if sitecustomize.py exists
python3 -c "import site; print(site.getsitepackages()[0])"
ls -la $(python3 -c "import site; print(site.getsitepackages()[0])")/sitecustomize.py

# Reinstall hooks
snakepit-uninstall-hooks
snakepit-install-hooks
```

### Container/venv sandbox issues

```bash
# Check if container engine is available
docker --version
# or
podman --version

# If neither available, snakepit will use venv sandbox
# Make sure venv module is available
python3 -m venv --help
```

### Permission errors

```bash
# For site-packages installation, may need sudo
sudo snakepit-install-hooks

# Or use user installation directory
pip install --user <package>
```

## Advanced Configuration

### Using with Virtual Environments

Snakepit works with virtual environments:

```bash
# Create venv
python3 -m venv myenv
source myenv/bin/activate

# Snakepit will automatically detect and use the active venv
pip install requests
```

### Custom Container Images

Edit `snakepit.toml`:

```toml
[handler]
container_image = "python:3.12-alpine"  # Use Alpine image
# or
container_image = "ubuntu:22.04"        # Use Ubuntu base
```

### Disable Validation for Trusted Sources

```bash
# Bypass validation for packages from private PyPI
SNAKEPIT_BYPASS=1 pip install --index-url https://private-pypi.example.com my-package
```

### Integration with CI/CD

```bash
# In your CI pipeline
export SNAKEPIT_AUTO_TEST=1
export SNAKEPIT_VERBOSE=1

# Install dependencies with validation
pip install -r requirements.txt
```

## Uninstallation

To completely remove snakepit routing:

```bash
# 1. Remove Python hooks
snakepit-uninstall-hooks

# 2. Remove from .bashrc
sed -i '/snakepit-shell-integration/d' ~/.bashrc

# 3. Unset environment variables
unset SNAKEPIT_HOME SNAKEPIT_BYPASS SNAKEPIT_INTERCEPT

# 4. Reload shell
exec bash
```

## FAQ

**Q: Does snakepit slow down installations?**
A: Yes, slightly. Validation adds overhead, but it prevents malicious or broken packages from being installed.

**Q: Can I trust packages validated by snakepit?**
A: Snakepit validates that packages import and have basic functionality. It doesn't perform deep security audits.

**Q: What happens if validation fails?**
A: The package is NOT installed, and the sandbox is cleaned up automatically.

**Q: Can I use this with conda/poetry?**
A: Currently snakepit primarily targets pip. Conda/poetry support is experimental.

**Q: How do I view validation history?**
A: Use `python3 snakepit_cli.py history --limit 20`

## Support

For issues or questions:
- Check logs in `~/.snakepit/package_history.json`
- Run `snakepit-test` for diagnostics
- Use `SNAKEPIT_VERBOSE=1` for detailed output
