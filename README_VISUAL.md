# 🐍 Visual Install for Python (VIP)

**Transform boring pip installations into an engaging snake game!**

Every time you run `pip install`, watch an automated snake game visualize your package downloads and builds in real-time.

## ✨ What Is This?

VIP wraps `pip` to provide a beautiful, automated pygame GUI that shows package installation progress as a snake game:

- **Packages appear as colored pellets** (white=queued, cyan=downloading, orange=building, green=ready)
- **Snake automatically navigates** to collect packages
- **Progress bars** show download/build progress in real-time
- **Real-time status messages** show what's happening
- **Completely automatic** - no manual control needed
- **Falls back gracefully** if GUI isn't available

## 🚀 Quick Start

### One-Line Install

```bash
cd ~/snakepit
./install-visual.sh
source ~/.bashrc
```

### Test It

```bash
# Demo mode (simulated installations)
python3 snake_monitor.py

# Real installation
pip install colorama
```

## 📦 What You Get

### Files Created

- **vip** - pip wrapper that launches the GUI
- **snake_monitor.py** - Automated snake game monitor
- **snake_gui.py** - Manual playable version
- **vip-integration.sh** - Shell aliases and functions
- **visual_installer.rs** - Rust integration for snakepit daemon
- **install-visual.sh** - Quick setup script

### Shell Integration

After installation, these commands are available:

```bash
pip install package       # Automatically uses visual mode
vip install package       # Direct VIP command
pip-classic install pkg   # Original pip (no GUI)

vip-status               # Check configuration
vip-gui-on              # Enable visual mode
vip-gui-off             # Disable visual mode
```

## 🎮 Features

### Visual Elements

| Element | Meaning |
|---------|---------|
| 🟢 Bright green snake | Your installer |
| ⚪ White pellet | Queued package |
| 🔵 Cyan pellet | Downloading |
| 🟠 Orange pellet | Building |
| 🟢 Green pellet | Ready to install |
| 🔴 Red pellet | Failed package |
| ▬ Progress bar | Download/build progress |

### AI Behavior

- Automatically pathfinds to nearest package
- Prioritizes ready packages over downloading ones
- Avoids walls and obstacles
- Grows as it collects packages
- Shrinks if it hits failed packages

### Status Display

Bottom bar shows:
- Packages completed/total
- Failed package count
- Snake length and score
- Real-time event messages
- Total installation time

## 🛠️ Usage Examples

### Basic Installation

```bash
pip install numpy
```

### Multiple Packages

```bash
pip install pandas matplotlib scikit-learn
```

### From requirements.txt

```bash
pip install -r requirements.txt
```

All packages appear in the game!

### With Version Specification

```bash
pip install flask==2.0.0
```

### In Virtual Environments

```bash
python3 -m venv myenv
source myenv/bin/activate
pip install django  # Still shows visual mode
```

## 🔧 Configuration

### Disable GUI Temporarily

```bash
vip-gui-off
pip install something  # Classic mode
vip-gui-on
```

### Disable for Scripts

```bash
# In your script
export VIP_NO_GUI=1
pip install automated-package
```

### Permanent Disable

Edit `~/.bashrc` and comment out:
```bash
# source ~/snakepit/vip-integration.sh
```

## 🎯 Integration with Snakepit

The visual installer integrates with the snakepit Rust tool:

```bash
# When compiled with visual_installer module
snakepit install numpy     # Uses VIP

# Daemon mode
snakepit daemon start      # Can use visual feedback
```

## 🧩 Architecture

```
User runs: pip install package
           ↓
     vip wrapper (Python)
           ↓
    ├─→ Parses pip output
    │   Generates events
    │   (package_queued, downloading, etc.)
    │
    ├─→ snake_monitor.py (pygame)
    │   Displays animated snake game
    │   Shows progress in real-time
    │
    └─→ Real pip subprocess
        Downloads & installs package
        Outputs to terminal
```

## 📋 Requirements

- Python 3.6+
- pip
- pygame (`pip install pygame`)
- X11/Wayland display (DISPLAY variable set)
- Linux/Unix-like system

## 🐛 Troubleshooting

### GUI doesn't appear

```bash
# Check pygame
python3 -c "import pygame; print('OK')"

# Check display
echo $DISPLAY

# Check VIP status
vip-status
```

### Want to use classic pip

```bash
pip-classic install package
# or
python3 -m pip install package
```

### Temporary GUI issues

```bash
export VIP_NO_GUI=1  # Disable for session
```

## 🎨 Customization

Edit `snake_monitor.py` to customize:

```python
WINDOW_WIDTH = 1000      # Window size
WINDOW_HEIGHT = 700
GRID_SIZE = 20          # Cell size
FPS = 15                # Animation speed

# Colors
BLACK = (0, 0, 0)
BRIGHT_GREEN = (0, 255, 0)
# ... etc
```

## 📚 Documentation

- **VISUAL_SETUP.md** - Detailed setup guide
- **GUI_README.md** - Manual game mode
- **README.md** - Main snakepit docs

## 🤝 How It Works

1. **Aliasing**: `pip` is aliased to `vip`
2. **Interception**: `vip` wraps the real pip command
3. **Parsing**: Regex parses pip output for events
4. **Events**: Queue sends events to pygame thread
5. **Visualization**: Snake game displays progress
6. **Completion**: Window auto-closes after 5 seconds

## ⚡ Performance

- Minimal overhead (parsing is lightweight)
- GUI runs in separate thread
- Falls back to classic mode if slow
- No impact on actual installation speed

## 🎉 Why?

Because package installation shouldn't be boring! Watch your dependencies come to life as an entertaining snake game while still getting all the terminal output you need.

## 🐍 Enjoy!

Make pip fun again. Every installation is now a mini-game!

```bash
pip install everything
```

Watch the snake grow! 🚀
