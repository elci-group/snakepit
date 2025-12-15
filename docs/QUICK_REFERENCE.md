# Snakepit Routing Quick Reference

## Installation

```bash
cd ~/snakepit
./install-routing.sh
source ~/.bashrc
```

## Common Commands

| Command | Description |
|---------|-------------|
| `pip install <package>` | Install with validation |
| `pip-direct install <package>` | Bypass validation |
| `snakepit-status` | Show configuration |
| `snakepit-enable` | Enable routing |
| `snakepit-disable` | Disable routing |
| `snakepit-test` | Run diagnostics |
| `snakepit-install-hooks` | Install Python hooks |

## Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `SNAKEPIT_BYPASS=1` | Skip validation once | `SNAKEPIT_BYPASS=1 pip install pkg` |
| `SNAKEPIT_AUTO_TEST=0` | Validate only, don't install | `SNAKEPIT_AUTO_TEST=0 pip install pkg` |
| `SNAKEPIT_VERBOSE=1` | Show detailed output | `SNAKEPIT_VERBOSE=1 pip install pkg` |

## Files Created

- `snakepit-pip-wrapper.sh` - Pip command wrapper
- `snakepit-shell-integration.sh` - Shell functions and helpers
- `snakepit_sitecustomize.py` - Python import hooks
- `snakepit_handler.py` - Core validation logic
- `snakepit_cli.py` - Command-line interface
- `install-routing.sh` - Installation script

## Validation Workflow

```
User runs: pip install requests

1. Shell wrapper intercepts command
2. Extracts package name and version
3. Calls snakepit handler:
   a. INGEST - Download to sandbox
   b. TEST - Validate functionality
   c. DESTROY if failed / INSTALL if passed
4. Returns result to user
```

## Troubleshooting

**Problem**: "Handler not found"  
**Solution**: `chmod +x ~/snakepit/*.py && source ~/.bashrc`

**Problem**: Hooks not working  
**Solution**: `snakepit-install-hooks`

**Problem**: Package fails validation  
**Solution**: `SNAKEPIT_BYPASS=1 pip install <package>`

## Documentation

- Full guide: `ROUTING_README.md`
- Setup details: `ROUTING_SETUP.md`
- Main README: `README.md`
