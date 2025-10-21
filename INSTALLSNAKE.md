# InstallSnake 🐍

**InstallSnake** is a retro terminal-based Snake game that visualizes Python package installations as gameplay events. Every pip install becomes an interactive adventure!

## Overview

Instead of watching boring progress bars, InstallSnake animates your pip installations as a classic Snake game:

- **Pellets (●)** = Python packages to install
- **Snake head (►)** = Your installer's progress
- **Snake body (█)** = Completed installations accumulate as growth
- **Crashes (red flash)** = Build failures or errors
- **Level complete** = All packages installed successfully

## Features

✨ **Multi-Theme Support**
- `retro` - Classic green-on-black CRT aesthetic
- `amber` - Vintage amber terminal
- `matrix` - Neon green digital rain style
- `minimal` - Monochrome ASCII-only
- `error` - Red glitch aesthetic for debugging

⚙️ **Configurable**
- Adjustable FPS (frames per second)
- Custom board width
- Themeable colors and characters

🎮 **Event-Driven**
- Parses real pip output into game events
- Deterministic animation tied to install progress
- Crash animations for build failures

## Usage

### Play Demo Game

```bash
snakepit game --theme retro --fps 12 --width 60
```

### Options

```
  -t, --theme <THEME>  [default: retro]
                       Options: retro, amber, matrix, minimal, error
  
  -f, --fps <FPS>      Frames per second [default: 12]
  
  -w, --width <WIDTH>  Board width in characters [default: 60]
```

### Examples

**Amber theme, high FPS:**
```bash
snakepit game --theme amber --fps 20
```

**Minimal monochrome (for dumb terminals):**
```bash
snakepit game --theme minimal --width 40
```

**Matrix style:**
```bash
snakepit game --theme matrix
```

## How It Works

### Game State Machine

```
Initializing → Playing → Error (if crash) → Playing → Won
```

### Event Types

| Event | Trigger | Effect |
|-------|---------|--------|
| `PackageQueued` | pip discovers a package | Pellet spawns on board |
| `DownloadProgress` | Downloading package wheel | Snake advances toward pellet |
| `BuildStarted` | Building wheel from source | Pellet state changes (visual shift) |
| `BuildProgress` | Wheel build progresses | Pellet progress bar animates |
| `InstallComplete` | Package installed | Snake eats pellet, grows 3 segments |
| `BuildFailed` | Wheel build fails | Snake crashes (red flash), respawns |
| `AllDone` | All packages installed | Board fills, "Level Complete" |

### Terminal Rendering

- **Efficient diff-rendering**: Only changed cells are redrawn
- **Fixed timestep**: Smooth 12 FPS default animation
- **Fallback support**: Works on dumb terminals (no ANSI codes needed)

## Architecture

```
snakepit/src/
├── installsnake.rs      # Core game engine, board, snake physics
├── game_runner.rs       # High-level runner, pip output parsing
└── main.rs              # CLI integration
```

### Key Types

- `InstallSnake` - Main game engine
- `SnakeConfig` - Configuration (theme, FPS, size)
- `InstallEvent` - Parsed pip subprocess events
- `GameRunner` - Manages game loop and subprocess integration

## Game Physics

**Snake Movement**
- Snake head always moves forward in current direction
- Body segments follow head like a queue
- Wraps at board boundaries (no collision with walls in demo)

**Pellet Spawning**
- Random free position on board
- One pellet per queued package
- Pellets marked with ✗ if build fails

**Growth & Scoring**
- Eating pellet = +3 body segments
- Crash animation = temporary red state, respawn shorter

## Pip Event Parsing

The `game_runner` module parses typical pip output:

```
"Collecting numpy==1.21.0"           → PackageQueued("numpy==1.21.0")
"Downloading numpy-1.21.0-*.whl (14MB)" → DownloadStarted { ... }
"Building wheel for pandas"           → BuildStarted("pandas")
"Successfully built scikit-learn"     → InstallComplete("scikit-learn")
"ERROR: Failed building wheel"        → Error(...) / BuildFailed
```

## Configuration File (Future)

```toml
# ~/.config/snakepit/game.toml
[display]
theme = "retro"
width = 60
height = 15
fps = 12
sound = false

[behavior]
on_error = "crash"      # crash | pause | skip
on_success = "grow"     # grow | pulse | sparkle
show_pellet_labels = true
```

## Testing

Run demo with mock events:

```bash
cargo build --release
./target/release/snakepit game --theme matrix --fps 15 --width 50
```

Run tests (includes pip output parsing):

```bash
cargo test game_runner::tests
```

## Performance

- **Binary size**: ~5MB (stripped)
- **Memory**: <10MB at runtime
- **CPU**: <2% during animation (fixed timestep)
- **Terminal writes**: O(changed_cells) per frame (diff-based)

## Roadmap

- [ ] Real pip subprocess integration (hook into `snakepit install`)
- [ ] Keyboard controls (player-directed snake instead of auto-move)
- [ ] High score persistence
- [ ] Multiplayer mode (race multiple installs)
- [ ] Custom level editor
- [ ] Sound effects (bell on crash, fanfare on win)
- [ ] Accessibility mode (high contrast, larger text)

## Design Inspirations

- Classic Snake (Nokia 3310)
- Terminal Emulator Retro Games
- Progress bars as mini-games (Patchwork, etc.)
- Hacker aesthetic (Matrix, WarGames)

## License

MIT (same as snakepit)
