//! User Guide for Snakepit

# Snakepit User Guide

## Table of Contents

1. [Getting Started](#getting-started)
2. [Package Management](#package-management)
3. [Virtual Environments](#virtual-environments)
4. [Project Workflows](#project-workflows)
5. [SnakeEgg: Organic Evolution](#snakeegg-organic-evolution)
6. [Configuration](#configuration)
7. [Troubleshooting](#troubleshooting)

---

## Getting Started

### Installation

**From Source:**
```bash
git clone https://github.com/elci-group/snakepit.git
cd snakepit
cargo build --release

# Binary will be at: target/release/snakepit
# Add to PATH or create symlink
sudo ln -s $(pwd)/target/release/snakepit /usr/local/bin/snakepit
```

**Verify Installation:**
```bash
snakepit --version
# snakepit 0.1.0
```

### 5-Minute Quickstart

```bash
# 1. Create a new project
snakepit init my-project
cd my-project

# 2. Install packages (like pip)
snakepit install requests pandas numpy

# 3. Create virtual environment
snakepit venv create my-env --python 3.11

# 4. Activate (sets up shell)
source ./venv  activator script

# 5. Run your code
python script.py
```

---

## Package Management

### Installing Packages

**Basic Installation:**
```bash
# Single package
snakepit install requests

# Multiple packages
snakepit install requests numpy pandas flask
```

**Version Constraints:**
```bash
# Specific version
snakepit install numpy==1.21.0

# Version range
snakepit install "requests>=2.25.0,<3.0.0"

# Latest compatible
snakepit install flask~=2.0
```

**Development Dependencies:**
```bash
snakepit install pytest black mypy --dev
```

### Uninstalling Packages

**Safe Uninstall with Snapshot:**
```bash
snakepit uninstall pandas --snapshot
# Creates rollback point before removal
```

**Force Uninstall:**
```bash
snakepit uninstall pandas --force
# Removes without snapshot (use cautiously)
```

**Restore from Snapshot:**
```bash
snakepit restore pandas_2025-12-15_20-00
```

### Listing Packages

```bash
# List all installed
snakepit list

# Show package details
snakepit show requests

# Search PyPI
snakepit search "machine learning"
```

### Syncing Dependencies

```bash
# From requirements.txt
snakepit sync

# From pyproject.toml
snakepit sync --from pyproject.toml

# From Pipfile
snakepit sync --from Pipfile
```

---

## Virtual Environments

### Creating Environments

```bash
# Default Python version
snakepit venv create my-env

# Specific Python version
snakepit venv create my-env --python 3.11

# With initial packages
snakepit venv create my-env --with requests,flask
```

### Managing Environments

```bash
# List all environments
snakepit venv list

# Activate environment
snakepit venv activate my-env

# Deactivate
deactivate  # (standard Python venv command)

# Delete environment
snakepit venv delete my-env
```

---

## Project Workflows

### Initializing Projects

```bash
snakepit init my-project
```

Creates:
```
my-project/
├── snakepit.toml       # Project configuration
├── requirements.txt    # Dependencies
├── .gitignore         # Sensible defaults
└── README.md          # Basic documentation
```

### Project Configuration (snakepit.toml)

```toml
name = "my-project"
version = "0.1.0"
description = "My awesome project"
python_version = "3.11"

dependencies = [
    "requests>=2.25.0",
    "pandas>=1.3.0",
]

dev_dependencies = [
    "pytest>=6.0.0",
    "black>=21.0.0",
]

[scripts]
test = "pytest tests/"
lint = "black . && mypy ."
```

### Running Scripts

```bash
snakepit run test
# Executes: pytest tests/

snakepit run lint
# Executes: black . && mypy .
```

---

## SnakeEgg: Organic Evolution

**Note:** These features are optional and for advanced use cases.

### Initializing a Nest

```bash
snakepit nest init
# Creates quantum nest structure
```

### Creating Eggs

**Dual Egg (Python + Rust):**
```bash
snakepit egg create web_api --species Service --type dual
```

**Organic Only (Python):**
```bash
snakepit egg create data_processor --species Worker --type organic
```

### Evolving Eggs

```bash
# Single evolution cycle
snakepit egg evolve web_api

# Continuous evolution (watch mode)
snakepit egg evolve web_api --watch

# Multiple eggs
snakepit egg evolve-all
```

### Checking Status

```bash
snakepit egg status web_api
# Output:
# 🥚 web_api (Fetus stage, 67% complete)
#    Temperature: 72°C 🔥
#    Fitness: 0.84
#    Organic: 847 lines Python
#    Metallic: 923 lines Rust
#    Last evolved: 5 minutes ago
```

### Heat Sharing (Clutches)

```bash
# Create clutch (egg group)
snakepit clutch create backend-services

# Add eggs to clutch
snakepit clutch add backend-services web_api auth_handler db_manager

# Trigger thermal cycle (knowledge sharing)
snakepit clutch thermal-cycle backend-services
```

### Quantum Storage

```bash
# Vacuum idle eggs (evaporate to git)
snakepit nest vacuum --max-idle 24h

# Checkpoint all eggs (save to git)
snakepit nest checkpoint

# Observe egg (materialize from git)
snakepit nest observe web_api
```

---

## Configuration

### Global Configuration

**Location:** `~/.config/snakepit/config.toml`

```toml
# Default backend for package operations
default_backend = "pip"  # pip, conda, poetry

# Virtual environment settings
default_venv_backend = "venv"  # venv, virtualenv, conda
venv_path = "~/.snakepit/venvs"

# Performance
cache_enabled = true
parallel_downloads = 4

# Python version
python_version = "3.11"

# PyPI mirrors
[mirrors]
mirrors = ["https://pypi.org/simple/"]

# AI settings (for SnakeEgg)
[ai]
provider = "google"  # google, openai, anthropic
model = "gemini-2.0-flash"
api_key_env = "GEMINI_API_KEY"
```

### Project Configuration

**Location:** `<project>/snakepit.toml`

See [Project Workflows](#project-workflows) section above.

---

## Troubleshooting

### Common Issues

**1. Package installation fails**

```bash
# Check network connection
ping pypi.org

# Try with verbose output
snakepit install requests --verbose

# Use specific mirror
snakepit install requests --index-url https://pypi.org/simple/
```

**2. Virtual environment not found**

```bash
# List all environments
snakepit venv list

# Recreate environment
snakepit venv delete my-env
snakepit venv create my-env
```

**3. Dependency resolution conflict**

```bash
# Show resolution details
snakepit resolve --explain

# Force resolution (use cautiously)
snakepit install package --no-deps
```

**4. Permission errors**

```bash
# Install in user directory (no sudo)
snakepit install package --user

# Or use virtual environment
snakepit venv create my-env
snakepit venv activate my-env
snakepit install package
```

### Debug Mode

```bash
# Enable debug output
RUST_LOG=debug snakepit install requests

# Trace-level logging
RUST_LOG=trace snakepit resolve
```

### Getting Help

```bash
# General help
snakepit --help

# Command-specific help
snakepit install --help
snakepit egg evolve --help
```

### Reporting Issues

If you encounter bugs:

1. Check [GitHub Issues](https://github.com/elci-group/snakepit/issues)
2. Run with `--verbose` flag
3. Include error output
4. Provide system info (`snakepit --version`, Python version, OS)

---

## Next Steps

- Read [Architecture Documentation](./ARCHITECTURE.md)
- Explore [Examples](../examples/)
- Join [Discussions](https://github.com/elci-group/snakepit/discussions)
- Review [Graduate Report](../Snakepit_Graduate_Report.pdf) for deep dive

---

**Happy coding with Snakepit! 🐍**
