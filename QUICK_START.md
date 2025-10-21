# 🚀 VIP Quick Start Guide

## Installation (30 seconds)

```bash
cd ~/snakepit
./install-visual.sh
source ~/.bashrc
```

Done! Now every `pip install` shows a snake game! 🐍

## Basic Usage

```bash
# Install with visual game
pip install numpy

# Multiple packages
pip install pandas matplotlib requests

# From requirements file
pip install -r requirements.txt
```

## Control Commands

| Command | Action |
|---------|--------|
| `vip install pkg` | Install with visual mode |
| `pip-classic install pkg` | Use classic pip |
| `vip-status` | Check configuration |
| `vip-gui-off` | Disable visualization |
| `vip-gui-on` | Enable visualization |

## Visual Legend

| Symbol | Meaning |
|--------|---------|
| ⚪ White | Package queued |
| 🔵 Cyan | Downloading |
| 🟠 Orange | Building |
| 🟢 Green | Ready to install |
| 🔴 Red | Failed |
| 🐍 Green snake | Your installer (AI controlled) |

## Test It

```bash
# Demo mode (simulated)
cd ~/snakepit
python3 snake_monitor.py

# Real install
pip install colorama
```

## Disable If Needed

```bash
# Temporary
export VIP_NO_GUI=1

# Permanent
vip-gui-off
```

## Files

- **vip** - Wrapper script
- **snake_monitor.py** - GUI visualizer
- **snake_gui.py** - Playable version
- **vip-integration.sh** - Shell integration

## Help

```bash
vip-status              # Check setup
which vip               # Verify location
python3 -c "import pygame"  # Test pygame
```

## Features

✅ Fully automated - no manual control
✅ **Smart AI pathfinding** - never gets stuck!
✅ Real-time progress bars
✅ Color-coded package states  
✅ **Continuous game session** - collects all packages
✅ Status message log with collection events
✅ Victory screen when complete
✅ Falls back to classic mode if needed
✅ Works with virtualenvs
✅ Zero performance impact

## Documentation

- **README_VISUAL.md** - Full features
- **VISUAL_SETUP.md** - Detailed setup
- **AUTOMATION_COMPLETE.md** - Architecture

---

**That's it! Install packages and watch the snake collect them! 🎮**
