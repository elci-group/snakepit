# Snakepit Routing System

## Overview

The Snakepit routing system ensures **all pip and Python installations** are validated through a smart backend before being installed on your system. This provides:

- **Security**: Packages are tested in isolated sandboxes before installation
- **Reliability**: Basic functionality validation catches broken packages
- **Transparency**: Full audit trail of all package operations
- **Control**: Easy enable/disable without modifying your system

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     USER INTERFACE                          │
│  Shell Commands (pip install X) or Python Code (pip.main) │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                  INTERCEPTION LAYER                         │
│                                                             │
│  ┌──────────────────┐    ┌──────────────────────────┐     │
│  │ Shell Wrapper    │    │ Python Hooks             │     │
│  │ (bash function)  │    │ (sitecustomize.py)       │     │
│  └──────────────────┘    └──────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│              SNAKEPIT HANDLER (Python)                      │
│                                                             │
│  Phase 1: INGEST      - Download to sandbox                │
│  Phase 2: TEST        - Validate functionality             │
│  Phase 3: DESTROY     - Clean up failed packages           │
│  Phase 4: CONSCRIPT   - Install approved packages          │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                 INSTALLATION BACKEND                        │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Rust Binary  │  │ Direct pip   │  │ Conda/Poetry │    │
│  │ (snakepit)   │  │ (fallback)   │  │ (future)     │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Installation

```bash
cd /home/adminx/snakepit
./install-routing.sh
```

This interactive script will:
1. Check dependencies
2. Make scripts executable  
3. Offer to add to ~/.bashrc
4. Offer to install Python hooks
5. Show next steps

### Manual Installation

If you prefer manual setup:

```bash
# 1. Add to ~/.bashrc
echo 'source /home/adminx/snakepit/snakepit-shell-integration.sh' >> ~/.bashrc

# 2. Reload shell
source ~/.bashrc

# 3. (Optional) Install Python hooks
snakepit-install-hooks

# 4. Verify
snakepit-status
```

## Components

### 1. Shell Wrapper (`snakepit-pip-wrapper.sh`)

Intercepts pip commands from the shell and routes them to the handler.

**What it does:**
- Parses pip command line arguments
- Extracts package names and versions
- Calls the snakepit CLI for validation
- Falls back to system pip for non-install commands

**Example:**
```bash
pip install requests
# ↓
snakepit-pip-wrapper.sh install requests
# ↓
python3 snakepit_cli.py install requests
```

### 2. Shell Integration (`snakepit-shell-integration.sh`)

Provides shell functions and environment setup.

**Features:**
- Wraps `pip` and `pip3` as bash functions
- Exports helper commands
- Provides enable/disable functions
- Status and diagnostic tools

**Helper Commands:**
- `pip-direct` - Bypass snakepit
- `snakepit-enable` - Enable routing
- `snakepit-disable` - Disable routing
- `snakepit-status` - Show status
- `snakepit-test` - Run diagnostics
- `snakepit-install-hooks` - Install Python hooks
- `snakepit-uninstall-hooks` - Remove Python hooks

### 3. Python Hooks (`snakepit_sitecustomize.py`)

Intercepts pip calls from within Python programs.

**What it does:**
- Hooks into Python's import system
- Wraps `pip.main()` function
- Intercepts subprocess calls to pip
- Routes to snakepit wrapper

**Installation:**
```bash
snakepit-install-hooks
```

This creates a symlink:
```
site-packages/sitecustomize.py -> /home/adminx/snakepit/snakepit_sitecustomize.py
```

### 4. Handler (`snakepit_handler.py`)

Core validation logic implementing the four-phase strategy.

**Phases:**

1. **INGEST** 📥
   - Creates isolated sandbox (container or venv)
   - Downloads package into sandbox
   - Prepares test environment

2. **TEST/COLLABORATE** 🧪
   - Runs import test
   - Checks basic functionality
   - Executes custom test script (if provided)
   - Validates package metadata

3. **KILL/DESTROY** 💀
   - Removes failed packages
   - Cleans up sandbox
   - Logs failure details

4. **CONSCRIPT/INSTALL** ⚔️
   - Installs approved packages
   - Uses Rust binary or direct pip
   - Updates dependency tracking
   - Cleans up sandbox after install

### 5. CLI (`snakepit_cli.py`)

Command-line interface for explicit package management.

**Commands:**

```bash
# Install with validation
snakepit-smart install requests

# Validate without installing
snakepit-smart validate numpy --verbose

# Check package status
snakepit-smart status --package requests

# View history
snakepit-smart history --limit 20

# Cleanup sandboxes
snakepit-smart cleanup

# Configuration
snakepit-smart config --show
snakepit-smart config --set key=value
```

## Usage

### Basic Installation

```bash
pip install requests
```

Output:
```
🐍 Snakepit: Processing requests through smart handler...
📥 INGEST: Starting ingestion of requests
✅ INGEST: Successfully ingested requests
🧪 TEST/COLLABORATE: Validating requests
✅ TEST/COLLABORATE: requests approved for installation
⚔️ CONSCRIPT: Installing requests
✅ Successfully installed requests
```

### Installation with Version

```bash
pip install numpy==1.24.0
```

### Dry Run (Validate Only)

```bash
# Using flag
pip install pandas --dry-run

# Using environment variable
SNAKEPIT_AUTO_TEST=0 pip install pandas
```

### Custom Test Script

```bash
pip install mypackage --test-script ./my_validation.py
```

Test script format:
```python
#!/usr/bin/env python3
import sys

def test_import():
    try:
        import mypackage
        return True
    except:
        return False

if __name__ == "__main__":
    sys.exit(0 if test_import() else 1)
```

### Bypass Snakepit

When you need to install without validation:

```bash
# One-time bypass
SNAKEPIT_BYPASS=1 pip install trusted-package

# Using helper function
pip-direct install trusted-package

# Disable completely
snakepit-disable
pip install package
snakepit-enable
```

## Environment Variables

### Configuration

| Variable | Values | Default | Description |
|----------|--------|---------|-------------|
| `SNAKEPIT_BYPASS` | 0/1 | 0 | Bypass validation for single command |
| `SNAKEPIT_INTERCEPT` | 0/1 | 1 | Enable/disable Python hooks |
| `SNAKEPIT_AUTO_TEST` | 0/1 | 1 | Disable auto-install (validate only) |
| `SNAKEPIT_VERBOSE` | 0/1 | 0 | Enable verbose logging |
| `SNAKEPIT_QUIET` | 0/1 | 0 | Suppress welcome messages |
| `SNAKEPIT_HOME` | path | auto | Snakepit installation directory |

### Examples

```bash
# Validate but don't install
SNAKEPIT_AUTO_TEST=0 pip install somepackage

# Verbose validation
SNAKEPIT_VERBOSE=1 pip install somepackage

# Bypass for trusted source
SNAKEPIT_BYPASS=1 pip install --index-url https://private/pypi package

# Disable Python hooks temporarily
SNAKEPIT_INTERCEPT=0 python script.py
```

## Configuration File

Edit `snakepit.toml`:

```toml
[handler]
# Sandbox configuration
sandbox_dir = "/tmp/snakepit-sandbox"
container_image = "python:3.11-slim"

# Timeouts and limits
validation_timeout = 60
max_retries = 3

# Behavior
auto_install = true
dry_run = false
verbose = true
log_level = "INFO"

# History tracking
history_file = "~/.snakepit/package_history.json"
```

## Sandbox Types

### Container Sandbox (Preferred)

Uses Docker or Podman for complete isolation.

**Pros:**
- Complete isolation
- Can't affect system
- Clean environment
- Fast cleanup

**Cons:**
- Requires Docker/Podman
- Slightly slower

**Auto-detection:**
```bash
# Checks for podman first, then docker
command -v podman || command -v docker
```

### Venv Sandbox (Fallback)

Uses Python venv for isolation.

**Pros:**
- No dependencies
- Fast creation
- Works everywhere

**Cons:**
- Less isolation
- Shares system packages
- Can be affected by system state

## Security Considerations

### What Snakepit Validates

✅ **Does Validate:**
- Package imports successfully
- Basic functionality works
- Package has expected attributes
- Installation completes without errors

❌ **Does NOT Validate:**
- Malicious code in package
- Security vulnerabilities
- Backdoors or trojans
- License compliance
- Code quality

### Recommendations

1. **Don't rely solely on snakepit for security** - Use additional tools:
   - `pip-audit` for vulnerabilities
   - `safety` for known issues
   - Code review for critical packages

2. **Use trusted sources** - Prefer packages from:
   - Official PyPI with maintainer verification
   - Well-known organizations
   - Packages with many users/stars

3. **Review before installation** - Check:
   - Package documentation
   - GitHub repository
   - Recent updates
   - Community reviews

4. **Use virtual environments** - Isolate projects:
   ```bash
   python3 -m venv myproject
   source myproject/bin/activate
   pip install packages
   ```

## Troubleshooting

### "Handler not found" Error

```bash
# Verify files exist
ls -la ~/snakepit/snakepit_handler.py
ls -la ~/snakepit/snakepit_cli.py

# Check PATH
echo $PATH | grep snakepit
```

**Solution:**
```bash
cd ~/snakepit
chmod +x snakepit_handler.py snakepit_cli.py
source ~/.bashrc
```

### Python Hooks Not Working

```bash
# Check installation
python3 -c "import site; print(site.getsitepackages()[0])"
ls -la $(python3 -c "import site; print(site.getsitepackages()[0])")/sitecustomize.py
```

**Solution:**
```bash
snakepit-uninstall-hooks
snakepit-install-hooks
```

### Container Issues

```bash
# Check container engine
docker --version
podman --version

# Test container
docker run --rm python:3.11-slim python --version
```

**Solution:** If no container engine, snakepit will automatically use venv sandbox.

### Permission Errors

```bash
# Check site-packages permissions
python3 -c "import site; import os; sp = site.getsitepackages()[0]; print(f'{sp}: writable={os.access(sp, os.W_OK)}')"
```

**Solution:**
```bash
# Use sudo for system Python
sudo snakepit-install-hooks

# Or install packages with --user
pip install --user package
```

### Validation Timeouts

If packages take too long to validate:

```toml
# Edit snakepit.toml
[handler]
validation_timeout = 120  # Increase timeout
```

### False Positives

Some packages may fail validation but work fine:

```bash
# Bypass validation for known-good packages
SNAKEPIT_BYPASS=1 pip install tricky-package

# Or use custom test script
pip install tricky-package --test-script custom_test.py
```

## Advanced Usage

### Integration with Virtual Environments

Snakepit detects and respects active virtual environments:

```bash
# Create venv
python3 -m venv myenv
source myenv/bin/activate

# Packages install to venv
pip install requests  # Validated, then installed to myenv
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Setup Snakepit
  run: |
    git clone https://github.com/user/snakepit.git
    cd snakepit
    ./install-routing.sh -y  # Auto-yes
    
- name: Install dependencies with validation
  run: |
    source ~/.bashrc
    export SNAKEPIT_VERBOSE=1
    pip install -r requirements.txt
```

### Custom Container Images

```toml
# snakepit.toml
[handler]
# Use Alpine for smaller images
container_image = "python:3.11-alpine"

# Or use custom image with dependencies
container_image = "myregistry/python-custom:latest"
```

### Batch Operations

Install multiple packages:

```bash
# Create batch file
cat > packages.txt <<EOF
requests==2.31.0
numpy==1.24.0
pandas>=1.5.0
EOF

# Install with validation
while read pkg; do
    pip install "$pkg"
done < packages.txt
```

### Monitoring

View audit trail:

```bash
# Recent installations
python3 ~/snakepit/snakepit_cli.py history --limit 10

# Failed installations
python3 ~/snakepit/snakepit_cli.py history --status failed

# Specific package
python3 ~/snakepit/snakepit_cli.py history --package numpy

# JSON output
python3 ~/snakepit/snakepit_cli.py history --limit 5 --json
```

## Performance

### Overhead

- **Shell interception**: ~5ms per command
- **Sandbox creation**: 2-5 seconds (container) or 0.5-1 second (venv)
- **Validation**: 1-10 seconds depending on package
- **Installation**: Same as normal pip

### Optimization Tips

1. **Disable for development**:
   ```bash
   snakepit-disable  # During dev work
   snakepit-enable   # When deploying
   ```

2. **Use validation cache** (future feature):
   ```toml
   [handler]
   cache_validated = true
   cache_ttl = 86400  # 24 hours
   ```

3. **Skip validation for trusted packages**:
   ```bash
   SNAKEPIT_BYPASS=1 pip install -r requirements-trusted.txt
   ```

## Uninstallation

Complete removal:

```bash
# 1. Remove Python hooks
snakepit-uninstall-hooks

# 2. Remove from ~/.bashrc
sed -i '/snakepit-shell-integration/d' ~/.bashrc

# 3. Clean environment
unset SNAKEPIT_HOME SNAKEPIT_BYPASS SNAKEPIT_INTERCEPT

# 4. Remove sandboxes
rm -rf /tmp/snakepit-sandbox

# 5. Remove history
rm -rf ~/.snakepit

# 6. Reload shell
exec bash
```

## Support

### Getting Help

1. **Check status**: `snakepit-status`
2. **Run diagnostics**: `snakepit-test`
3. **Enable verbose**: `SNAKEPIT_VERBOSE=1 pip install package`
4. **Check logs**: `cat ~/.snakepit/package_history.json`
5. **Read docs**: `less ~/snakepit/ROUTING_SETUP.md`

### Reporting Issues

When reporting issues, include:

```bash
# System info
uname -a
python3 --version
docker --version || podman --version

# Snakepit status
snakepit-status

# Recent history
python3 ~/snakepit/snakepit_cli.py history --limit 5

# Verbose output of failing command
SNAKEPIT_VERBOSE=1 pip install problematic-package 2>&1 | tee error.log
```

## Future Enhancements

- [ ] Validation result caching
- [ ] Integration with pip-audit
- [ ] Custom validation plugins
- [ ] Web dashboard for history
- [ ] Support for conda/poetry
- [ ] Signature verification
- [ ] Network isolation for sandboxes
- [ ] Parallel validation for multiple packages

---

**Version**: 1.0  
**Last Updated**: 2025  
**License**: MIT
