# Snakepit 🐍

A dynamic Rust-based Python dependency installer that provides intelligent package management with support for multiple backends (pip, conda, poetry) and virtual environment management.

## ✨ Features

- **Multi-backend Support**: Automatically detects and uses pip, conda, or poetry
- **Dynamic Dependency Resolution**: Resolves dependencies with version constraints from PyPI
- **Virtual Environment Management**: Create, activate, and manage Python virtual environments
- **Project Initialization**: Quick project setup with dependency management
- **Configuration Management**: Flexible configuration with TOML files
- **Beautiful CLI**: Colorized output with progress indicators
- **Cross-platform**: Works on Linux, macOS, and Windows
- **Shell Integration**: Bash integration with auto-install and command retry (new!)
- **Daemon Support**: Background process monitoring and automatic installation
- **Zero-config Auto-install**: Install missing packages on-demand without active venv
- **Zero Dependencies**: Core functionality implemented in pure Rust with no external crate dependencies for critical paths
- **Lightweight**: Minimal binary size and memory footprint

## Installation

### From Source

```bash
git clone <repository-url>
cd snakepit
cargo build --release
```

### Using Cargo

```bash
cargo install snakepit
```

## Shell Integration 🚀 (NEW!)

Snakepit integrates with bash for zero-friction Python development:

### Auto-Install Missing Packages

When you run a Python script with missing dependencies, snakepit automatically installs them and retries:

```bash
$ python3 script.py
# ✨ Missing module detected: requests
#    Installing via snakepit...
# ✅ Package installed. Retrying command...
# [script runs successfully]
```

### Setup (One-time)

```bash
# Add to ~/.bashrc (or run after installation)
source ~/.bashrc
```

This enables:
- `venv-activate [name]` - Quick venv activation
- `venv-create [name] [py-version]` - Create new venv
- `venv-list` - List all venvs
- `venv-deactivate` - Deactivate current venv
- `pip-snakepit [packages]` - Install in active venv
- `snakepit-info` - Show configuration
- **Auto-install on import errors** - No configuration needed

### How It Works

1. Run any Python command normally
2. If a module is missing, bash intercepts the error
3. Snakepit extracts the package name
4. Package is installed (to active venv or system Python)
5. Command is retried automatically

**Works without active venv** - Falls back to system Python if needed.

## Quick Start

### Initialize a New Project

```bash
snakepit init my-awesome-project
```

This creates a new project directory with:
- `snakepit.toml` - Project configuration
- `requirements.txt` - Dependency file
- Virtual environment (if configured)

### Install Packages

```bash
# Install a package
snakepit install requests

# Install with specific version
snakepit install numpy --version 1.21.0

# Install as development dependency
snakepit install pytest --dev
```

### Manage Virtual Environments

```bash
# Create a virtual environment
snakepit venv create my-env --python-version 3.9

# List virtual environments
snakepit venv list

# Activate a virtual environment
snakepit venv activate my-env

# Delete a virtual environment
snakepit venv delete my-env
```

### Sync Dependencies

```bash
# Sync from requirements.txt or pyproject.toml
snakepit sync
```

## Configuration

Snakepit supports configuration through TOML files:

### Global Configuration (`~/.config/snakepit/config.toml`)

```toml
default_backend = "pip"  # pip, conda, poetry
default_venv_backend = "venv"  # venv, virtualenv, conda, poetry
venv_path = "~/.snakepit/venvs"
cache_enabled = true
python_version = "3.9"
timeout = 30
retries = 3

[mirrors]
# Custom PyPI mirrors
mirrors = ["https://pypi.org/simple/"]
```

### Project Configuration (`snakepit.toml`)

```toml
name = "my-project"
version = "0.1.0"
description = "My awesome project"
python_version = "3.9"
backend = "pip"
venv_name = "my-project-env"

dependencies = [
    "requests>=2.25.0",
    "numpy>=1.21.0",
]

dev_dependencies = [
    "pytest>=6.0.0",
    "black>=21.0.0",
]

[scripts]
test = "pytest"
lint = "black ."
```

## Commands

### Package Management

- `snakepit install <package>` - Install a package
- `snakepit uninstall <package>` - Uninstall a package
- `snakepit list` - List installed packages
- `snakepit sync` - Sync dependencies from files

### Project Management

- `snakepit init [name]` - Initialize a new project
- `snakepit sync` - Sync project dependencies

### Virtual Environment Management

- `snakepit venv create <name>` - Create virtual environment
- `snakepit venv activate <name>` - Activate virtual environment
- `snakepit venv delete <name>` - Delete virtual environment
- `snakepit venv list` - List virtual environments

## Backend Support

### Pip Backend
- Uses system pip or virtual environment pip
- Supports requirements.txt and pyproject.toml
- Automatic virtual environment detection

### Conda Backend
- Uses conda for package management
- Supports conda environments
- Automatic conda environment detection

### Poetry Backend
- Uses poetry for dependency management
- Supports pyproject.toml
- Automatic poetry project detection

## Virtual Environment Backends

### venv (Default)
- Uses Python's built-in venv module
- Lightweight and fast
- Cross-platform support

### virtualenv
- Uses virtualenv package
- More features than venv
- Better compatibility with older Python versions

### conda
- Uses conda environments
- Better for scientific computing
- Supports multiple Python versions

### poetry
- Uses poetry's virtual environment management
- Integrated with poetry projects
- Automatic dependency resolution

## Examples

### Basic Usage

```bash
# Initialize a project
snakepit init my-project
cd my-project

# Install dependencies
snakepit install requests numpy pandas

# Install development dependencies
snakepit install pytest black --dev

# Sync all dependencies
snakepit sync
```

### Advanced Usage

```bash
# Create a virtual environment with specific Python version
snakepit venv create my-env --python-version 3.9

# Install packages in specific virtual environment
snakepit install requests --venv my-env

# List all virtual environments
snakepit venv list

# Delete a virtual environment
snakepit venv delete my-env
```

### Configuration Examples

```bash
# Set default backend to conda
snakepit config set default_backend conda

# Set custom virtual environment path
snakepit config set venv_path /custom/path/venvs

# Disable caching
snakepit config set cache_enabled false
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running

```bash
cargo run -- install requests
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Roadmap

- [x] Shell integration with bash (auto-install and retry)
- [x] Daemon process monitoring
- [ ] Zsh and Fish shell integration
- [ ] Python auto-install hooks via sitecustomize.py
- [ ] Plugin system for custom backends
- [ ] Dependency conflict resolution
- [ ] Package caching and offline support
- [ ] Integration with CI/CD systems (GitHub Actions, GitLab CI)
- [ ] GUI interface
- [ ] Package vulnerability scanning
- [ ] Automatic dependency updates
- [ ] Multi-language support (Node.js, Go, Rust, etc.)

## Troubleshooting

### Common Issues

1. **Virtual environment not found**: Make sure the virtual environment exists and is accessible
2. **Package installation fails**: Check your internet connection and PyPI availability
3. **Permission errors**: Ensure you have write permissions to the target directory
4. **Python not found**: Make sure Python is installed and in your PATH

### Debug Mode

Run with debug output:

```bash
RUST_LOG=debug snakepit install requests
```

### Getting Help

- Check the documentation
- Open an issue on GitHub
- Join our community discussions

---

Made with ❤️ in Rust 🦀
