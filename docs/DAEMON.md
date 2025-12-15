# Snakepit Daemon 🐍

The Snakepit Daemon is an intelligent background service that automatically detects when Python processes fail due to missing modules and installs them on-the-fly.

## Features

- **Automatic Detection**: Monitors Python processes for `ModuleNotFoundError` and `ImportError` exceptions
- **Smart Installation**: Automatically installs missing modules using the configured backend (pip/conda/poetry)
- **Process Monitoring**: Real-time monitoring of Python processes with configurable check intervals
- **Security Controls**: Whitelist/blacklist modules, configurable installation limits
- **System Integration**: systemd service support for auto-start on boot
- **Comprehensive Logging**: Detailed logging of all daemon activities

## Quick Start

### Start the Daemon

```bash
# Start in foreground (for testing)
snakepit daemon start

# Start in background
snakepit daemon start --daemon
```

### Check Status

```bash
snakepit daemon status
```

### Stop the Daemon

```bash
snakepit daemon stop
```

## Installation as System Service

### Automatic Installation

```bash
# Install as systemd service (requires sudo)
sudo ./install-daemon.sh
```

This will:
- Build and install the snakepit binary
- Create systemd service file
- Set up configuration directories
- Enable auto-start on boot

### Manual Installation

1. **Copy the service file:**
   ```bash
   sudo cp snakepit-daemon.service /etc/systemd/system/
   ```

2. **Reload systemd:**
   ```bash
   sudo systemctl daemon-reload
   ```

3. **Enable the service:**
   ```bash
   sudo systemctl enable snakepit-daemon
   ```

4. **Start the service:**
   ```bash
   sudo systemctl start snakepit-daemon
   ```

## Configuration

### Daemon Configuration File

The daemon configuration is stored in `~/.config/snakepit/daemon.toml`:

```toml
enabled = true
auto_install = true
check_interval = 5
max_install_attempts = 3
whitelist_modules = []
blacklist_modules = ["sys", "os", "builtins"]
log_file = "/home/user/.config/snakepit/daemon.log"
pid_file = "/home/user/.config/snakepit/snakepit.pid"
```

### Configuration Commands

```bash
# Show current configuration
snakepit daemon config show

# Set configuration values
snakepit daemon config set auto_install true
snakepit daemon config set check_interval 10
snakepit daemon config set max_install_attempts 5

# Reset to defaults
snakepit daemon config reset
```

## How It Works

### Process Monitoring

The daemon continuously monitors system processes to identify Python applications. It uses multiple detection methods:

1. **Process Name Detection**: Identifies processes with "python" in their name
2. **Command Line Analysis**: Scans command-line arguments for Python interpreters
3. **Error Pattern Matching**: Monitors stderr output for import error patterns

### Error Detection

The daemon detects missing modules through:

- **ModuleNotFoundError**: `ModuleNotFoundError: No module named 'requests'`
- **ImportError**: `ImportError: No module named 'numpy'`
- **Import Name Errors**: `ImportError: cannot import name 'function'`

### Auto-Installation Process

When a missing module is detected:

1. **Validation**: Checks if the module is in the whitelist/blacklist
2. **Installation**: Attempts to install using the configured backend
3. **Logging**: Records the installation attempt and result
4. **Retry Logic**: Implements exponential backoff for failed installations

## Security Features

### Module Filtering

- **Blacklist**: Prevents installation of system modules (`sys`, `os`, `builtins`)
- **Whitelist**: Only allows installation of approved modules (if configured)
- **Attempt Limits**: Prevents infinite retry loops

### Permission Controls

- **User Isolation**: Each user has their own daemon instance
- **Environment Detection**: Automatically detects and uses virtual environments
- **Backend Selection**: Uses the most appropriate package manager

## Monitoring and Logging

### Status Monitoring

```bash
# Check daemon status
snakepit daemon status

# View systemd service status
sudo systemctl status snakepit-daemon

# View logs
sudo journalctl -u snakepit-daemon -f
```

### Log Files

- **Daemon Log**: `~/.config/snakepit/daemon.log`
- **Error Log**: `/tmp/snakepit-errors.log`
- **System Log**: `journalctl -u snakepit-daemon`

### Testing

```bash
# Test missing module detection
snakepit daemon test requests

# Test with specific module
snakepit daemon test numpy
```

## Advanced Configuration

### Custom Backends

The daemon automatically detects and uses the best available backend:

- **pip**: Default Python package manager
- **conda**: Anaconda/Miniconda package manager
- **poetry**: Poetry dependency manager

### Virtual Environment Support

The daemon automatically detects and works within virtual environments:

- **venv**: Python's built-in virtual environment
- **virtualenv**: Third-party virtual environment tool
- **conda**: Conda environments
- **poetry**: Poetry-managed environments

### Performance Tuning

```toml
# Reduce monitoring frequency for better performance
check_interval = 10

# Limit installation attempts
max_install_attempts = 2

# Disable auto-installation for manual control
auto_install = false
```

## Troubleshooting

### Common Issues

1. **Daemon not starting**: Check permissions and systemd service status
2. **Modules not installing**: Verify backend configuration and network connectivity
3. **High CPU usage**: Increase `check_interval` in configuration
4. **Permission errors**: Ensure proper user permissions for package installation

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug snakepit daemon start

# Check daemon logs
tail -f ~/.config/snakepit/daemon.log
```

### Service Management

```bash
# Restart daemon
snakepit daemon restart

# Stop and start systemd service
sudo systemctl restart snakepit-daemon

# Disable auto-start
sudo systemctl disable snakepit-daemon
```

## Integration Examples

### With Development Workflows

```bash
# Start daemon for development session
snakepit daemon start

# Run Python script (missing modules auto-installed)
python my_script.py

# Stop daemon when done
snakepit daemon stop
```

### With CI/CD Pipelines

```bash
# Start daemon in CI environment
snakepit daemon start --daemon

# Run tests (dependencies auto-installed)
python -m pytest

# Clean up
snakepit daemon stop
```

### With Jupyter Notebooks

```bash
# Start daemon
snakepit daemon start

# Launch Jupyter (missing packages auto-installed)
jupyter notebook

# Import statements will trigger auto-installation
import requests  # Auto-installed if missing
import numpy     # Auto-installed if missing
```

## Best Practices

1. **Use Virtual Environments**: Always work within virtual environments for isolation
2. **Configure Whitelists**: Use whitelists in production environments for security
3. **Monitor Logs**: Regularly check daemon logs for issues
4. **Test Configuration**: Use `snakepit daemon test` to verify setup
5. **Backup Configuration**: Keep backups of daemon configuration files

## Security Considerations

- **Module Validation**: Only install modules from trusted sources
- **Network Security**: Ensure secure network connections for package downloads
- **User Permissions**: Run daemon with minimal required permissions
- **Audit Logging**: Monitor installation logs for suspicious activity

---

The Snakepit Daemon transforms Python development by eliminating the friction of missing dependencies, making your development workflow truly seamless! 🚀

