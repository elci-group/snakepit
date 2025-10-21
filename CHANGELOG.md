# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Bash shell integration with auto-install and command retry
- Automatic module extraction from Python error messages
- Smart package name mapping (PIL → Pillow, cv2 → opencv-python, etc.)
- Zero-config auto-install: works with or without active venv
- Shell helper functions: `venv-activate`, `venv-create`, `venv-list`, etc.
- Daemon process monitoring and background auto-install
- Configuration tracking for active virtual environments

### Changed
- Improved error message handling for better user experience
- Enhanced shell integration to support system Python fallback

### Fixed
- Module name extraction from various error message formats
- Import name to package name mapping for common packages

## [0.1.0] - 2025-10-21

### Added
- Initial release of Snakepit
- Multi-backend support (pip, conda, poetry)
- Dynamic dependency resolution from PyPI
- Virtual environment management (create, activate, delete, list)
- Project initialization with configuration
- Configuration management via TOML files
- Beautiful CLI with colorized output and progress indicators
- Cross-platform support (Linux, macOS, Windows)
- Daemon support for background process monitoring
- Test mode for simulating missing module detection

### Features
- `snakepit install` - Install packages
- `snakepit uninstall` - Uninstall packages
- `snakepit list` - List installed packages
- `snakepit sync` - Sync dependencies from files
- `snakepit init` - Initialize new projects
- `snakepit venv` - Virtual environment management
- `snakepit daemon` - Daemon process management
- Global and project-level configuration

[Unreleased]: https://github.com/adminx/snakepit/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/adminx/snakepit/releases/tag/v0.1.0
