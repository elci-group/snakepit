# AGENTS.md

## Project Overview
Snakepit is a Rust-based CLI tool for Python dependency management, supporting multiple backends (pip, conda, poetry) and virtual environments. It includes features like dynamic dependency resolution, shell integration, daemon process monitoring, and an integrated snake game for visualization. The project combines Rust core with Python scripts and shell integrations, licensed under MIT.

## Code Organization
- **Rust Source**: Located in `src/` with main entry in `src/main.rs`. Modules include `cli.rs`, `config.rs`, `daemon.rs`, `dependency.rs`, `installer.rs`, `resolver.rs`, `venv.rs`, and a `game/` subdirectory with `ai.rs`, `board.rs`, `engine.rs`, etc.
- **Python Scripts**: Root-level files like `snake_gui.py` (pygame-based game), `dependency_resolver.py`, `smart_snakepit.py`, `test_resolver.py`.
- **Shell Scripts**: Root-level `.sh` files for installation and integration, e.g., `install-daemon.sh`, `snakepit-shell-integration.sh`.
- **Documentation**: In `docs/` with files like `README.md`, `INSTALLATION.md`, `CHANGELOG.md`.
- **Configuration**: `Cargo.toml` for Rust dependencies, `.github/workflows/ci.yml` for CI, `.gitignore`, and systemd service file `snakepit-daemon.service`.
- **Examples**: In `examples/` with `basic_usage.md` and `run_game.sh`.

## Essential Commands
- **Build**: `cargo build` or `cargo build --release`.
- **Test**: `cargo test` or `cargo test --verbose` (runs on Linux, macOS, Windows in CI).
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`.
- **Format Check**: `cargo fmt -- --check`.
- **Run**: `cargo run -- [command]`, e.g., `cargo run -- install requests`.
- **Install from Source**: `cargo install snakepit` or build from source with `cargo build --release`.
- **Game**: Integrated via `snakepit game` CLI command.
- **Python Testing**: Run scripts like `python3 test_resolver.py`.
- **Shell Integration Setup**: Source scripts like `snakepit-shell-integration.sh` in shell config.

## Naming Conventions and Style
- **Rust**: Snake_case for file/module names (e.g., `game_runner.rs`), CamelCase for structs/enums (e.g., `SnakeConfig`, `Theme`). Uses async/await with Tokio, anyhow for error handling, clap for CLI parsing. Code follows standard Rust idioms with derive macros and pattern matching.
- **Python**: Snake_case for file names (e.g., `snake_gui.py`), classes in CamelCase (e.g., `SnakeGame`). Uses dataclasses, enums, and type hints. Pygame for GUI in `snake_gui.py`.
- **Shell**: Descriptive names like `install-daemon.sh`, using bash syntax.
- **General**: Consistent use of comments, with focus on functionality. Indentation uses 4 spaces in viewed Python files, standard Rust formatting.

## Testing Approach and Patterns
- **Rust**: Unit/integration tests via `cargo test`, executed in CI on multiple platforms. CI includes clippy for linting and fmt for style checks.
- **Python**: Script-based testing, e.g., `test_resolver.py` simulates pip output and tests resolution logic by printing results (no assertion framework observed in the file).
- **CI**: GitHub Actions workflow (`ci.yml`) runs build, test, clippy, and fmt on push/PR to main branch across OSes.

## Important Gotchas
- Multi-language codebase requires managing Rust compilation alongside Python/shell scripts; ensure Cargo is used for Rust parts.
- Daemon and shell integrations involve system-level changes (e.g., systemd service, bash sourcing) – test in isolated environments.
- Game component in Rust (`game/`) and Python (`snake_gui.py`) may require pygame for visual features.
- Dependency conflicts simulated in tests; resolver handles pip output parsing.
- CI enforces no warnings (`-D warnings` in clippy) and format consistency.