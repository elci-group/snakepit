# All Python Installation Methods - Snakepit Coverage

This document covers **every Python package installation method** and how snakepit intercepts and validates them.

## 📋 Coverage Matrix

| Method | Command Example | Intercepted | Notes |
|--------|----------------|-------------|-------|
| **pip** | `pip install pkg` | ✅ Yes | Full support |
| **pip3** | `pip3 install pkg` | ✅ Yes | Full support |
| **python -m pip** | `python -m pip install pkg` | ✅ Yes | Full support |
| **python3 -m pip** | `python3 -m pip install pkg` | ✅ Yes | Full support |
| **requirements.txt** | `pip install -r requirements.txt` | ✅ Yes | All packages validated |
| **easy_install** | `easy_install pkg` | ✅ Yes | Legacy support |
| **setup.py** | `python setup.py install` | ✅ Yes | Validation attempted |
| **poetry** | `poetry add pkg` | ✅ Yes | Full support |
| **poetry install** | `poetry install` | ✅ Yes | Validates dependencies |
| **pipenv** | `pipenv install pkg` | ✅ Yes | Full support |
| **conda** | `conda install pkg` | ✅ Yes | PyPI validation |
| **mamba** | `mamba install pkg` | ✅ Yes | Same as conda |
| **pdm** | `pdm add pkg` | ✅ Yes | Full support |
| **flit** | `flit install` | ✅ Yes | Project validation |
| **hatch** | `hatch env create` | ✅ Yes | Monitoring enabled |
| **pip-sync** | `pip-sync requirements.txt` | ✅ Yes | Validates all packages |
| **pip-compile** | `pip-compile` | ⚠️ Partial | Monitoring only |
| **build** | `python -m build` | ⚠️ Partial | Build monitoring |
| **twine** | `twine upload` | ❌ No | Upload only |

## Detailed Coverage

### 1. Standard pip Commands

#### Basic Installation
```bash
pip install requests
pip3 install numpy
```
**Intercepts:** ✅ Full validation  
**Flow:** Shell wrapper → Handler → Validate → Install

#### With Version
```bash
pip install requests==2.31.0
pip install 'numpy>=1.24.0'
```
**Intercepts:** ✅ Full validation  
**Version:** Extracted and validated

#### Requirements File
```bash
pip install -r requirements.txt
```
**Intercepts:** ✅ Full validation  
**Flow:** Parses file → Validates each package → Installs

#### Multiple Packages
```bash
pip install requests numpy pandas
```
**Intercepts:** ✅ Full validation  
**Flow:** Each package validated separately

#### Upgrade
```bash
pip install --upgrade requests
```
**Intercepts:** ✅ Full validation  
**Note:** Validates new version before upgrade

#### User Install
```bash
pip install --user requests
```
**Intercepts:** ✅ Full validation  
**Note:** Still validates before local install

---

### 2. Python Module Invocation

#### Standard Form
```bash
python -m pip install requests
python3 -m pip install numpy
```
**Intercepts:** ✅ Full validation  
**Flow:** Python wrapper detects `-m pip` → Routes to pip wrapper

#### With Arguments
```bash
python3 -m pip install -r requirements.txt
python -m pip install --upgrade package
```
**Intercepts:** ✅ Full validation  
**Handles:** All pip arguments preserved

---

### 3. easy_install (Legacy)

#### Basic Usage
```bash
easy_install requests
```
**Intercepts:** ✅ Full validation  
**Note:** Legacy method, validation recommended

#### With Options
```bash
easy_install --upgrade package
```
**Intercepts:** ✅ Full validation  
**Flow:** Extracts package names → Validates → Installs

---

### 4. setup.py Direct Installation

#### Local Package
```bash
python setup.py install
```
**Intercepts:** ✅ Validation attempted  
**Flow:**
1. Extracts package name from setup.py
2. Attempts validation against PyPI
3. Proceeds with installation

#### Development Mode
```bash
python setup.py develop
```
**Intercepts:** ✅ Monitoring  
**Note:** Validates if package name extractable

#### Build and Install
```bash
python setup.py build install
```
**Intercepts:** ✅ Validation attempted

---

### 5. Poetry

#### Add Package
```bash
poetry add requests
poetry add numpy==1.24.0
```
**Intercepts:** ✅ Full validation  
**Flow:** Validates before poetry modifies pyproject.toml

#### Add Multiple
```bash
poetry add requests numpy pandas
```
**Intercepts:** ✅ Full validation  
**Each package validated separately

#### Install from Lock
```bash
poetry install
```
**Intercepts:** ✅ Validates dependencies  
**Flow:** Reads pyproject.toml → Validates packages

#### Add Dev Dependency
```bash
poetry add --dev pytest
```
**Intercepts:** ✅ Full validation

---

### 6. Pipenv

#### Install Package
```bash
pipenv install requests
```
**Intercepts:** ✅ Full validation  
**Flow:** Validates → Updates Pipfile → Installs

#### Install from Pipfile
```bash
pipenv install
```
**Intercepts:** ✅ Validates all dependencies  
**Flow:** Reads Pipfile → Validates each → Installs

#### Dev Dependencies
```bash
pipenv install --dev pytest
```
**Intercepts:** ✅ Full validation

---

### 7. Conda / Mamba

#### Conda Install
```bash
conda install numpy
conda install -c conda-forge package
```
**Intercepts:** ✅ PyPI validation  
**Note:** Validates against PyPI equivalent

#### Mamba Install
```bash
mamba install requests
```
**Intercepts:** ✅ PyPI validation  
**Flow:** Same as conda, uses PyPI for validation

#### Environment Install
```bash
conda install -n myenv package
```
**Intercepts:** ✅ Full validation  
**Preserves:** Environment specifications

---

### 8. PDM (Python Development Master)

#### Add Package
```bash
pdm add requests
pdm add "numpy>=1.24"
```
**Intercepts:** ✅ Full validation  
**Flow:** Validates → Updates pyproject.toml → Installs

#### Install from Lock
```bash
pdm install
```
**Intercepts:** ✅ Validates dependencies

---

### 9. Flit

#### Install Current Project
```bash
flit install
```
**Intercepts:** ✅ Project validation  
**Flow:**
1. Reads package name from pyproject.toml
2. Validates package if on PyPI
3. Proceeds with installation

#### Install with Symlink
```bash
flit install -s
```
**Intercepts:** ✅ Validation attempted

---

### 10. Hatch

#### Environment Creation
```bash
hatch env create
```
**Intercepts:** ✅ Monitoring  
**Note:** Monitors dependency installation

---

### 11. pip-tools

#### pip-sync
```bash
pip-sync requirements.txt
```
**Intercepts:** ✅ Full validation  
**Flow:** Parses requirements → Validates each → Syncs

#### pip-compile
```bash
pip-compile requirements.in
```
**Intercepts:** ⚠️ Monitoring  
**Note:** Compilation only, no installation

---

## Special Cases

### Editable Installs

```bash
pip install -e .
pip install -e git+https://github.com/user/repo.git
```
**Intercepts:** ⚠️ Skipped  
**Reason:** Direct source, validation not applicable  
**Message:** "Skipping (direct source installation)"

### URL Installs

```bash
pip install https://files.pythonhosted.org/package.whl
pip install git+https://github.com/user/repo.git
```
**Intercepts:** ⚠️ Skipped  
**Reason:** Direct URLs bypass PyPI  
**Message:** "Skipping (direct source installation)"

### Local Wheel Files

```bash
pip install ./package-1.0-py3-none-any.whl
```
**Intercepts:** ⚠️ Skipped  
**Reason:** Pre-built, no PyPI validation possible

### Constraints Files

```bash
pip install package -c constraints.txt
```
**Intercepts:** ✅ Package validated  
**Constraints:** Applied after validation

---

## Environment-Specific Installation

### Virtual Environments

#### venv
```bash
python -m venv myenv
source myenv/bin/activate
pip install requests  # ✅ Validated and installed to venv
```

#### virtualenv
```bash
virtualenv myenv
source myenv/bin/activate
pip install requests  # ✅ Validated and installed
```

### Conda Environments
```bash
conda create -n myenv python=3.11
conda activate myenv
pip install requests  # ✅ Validated with snakepit
```

### Poetry Environments
```bash
poetry shell
pip install requests  # ✅ Validated in poetry venv
```

---

## Bypassing Validation

### Per-Command Bypass

```bash
# Environment variable
SNAKEPIT_BYPASS=1 pip install trusted-package
SNAKEPIT_BYPASS=1 poetry add known-good-package
SNAKEPIT_BYPASS=1 conda install verified-package

# Helper function
pip-direct install trusted-package
```

### Temporary Disable

```bash
snakepit-disable
pip install package1
poetry add package2
conda install package3
snakepit-enable
```

### Permanent Exclusions

Add to `~/.bashrc`:
```bash
# Always bypass for specific tools
export SNAKEPIT_BYPASS_POETRY=1
export SNAKEPIT_BYPASS_CONDA=1
```

---

## Installation Method Detection

Snakepit automatically detects which tools are installed:

```bash
snakepit-status
```

Output shows:
```
🐍 Snakepit Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Wrapped Tools:
    ✅ pip / pip3
    ✅ python / python3
    ✅ poetry
    ✅ pipenv
    ⬜ pdm (not installed)
    ⬜ flit (not installed)
    ⬜ conda (not installed)
```

---

## Advanced Usage Examples

### Mixed Tool Workflow

```bash
# All validated through snakepit
poetry add django
pip install pillow
conda install numpy
pipenv install requests
```

### Requirements Files with Different Tools

```bash
# pip requirements
pip install -r requirements.txt  # ✅ All validated

# Poetry dependencies
poetry install  # ✅ All validated

# Conda environment
conda env create -f environment.yml  # ✅ Packages validated
```

### CI/CD Pipeline

```yaml
# .github/workflows/test.yml
- name: Install dependencies
  run: |
    source ~/snakepit/snakepit-shell-integration.sh
    pip install -r requirements.txt  # All validated
    poetry install  # All validated
```

---

## Troubleshooting

### Tool Not Being Intercepted

**Check if wrapper exists:**
```bash
type pip  # Should show it's a function
type poetry  # Should show it's a function
```

**Reload integration:**
```bash
source ~/.bashrc
# or
source ~/snakepit/snakepit-shell-integration.sh
```

### Validation Failing for Tool-Specific Package

**Bypass for that package:**
```bash
SNAKEPIT_BYPASS=1 poetry add problematic-package
```

### Performance Issues

**Disable for bulk operations:**
```bash
snakepit-disable
poetry install  # Fast, no validation
pip install -r large-requirements.txt
snakepit-enable
```

---

## Testing Coverage

### Test All Methods

```bash
# Run comprehensive test
snakepit-test

# Manual testing
echo "Testing pip..." && pip install requests
echo "Testing poetry..." && poetry add requests
echo "Testing pipenv..." && pipenv install requests
echo "Testing conda..." && conda install requests
```

---

## Implementation Details

### Wrapper Hierarchy

```
User Command
    ↓
Shell Function Wrapper (bash)
    ↓
Universal Wrapper (_snakepit_*_wrapper)
    ↓
Snakepit CLI (snakepit_cli.py)
    ↓
Snakepit Handler (snakepit_handler.py)
    ↓
Sandbox Validation
    ↓
Backend Installation (Rust binary or pip)
```

### Detection Logic

```bash
# Check if tool available
if command -v poetry &> /dev/null; then
    # Wrap it
    poetry() { _snakepit_poetry_wrapper "$@"; }
fi
```

### Package Name Extraction

```bash
# From: requests==2.31.0
# Extract: requests

# From: numpy>=1.24.0
# Extract: numpy

# From: package[extras]
# Extract: package
```

---

## Configuration

### Per-Tool Configuration

In `snakepit.toml`:

```toml
[tools]
validate_pip = true
validate_poetry = true
validate_pipenv = true
validate_conda = true
validate_pdm = true

[tools.pip]
handle_requirements_files = true
handle_editable = false

[tools.poetry]
validate_before_add = true

[tools.conda]
use_pypi_validation = true
```

---

## Summary

**✅ Fully Covered (11 methods):**
- pip, pip3
- python -m pip, python3 -m pip
- requirements.txt
- poetry
- pipenv
- conda / mamba
- pdm
- flit
- pip-sync
- easy_install
- setup.py

**⚠️ Partially Covered (3 methods):**
- pip-compile (monitoring)
- build (monitoring)
- Editable installs (skipped)

**❌ Not Covered (1 method):**
- twine (upload tool, not installation)

**Total Coverage:** **93% of installation methods**

---

## Getting Help

```bash
# Check what's covered
snakepit-status

# Test coverage
snakepit-test

# Verbose mode for any tool
SNAKEPIT_VERBOSE=1 <tool> install <package>

# Bypass if needed
SNAKEPIT_BYPASS=1 <tool> install <package>
```

For issues or questions, see `ROUTING_README.md` or `TROUBLESHOOTING.md`.
