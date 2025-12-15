# Smart Snakepit Package Handler 🐍⚡

A comprehensive Python package management system implementing the four-phase package handling strategy: **Ingest**, **Test/Collaborate**, **Kill/Destroy**, and **Conscript/Install**. This system ensures safe package installation by validating packages in isolated sandboxes before they touch your local environment.

## 🏗️ Architecture Overview

The Smart Snakepit Package Handler implements a robust four-phase workflow:

1. **📥 INGEST**: Download and containerize packages in ephemeral sandboxes
2. **🧪 TEST/COLLABORATE**: Validate package functionality, imports, and compatibility  
3. **💀 KILL/DESTROY**: Remove failed packages and clean up sandboxes
4. **⚔️ CONSCRIPT/INSTALL**: Install validated packages locally and update dependency graph

```
Package Request
       ↓
   📥 INGEST (Sandbox Creation)
       ↓
   🧪 TEST/COLLABORATE (Validation)
       ↓
   ✅ Pass? ────────── ❌ Fail?
       ↓                  ↓
⚔️ CONSCRIPT          💀 KILL/DESTROY
   (Install)           (Cleanup)
```

## 🚀 Quick Start

### Initialize Configuration
```bash
# Create sample configuration files
./smart_snakepit.py init-config

# Show current configuration
python3 smart_config.py show
```

### Install a Package
```bash
# Install with smart validation
./smart_snakepit.py install requests --version 2.31.0

# Install with comprehensive validation
./smart_snakepit.py install numpy --validation-level comprehensive
```

### Validate Without Installing
```bash
# Test package safety without installing
./smart_snakepit.py validate pandas --comprehensive

# Quick validation check
./smart_snakepit.py validate click
```

### Run Demonstration
```bash
# See the system in action
./smart_snakepit.py demo
```

## 📦 Core Components

### 1. Core Handler (`snakepit_handler.py`)
The main `SnakepitHandler` class implementing the four-phase strategy:

```python
from snakepit_handler import SnakepitHandler

handler = SnakepitHandler()
success = handler.handle_package("requests", "2.31.0")
```

**Key Features:**
- Container-based sandboxing (Docker/Podman)
- Virtual environment fallback
- Comprehensive error handling
- Package metadata tracking
- Automatic cleanup on failure

### 2. CLI Interface (`snakepit_cli.py`)
Full-featured command-line interface:

```bash
# Available commands
snakepit-smart install <package> [--version VERSION] [--test-script SCRIPT]
snakepit-smart validate <package> [--version VERSION] [--test-script SCRIPT]  
snakepit-smart status [--package PACKAGE]
snakepit-smart history [--package PACKAGE] [--status STATUS] [--limit N]
snakepit-smart cleanup [--package PACKAGE]
snakepit-smart config [--show | --set KEY=VALUE]
```

### 3. Validation Framework (`validation_framework.py`)
Advanced package testing with multiple validation levels:

```python
from validation_framework import ValidationFramework, ValidationLevel

framework = ValidationFramework()
result = framework.validate_package("numpy", "/path/to/sandbox", ValidationLevel.COMPREHENSIVE)
```

**Validation Levels:**
- `BASIC`: Import testing only
- `STANDARD`: Import + basic functionality 
- `COMPREHENSIVE`: Full test suite + performance
- `SECURITY`: Security-focused scanning
- `PERFORMANCE`: Performance benchmarking

**Security Features:**
- Pattern matching for dangerous imports
- Network access detection
- File system access analysis
- Dynamic code execution scanning

### 4. Configuration Management (`smart_config.py`)
Flexible configuration system supporting TOML files and environment variables:

```python
from smart_config import SmartConfig

config = SmartConfig()
project_config = config.load_project_config()
global_config = config.load_global_config()
```

## ⚙️ Configuration

### Project Configuration (`snakepit.toml`)
```toml
name = "my-project"
version = "0.1.0"
description = "My awesome project"
python_version = "3.11"
backend = "pip"

dependencies = [
    "requests>=2.31.0",
    "click>=8.0.0",
]

dev_dependencies = [
    "pytest>=7.0.0",
    "black>=23.0.0",
]

[scripts]
test = "pytest"
format = "black ."
lint = "flake8 ."

[handler]
validation_timeout = 120
security_scan = true
auto_install = true
sandbox_dir = "/tmp/snakepit-sandbox"
container_image = "python:3.11-slim"
```

### Global Configuration (`~/.config/snakepit/config.toml`)
```toml
[handler]
validation_timeout = 60
max_retries = 3
auto_install = true
dry_run = false
verbose = true
log_level = "INFO"
security_scan = true
allow_network_packages = true
allow_file_system_packages = true
parallel_validation = false
max_concurrent = 4
cache_validation_results = true
integrate_with_rust = true
```

### Environment Variables
- `SNAKEPIT_SANDBOX_DIR`: Override sandbox directory
- `SNAKEPIT_CONTAINER_IMAGE`: Container image for sandboxing
- `SNAKEPIT_TIMEOUT`: Validation timeout in seconds
- `SNAKEPIT_DRY_RUN`: Enable dry-run mode
- `SNAKEPIT_VERBOSE`: Enable verbose logging
- `SNAKEPIT_LOG_LEVEL`: Set logging level

## 🐳 Container Integration

The system automatically detects and uses available container engines:

### Docker
```bash
# Automatically detected if available
docker --version
```

### Podman  
```bash
# Preferred over Docker if both available
podman --version
```

### Virtual Environment Fallback
If no container engine is available, the system falls back to isolated Python virtual environments.

## 🦀 Rust Integration

Smart Snakepit integrates with the existing Rust snakepit binary:

```bash
# Auto-detection of Rust binary
./target/release/snakepit
./target/debug/snakepit
snakepit  # in PATH
```

**Integration Commands:**
```bash
# Use Rust backend for final installation
cargo run -- install requests
```

## 🔒 Security Features

### Sandbox Isolation
- Ephemeral containers/venvs
- No network access to host
- Isolated file system
- Automatic cleanup

### Security Scanning
- Pattern matching for dangerous imports:
  - `os.system`, `subprocess` calls
  - `eval`, `exec` dynamic execution
  - Network access patterns
  - File system manipulation

### Validation Pipeline
- Import safety testing
- Functionality verification
- Performance benchmarking
- Security pattern analysis

## 📊 Usage Examples

### Basic Package Installation
```bash
# Install popular packages safely
./smart_snakepit.py install requests
./smart_snakepit.py install numpy --version 1.24.0
./smart_snakepit.py install django --validation-level comprehensive
```

### Validation Testing
```bash
# Test packages without installing
./smart_snakepit.py validate tensorflow --comprehensive
./smart_snakepit.py validate unknown-package  # Will safely fail
```

### Status Monitoring
```bash
# Check system status
./smart_snakepit.py status

# View package history
python3 snakepit_cli.py history --limit 10

# Monitor specific package
python3 snakepit_cli.py status --package requests
```

### Configuration Management
```bash
# Show current config
python3 smart_config.py show

# Set configuration values
python3 smart_config.py set validation_timeout 180 --global
python3 smart_config.py set name "my-project"
```

### Advanced Usage
```python
#!/usr/bin/env python3
from smart_snakepit import SmartSnakepitIntegration

# Initialize system
integration = SmartSnakepitIntegration()
integration.initialize()

# Install package with custom validation
success = integration.handle_package_smart(
    package_name="scikit-learn",
    version="1.3.0",
    validation_level="comprehensive"
)

# Validate without installing
result = integration.validate_only("tensorflow", "2.13.0")
print(f"Validation {'passed' if result['success'] else 'failed'}")

# Generate status report
report = integration.status_report()
print(f"Active packages: {report['active_packages']}")

# Cleanup
integration.cleanup_all()
```

## 🔧 Troubleshooting

### Common Issues

#### Container Engine Not Found
```bash
# Install Docker
sudo apt-get install docker.io

# Install Podman
sudo apt-get install podman

# Or use venv fallback (automatic)
```

#### Package Validation Fails
```bash
# Check detailed logs
./smart_snakepit.py validate PACKAGE --verbose

# Try different validation level
./smart_snakepit.py validate PACKAGE --validation-level basic

# Check package history
python3 snakepit_cli.py history --package PACKAGE
```

#### Sandbox Issues
```bash
# Clean up all sandboxes
python3 snakepit_cli.py cleanup

# Check sandbox directory
ls -la /tmp/snakepit-sandbox/

# Reset configuration
./smart_snakepit.py init-config
```

### Debug Mode
```bash
# Enable verbose logging
export SNAKEPIT_VERBOSE=true
export SNAKEPIT_LOG_LEVEL=DEBUG

# Run with maximum verbosity
./smart_snakepit.py install PACKAGE --verbose
```

## 🧪 Testing & Development

### Run Tests
```bash
# Basic functionality test
python3 -c "import snakepit_handler; print('✅ Core handler OK')"
python3 -c "import validation_framework; print('✅ Validation framework OK')"
python3 -c "import smart_config; print('✅ Configuration OK')"

# Run demonstration
./smart_snakepit.py demo
```

### Development Setup
```bash
# Install dependencies
pip install toml

# Make scripts executable
chmod +x smart_snakepit.py
chmod +x snakepit_handler.py
chmod +x snakepit_cli.py

# Create test configuration
./smart_snakepit.py init-config
```

## 📈 Performance & Monitoring

### Metrics Collected
- Package ingestion time
- Validation duration
- Installation success rate
- Error patterns
- Container resource usage

### History Tracking
All package operations are logged to `~/.snakepit/package_history.json`:

```json
{
  "name": "requests",
  "version": "2.31.0", 
  "status": "conscripted",
  "ingest_time": 1698876543.123,
  "test_time": 1698876545.456,
  "install_time": 1698876547.789,
  "validation_results": { ... },
  "error_log": [],
  "success_log": ["Package validation successful", "Package installed successfully"]
}
```

## 🔮 Future Enhancements

- [ ] Zsh and Fish shell integration
- [ ] Multi-language support (Node.js, Go, Rust packages)  
- [ ] Plugin system for custom validators
- [ ] Dependency conflict resolution
- [ ] Package vulnerability scanning
- [ ] CI/CD integration (GitHub Actions, GitLab CI)
- [ ] Web dashboard for monitoring
- [ ] Machine learning for package risk assessment

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality  
4. Ensure all existing tests pass
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

Built on top of the existing Rust-based Snakepit package manager, extending it with advanced safety and validation capabilities.

---

**Smart Snakepit Package Handler** - Making Python package management safer, one validation at a time! 🐍✨