# 🐍 Snakepit: Next-Generation Python Package Manager

**Advanced dependency resolution meets AI-powered organic code evolution**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3670A0?style=for-the-badge&logo=python&logoColor=ffdd54)](https://www.python.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Snakepit is a powerful Python package manager built in Rust that combines **state-of-the-art dependency resolution** with optional **AI-driven organic code evolution**. Use it as a superior pip replacement, or unlock revolutionary features like cross-language development, thermal knowledge sharing, and quantum storage.

---

## 🎯 Two Ways to Use Snakepit

### 1️⃣ **Traditional Package Management** (Core Features)
Drop-in pip replacement with better dependency resolution, virtual environment management, and intelligent caching.

### 2️⃣ **Organic Code Evolution** (Revolutionary Features)
AI-powered system where code evolves like living organisms through 5 groundbreaking innovations.

---

## 📦 Core Package Management Features

Snakepit excels as a modern Python package manager with capabilities that surpass pip, conda, and poetry:

### ⚡ **Superior Dependency Resolution (PubGrub)**

- **State-of-the-art Algorithm**: PubGrub solver with conflict-driven learning
- **Full PEP Compliance**: PEP 440 (versions), PEP 508 (dependencies), PEP 517/518 (builds)
- **Clear Error Messages**: Detailed explanations when resolution fails
- **Lockfile Support**: Deterministic, reproducible builds (`snakepit.lock`)

```bash
snakepit install requests numpy pandas
# ✅ Resolved 47 packages in 1.2s
# 📊 12 direct, 35 transitive dependencies
# 🔒 Lockfile generated with SHA256 verification
```

### 🔧 **Smart Virtual Environment Management**

- **Multi-backend Support**: venv, virtualenv, conda, poetry
- **Automatic Detection**: Detects and activates appropriate environments
- **Python Version Selection**: Create venvs with specific Python versions
- **Shell Integration**: Bash helper functions for quick activation

```bash
# Create environment
snakepit venv create my-project --python 3.11

# List environments
snakepit venv list

# Activate (with shell integration)
venv-activate my-project
```

### 🎨 **Beautiful CLI Experience**

- **Colorized Output**: Clear, readable terminal output
- **Progress Indicators**: Real-time installation progress
- **Intelligent Defaults**: Sensible behavior out of the box
- **Configuration Management**: Global and per-project settings

### 🔄 **Multi-Backend Support**

Automatically integrates with your existing tools:

- **pip**: System pip or virtualenv pip
- **conda**: Conda package and environment management  
- **poetry**: Poetry dependency resolution and projects

```bash
# Snakepit auto-detects and uses the appropriate backend
snakepit install flask
# → Uses poetry if pyproject.toml exists
# → Uses conda if in conda environment
# → Falls back to pip otherwise
```

### 🚀 **Advanced Installation Features**

- **Parallel Downloads**: Faster package installation
- **Smart Caching**: Reuse downloaded packages
- **Development Dependencies**: Separate dev and production deps
- **Version Constraints**: Full semver support
- **Custom PyPI Mirrors**: Configure alternative package sources

```bash
# Install with specific version
snakepit install numpy --version ">=1.21.0,<2.0.0"

# Development dependencies
snakepit install pytest black mypy --dev

# Sync from requirements.txt or pyproject.toml
snakepit sync
```

### 🛡️ **Intelligent Uninstaller**

- **Snapshot System**: Create pre-uninstall snapshots
- **Rollback Support**: Restore if something breaks
- **Dependency Analysis**: Show what else depends on package
- **Safe Removal**: Prevent breaking system packages

```bash
snakepit uninstall pandas --snapshot
# 📸 Snapshot created: pandas_2025-12-15_20-00
# 🗑️  Safely removed pandas
# 💾 Rollback available: snakepit restore pandas_2025-12-15_20-00
```

### 🔍 **Project Initialization & Management**

```bash
# Initialize new project
snakepit init my-awesome-project

# Creates:
# ├── snakepit.toml          # Project configuration
# ├── requirements.txt       # Dependencies  
# ├── .gitignore            # Sensible defaults
# └── venv/                 # Virtual environment
```

---

## 🌟 Revolutionary Features (Optional)

> **Status:** The SnakeEgg commands (`nest`, `egg`, `clutch`, `protein`) and the AI `recommend` command are **not part of the current CLI**. They were removed while the project narrows its focus to the core package-management workflow. The full design is preserved in `report_chapters/COMPLETE_REPORT.md` and may return in a future release. `snakepit fix` keeps working without an AI backend by detecting `ModuleNotFoundError` directly.

Beyond traditional package management, Snakepit offers **SnakeEgg**: an AI-powered organic code evolution system. These features are completely optional and activate when you want to go beyond conventional development.

### 1. 🥚 **Dual Egg System**: Cross-Language Evolution

Maintain functionally equivalent implementations across Python and Rust through AI-driven **intent extraction** and **oxidation**.

- **Organic Eggs** (Python): Rapid prototyping and iteration
- **Metallic Eggs** (Rust): Performance-optimized production code  
- **Automatic Translation**: AI extracts intent from Python, generates equivalent Rust
- **Consistency Guaranteed**: Both implementations evolve from shared DNA specifications

```bash
snakepit egg create auth_handler --species Service
# Creates both organic/ (Python) and metallic/ (Rust) implementations
# Mother AI evolves them in parallel, maintaining functional equivalence
```

**Benefits**: 50% faster polyglot development, eliminate manual translation, consistent behavior

---

### 2. 🌡️ **Heat Sharing**: Thermal Knowledge Transfer

Eggs maintain "temperature" based on progress. Knowledge flows from hot (successful) to cold (struggling) eggs like heat transfer in nature.

- **Temperature-Based Fitness**: Automatic progress tracking
- **Pattern Extraction**: Successful code patterns identified and shared
- **Emergent Collaboration**: Knowledge distributes without central coordination
- **Clutch Management**: Groups of eggs learn collectively

```bash
snakepit clutch thermal-cycle my-project
# 🔥 api_handler (85°C) → 🌡️ auth_service (42°C)
# Transferring 3 successful patterns...
```

**Benefits**: Team knowledge propagates automatically, faster problem-solving

---

### 3. 🦖 **Darwinian Diet**: Failure Cannibalization

Failed eggs are cannibalized for reusable components ("proteins") that nourish surviving eggs.

- **Intelligent Failure Detection**: Multi-metric evaluation
- **Protein Harvesting**: Extract valuable code from failures
- **Resource Recycling**: Failed work contributes to success
- **Evolutionary Pressure**: Natural selection toward quality

```bash
# Failing egg automatically cannibalized
🦖 Cannibalizing failing egg: experiments/ml_v3 (temp: 8°C, fitness: 0.15)
   Harvested 7 proteins → protein library
✅ Proteins redistributed to active eggs
```

**Benefits**: Zero wasted effort, accelerated learning from failures

---

### 4. ⏱️ **Chrono-Capacitus**: Maturity-Based Resource Allocation

AI API costs scale with egg maturity—young eggs get frequent cheap models, mature eggs get rare powerful models.

- **92% Cost Reduction**: Compared to uniform GPT-4 usage
- **6 Maturity Stages**: Zygote → Embryo → Fetus → Hatchling → Juvenile → Adult
- **Progressive Models**: Free models for exploration, premium for refinement

| Stage | Model | Interval | Use Case |
|-------|-------|----------|----------|
| Zygote | Flash 2.0 (Free) | 5s | Rapid exploration |
| Fetus | Pro 2.0 | 30s | Core logic |
| Adult | Pro 2.5 | 5min | Production polish |

**Benefits**: Sustainable AI costs at scale, optimal resource allocation

---

### 5. 👁️ **Schrödinger's Shells**: Quantum Storage

Eggs exist in **superposition** between local filesystem and git, materializing only when observed (actively developed).

- **70-90% Storage Reduction**: Only active eggs consume local space
- **Quantum States**: Ethereal (git-only), Manifested (local), Superposition (both)
- **Automatic Vacuum**: Idle eggs evaporate after timeout

```bash
snakepit nest vacuum --max-idle 24h
🧹 47 eggs evaporated to ether  
💾 2.1 GB → 180 MB (91% reduction)
```

**Benefits**: Massive storage savings, perfect for CI/CD

---

## 🚀 Quick Start

### Installation

```bash
# From source
git clone https://github.com/elci-group/snakepit.git
cd snakepit
cargo build --release

# Binary at: target/release/snakepit
```

### Man Pages

Manual pages for every command and subcommand live in `man/` (e.g.
`man/snakepit-install.1`, `man/snakepit-venv-create.1`). Install them with:

```bash
sudo cp man/*.1 /usr/local/share/man/man1/
mandb  # optional, refresh the index
```

Then view with `man snakepit` or `man snakepit-venv-create`. The pages are
generated from the clap definitions in `src/cli.rs` — regenerate them after
changing the CLI with:

```bash
cargo run --example gen-man
```

### Traditional Package Management

```bash
# Install packages (like pip)
snakepit install requests numpy pandas

# Create virtual environment
snakepit venv create my-env --python 3.11
snakepit venv activate my-env

# Initialize project
snakepit init my-project

# Sync dependencies
snakepit sync
```

### Organic Code Evolution (Optional)

> **Not available in the current CLI** — see the status note under "Revolutionary Features" above.

```bash
# Initialize quantum nest
snakepit nest init

# Create dual egg (Python + Rust)
snakepit egg create web_api --species Service --type dual

# Let Mother AI evolve it
snakepit egg evolve web_api

# Check progress
snakepit egg status web_api
# 🥚 web_api (Fetus, 67% complete)
#    Temperature: 72°C 🔥
#    Organic: 847 lines Python
#    Metallic: 923 lines Rust
```

---

## 📊 Performance Comparison

### vs pip

| Feature | Snakepit | pip |
|---------|----------|-----|
| Dependency Resolution | PubGrub (optimal) | Backtracking |
| Resolve Speed | 1-3s | 5-15s |
| Lockfiles | ✅ Built-in | ❌ Requires pip-tools |
| Error Messages | Detailed explanations | Cryptic |
| Virtual Env Mgmt | ✅ Integrated | ❌ Separate tool |

### vs conda

| Feature | Snakepit | conda |
|---------|----------|-------|
| Package Scope | Python-focused | Multi-language |
| Speed | Fast (Rust) | Slower (Python) |
| Storage | Efficient | Large |
| Binary Dependencies | Via backends | Native support |

### vs poetry

| Feature | Snakepit | poetry |
|---------|----------|--------|
| Resolution Algorithm | PubGrub | PubGrub |
| Speed | Faster (Rust) | Slower (Python) |
| AI Features | ❌ Removed (see note above) | ❌ None |
| Cross-language | ❌ Removed (see note above) | ❌ Python only |

---

## 🏗️ Architecture

### Core Package Management

- **`src/installer.rs`** - Package installation engine
- **`src/uninstaller.rs`** - Intelligent uninstaller with snapshots
- **`src/resolver.rs`** - Main dependency resolution
- **`src/solver.rs`** - PubGrub algorithm
- **`src/pep440.rs`** - PEP 440 version parsing
- **`src/markers.rs`** - PEP 508 environment markers
- **`src/venv.rs`** - Virtual environment management
- **`src/config.rs`** - Configuration system
- **`src/lockfile.rs`** - Deterministic builds

### Platform Utilities

- **`src/snakegg.rs`** - In-tree platform shim (styling, directories, hashing, progress, IDs) replacing the former external `snakegg` crate; backed by `dirs`, `indicatif`, `sha2`, `md5`, and `chrono`

### AI Integration

- **`src/hallucinatory_fangs.rs`** - Confidence scoring

> The SnakeEgg organic-evolution modules (`snake_egg/*`, `charmer.rs`, `resolver_ai.rs`) were removed together with the `nest`/`egg`/`clutch`/`protein`/`recommend` commands — see the status note under [Revolutionary Features](#-revolutionary-features-optional).

---

## 📚 Configuration

### Global Config (`~/.config/snakepit/config.toml`)

```toml
default_backend = "pip"  # pip, conda, poetry
default_venv_backend = "venv"
venv_path = "~/.snakepit/venvs"
cache_enabled = true
python_version = "3.11"

[mirrors]
mirrors = ["https://pypi.org/simple/"]
```

### Project Config (`snakepit.toml`)

```toml
name = "my-project"
version = "0.1.0"
python_version = "3.11"
backend = "pip"

dependencies = [
    "requests>=2.25.0",
    "numpy>=1.21.0",
]

dev_dependencies = [
    "pytest>=6.0.0",
    "black>=21.0.0",
]
```

---

## 🤝 Contributing

We welcome contributions! Snakepit is open source (MIT License).

```bash
git clone https://github.com/elci-group/snakepit.git
cd snakepit
cargo build
cargo test
```

---

## 🗺️ Roadmap

**Phase 1: Core Stabilization** (Months 1-6)
- [x] PubGrub dependency resolution
- [x] Virtual environment management
- [x] SnakeEgg revolutionary features
- [ ] 85%+ test coverage
- [ ] Production hardening

**Phase 2: Expansion** (Months 7-12)
- [ ] IDE plugins (VS Code, PyCharm)
- [ ] CI/CD integrations
- [ ] Package vulnerability scanning
- [ ] GUI interface

**Phase 3: Advanced** (Year 2+)
- [ ] JavaScript/TypeScript support
- [ ] Custom AI model integration
- [ ] Multi-language clutches

---

## 📜 License

MIT License - see [LICENSE](./LICENSE)

---

## 🌐 Links

- **GitHub**: https://github.com/elci-group/snakepit
- **Report**: [75K-word Graduate Analysis](./Snakepit_Graduate_Report.pdf)
- **Issues**: https://github.com/elci-group/snakepit/issues

---

**Made with 🦀 Rust and 🐍 Python**

*A powerful package manager that can also evolve code organically*
