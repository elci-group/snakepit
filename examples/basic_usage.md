# Basic Usage Examples

## Initialize a New Project

```bash
# Create a new project
snakepit init my-python-project

# This creates:
# - my-python-project/
#   - snakepit.toml (project configuration)
#   - requirements.txt (dependency file)
#   - venv/ (virtual environment, if configured)
```

## Install Packages

```bash
# Install a basic package
snakepit install requests

# Install with specific version
snakepit install numpy --version 1.21.0

# Install as development dependency
snakepit install pytest --dev
```

## Virtual Environment Management

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

## Sync Dependencies

```bash
# Sync from requirements.txt or pyproject.toml
snakepit sync
```

## List Installed Packages

```bash
# List all installed packages
snakepit list
```

## Configuration

Create a global configuration file at `~/.config/snakepit/config.toml`:

```toml
default_backend = "pip"
default_venv_backend = "venv"
venv_path = "~/.snakepit/venvs"
cache_enabled = true
python_version = "3.9"
```

## Project Configuration

Each project can have its own `snakepit.toml`:

```toml
name = "my-project"
version = "0.1.0"
description = "My awesome Python project"
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
```
