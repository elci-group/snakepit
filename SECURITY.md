# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Snakepit, please email security@example.com instead of using the issue tracker.

Please include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if applicable)

We take security seriously and will respond promptly to all security reports.

## Security Best Practices

When using Snakepit, follow these security best practices:

### Virtual Environments

Always use virtual environments to isolate project dependencies:

```bash
venv-create my-project
venv-activate my-project
```

This prevents dependency conflicts and contains potential security issues to a single environment.

### Package Verification

- Pin specific versions in requirements.txt for reproducible installs
- Review package sources and maintainers before installing
- Check package release notes for security updates

### Configuration Files

- Keep `~/.config/snakepit/config.toml` with restricted permissions (600)
- Don't commit snakepit configuration files to version control
- Use `.gitignore` to exclude sensitive files

### Daemon Security

When running Snakepit daemon:

- Only enable auto-install for trusted environments
- Review whitelist/blacklist settings for modules
- Monitor daemon logs for suspicious activity

## Supported Versions

Snakepit follows semantic versioning:
- Major versions: Breaking changes
- Minor versions: New features
- Patch versions: Security fixes and bug fixes

Security updates are released for the latest stable version.

## Security Considerations

### Automatic Installation

The auto-install feature bypasses explicit user confirmation. Use it only in:
- Personal development environments
- Trusted CI/CD pipelines
- Isolated virtual environments

### Python Package Index

Snakepit downloads packages from PyPI by default. While PyPI has security measures:
- Verify package integrity when possible
- Use private/mirror registries for sensitive packages
- Consider air-gapped deployment for critical systems

### Permissions

Ensure appropriate file permissions:

```bash
# Configuration (read/write owner only)
chmod 600 ~/.config/snakepit/config.toml

# Virtual environments (user readable/writable)
chmod 755 ~/.snakepit/venvs/my-env
```

## Dependencies

Snakepit depends on Rust crates from crates.io. We use:
- `cargo audit` to check for known vulnerabilities
- Pinned dependency versions in Cargo.toml
- Regular updates to dependencies

Check for vulnerable dependencies:

```bash
cargo audit
```

## Questions?

For security questions or concerns, please reach out to the maintainers directly.

---

Last updated: 2025-10-21
