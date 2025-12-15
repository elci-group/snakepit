# Snakepit Dependency Conflict Resolver

## Overview

The Snakepit Dependency Conflict Resolver automatically detects and resolves pip dependency conflicts in a deterministic, intelligent manner. It addresses common issues like:

- **Missing dependencies**: `requires X, which is not installed`
- **Version conflicts**: `requires Y>=2.0, but you have Y 1.4`  
- **Incompatible dependencies**: `requires Z>=3.0, but you have Z 2.1 which is incompatible`

## Features

✅ **Automatic Detection** - Parses pip output to identify all conflicts  
✅ **Intelligent Resolution** - Creates deterministic resolution plans  
✅ **Priority-Based** - Handles missing deps first, then version mismatches  
✅ **Integrated** - Works seamlessly with snakepit handler  
✅ **Standalone** - Can be used independently via CLI or wrapper  

## Usage

### 1. Integrated with Snakepit Handler

The resolver is automatically integrated into `snakepit_handler.py`. When a package installation encounters conflicts, it will:

1. Detect the conflicts in pip output
2. Create a resolution plan
3. Automatically install/upgrade required packages
4. Retry the original installation

No configuration needed - it just works!

### 2. Smart Pip Wrapper

Use `smart-pip` as a drop-in replacement for pip:

```bash
# Install packages with automatic conflict resolution
~/snakepit/smart-pip install requests flask numpy

# Install from requirements file
~/snakepit/smart-pip install -r requirements.txt

# Create an alias for convenience
alias pip='~/snakepit/smart-pip'
```

### 3. Direct Python API

```python
from dependency_resolver import DependencyResolver

# Create resolver
resolver = DependencyResolver()

# Parse pip output
conflicts = resolver.parse_pip_output(pip_output)

# Create resolution plan
resolution = resolver.create_resolution_plan(conflicts)

# Apply resolution
resolver.apply_resolution(resolution, dry_run=False)

# Or do it all at once
resolver.auto_resolve_from_output(pip_output, dry_run=False)
```

### 4. Command Line Tool

```bash
# Analyze existing pip output
python3 dependency_resolver.py --analyze pip_output.txt

# Install with auto-resolution
python3 dependency_resolver.py --install requests flask

# Dry run to see what would be done
python3 dependency_resolver.py --install numpy --dry-run
```

## How It Works

### Detection Phase

The resolver uses regex patterns to parse pip error output:

```python
PATTERNS = {
    'missing': r'(\S+) \d+ requires ([\w\-]+), which is not installed',
    'version_mismatch': r'(\S+) \d+ requires ([\w\-]+)([><=!]+)([\d.]+), but you have (\w+) ([\d.]+)',
    'incompatible': r'(\S+) \d+ requires ([\w\-]+)([><=!]+)([\d.]+), but you have (\w+) ([\d.]+) which is incompatible'
}
```

### Resolution Phase

Creates a deterministic plan with two priority levels:

1. **Priority 1**: Missing dependencies (install new packages)
2. **Priority 2**: Version mismatches (upgrade/downgrade existing packages)

### Execution Phase

Applies the resolution plan in order:

```bash
pip install --upgrade constantly>=15.1
pip install --upgrade hyperlink>=17.1.1
pip install --upgrade incremental>=24.7.0
pip install --upgrade zope-interface>=5
pip install --upgrade werkzeug>=3.1.0
pip install --upgrade blinker>=1.9.0
pip install --upgrade markupsafe>=2.1.1
```

## Example: Your Conflict

Given your pip output:

```
twisted 25.5.0 requires constantly>=15.1, which is not installed.
twisted 25.5.0 requires hyperlink>=17.1.1, which is not installed.
twisted 25.5.0 requires incremental>=24.7.0, which is not installed.
twisted 25.5.0 requires zope-interface>=5, which is not installed.
flask 3.1.2 requires werkzeug>=3.1.0, which is not installed.
flask 3.1.2 requires blinker>=1.9.0, but you have blinker 1.4 which is incompatible.
flask 3.1.2 requires markupsafe>=2.1.1, but you have markupsafe 2.0.1 which is incompatible.
```

The resolver will:

1. **Detect 7 unique conflicts** (5 missing, 2 version mismatches)
2. **Create resolution plan**:
   - Install: constantly, hyperlink, incremental, zope-interface, werkzeug
   - Upgrade: blinker, markupsafe
3. **Execute installations in order**
4. **Verify success**

## Configuration

The resolver can be configured via the snakepit handler config:

```toml
[handler]
# Enable/disable automatic conflict resolution
auto_resolve_conflicts = true

# Timeout for each package installation
conflict_resolution_timeout = 300

# Dry run mode (test without installing)
dry_run = false
```

## Environment Variables

```bash
# Disable auto-resolution for one command
SMART_PIP_AUTO_RESOLVE=0 smart-pip install flask

# Enable dry-run mode
SMART_PIP_DRY_RUN=1 smart-pip install numpy
```

## Integration with Snakepit Phases

The resolver fits into the existing 4-phase strategy:

1. **Ingest** → Download package into sandbox
2. **Test/Collaborate** → Validate package
3. **Kill/Destroy** → Remove failed packages
4. **Conscript/Install** → **[Resolver activates here]** Install locally with auto-resolution

## Testing

Run the test script to verify functionality:

```bash
cd ~/snakepit
python3 test_resolver.py
```

This simulates your exact conflict scenario and shows the resolution plan.

## Files Added

- `dependency_resolver.py` - Core resolver implementation
- `smart-pip` - CLI wrapper for pip with auto-resolution
- `test_resolver.py` - Test script with your example
- `DEPENDENCY_RESOLVER.md` - This documentation

## Files Modified

- `snakepit_handler.py` - Integrated resolver into conscript phase

## Advantages

1. **Deterministic** - Always resolves conflicts the same way
2. **Transparent** - Shows exactly what it's doing
3. **Safe** - Can run in dry-run mode first
4. **Fast** - Parallel-compatible resolution planning
5. **Smart** - Understands priority (missing deps before upgrades)
6. **Integrated** - Works with existing snakepit infrastructure

## Future Enhancements

- [ ] Support for circular dependencies
- [ ] Machine learning for optimal version selection
- [ ] Conflict avoidance predictions before install
- [ ] Integration with virtual environment management
- [ ] Rollback support for failed resolutions
- [ ] Package compatibility matrix caching

## License

Part of the Snakepit project by adminx
