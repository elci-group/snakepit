# Visual Install System Setup

Automated GUI visualization for all pip/Python package installations using InstallSnake.

## Quick Setup

### 1. Install pygame

```bash
pip install pygame
```

### 2. Enable Shell Integration

Add this to your `~/.bashrc`:

```bash
# Visual Install for Python (VIP)
source ~/snakepit/vip-integration.sh
```

Then reload your shell:

```bash
source ~/.bashrc
```

## Usage

### Automatic Visual Mode

Once integrated, all `pip install` commands will automatically launch the snake game visualization:

```bash
# This now shows the snake game!
pip install numpy pandas matplotlib

# Individual packages
pip install requests

# With version specification
pip install flask==2.0.0
```

### Direct VIP Command

You can also use `vip` directly:

```bash
vip install tensorflow
vip install scikit-learn numpy scipy
```

### Control Functions

```bash
# Check status
vip-status

# Temporarily disable visual mode
vip-gui-off
pip install something  # Will use classic pip

# Re-enable visual mode
vip-gui-on

# Use classic pip without affecting aliases
pip-classic install package
```

## Environment Variables

```bash
# Disable GUI for current session
export VIP_NO_GUI=1

# Re-enable
unset VIP_NO_GUI
```

## Test the System

### Test with Demo Mode

Run the monitor in demo mode to see it in action:

```bash
cd ~/snakepit
python3 snake_monitor.py
```

This will simulate package installations.

### Test with Real Installation

Try installing a small package:

```bash
pip install colorama
```

You should see:
1. The snake game window opens
2. Package appears as a colored pellet
3. Snake automatically navigates to collect packages
4. Progress bars show download/build status
5. Window closes after completion

## Features

### Visual Indicators

- **White pellets**: Queued packages
- **Cyan pellets**: Downloading
- **Orange pellets**: Building
- **Green pellets**: Ready to install
- **Red pellets**: Failed installations
- **Progress bars**: Download/build progress
- **Status messages**: Real-time event log

### AI Snake

The snake is fully automated:
- Automatically navigates to packages
- Prioritizes ready packages
- Avoids obstacles and itself
- Grows as it collects packages

### Status Bar

Shows:
- Packages completed/total
- Failed packages
- Snake score and length
- Real-time status messages
- Completion time

## Integration with Snakepit

The visual installer integrates with the main snakepit tool:

```bash
# Visual mode for snakepit commands (when compiled)
snakepit install numpy

# The daemon can also use visual mode
snakepit daemon start
```

## Troubleshooting

### No GUI appears

Check pygame installation:
```bash
python3 -c "import pygame; print('pygame OK')"
```

Check DISPLAY variable:
```bash
echo $DISPLAY
# Should show :0 or similar
```

### Visual mode not activating

Check VIP status:
```bash
vip-status
```

Verify integration loaded:
```bash
which vip
# Should show /home/adminx/snakepit/vip
```

### Want classic pip back

```bash
# Temporary
vip-gui-off

# Or use pip-classic alias
pip-classic install package

# Or bypass completely
python3 -m pip install package
```

## Architecture

```
┌─────────────────┐
│  pip command    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌──────────────────┐
│   vip wrapper   │────▶│  snake_monitor   │
│  (intercepts)   │     │   (pygame GUI)   │
└────────┬────────┘     └──────────────────┘
         │
         ▼
┌─────────────────┐
│  real pip       │
│  subprocess     │
└─────────────────┘
```

## Files

- `vip` - Main pip wrapper script
- `snake_monitor.py` - Automated snake game monitor
- `vip-integration.sh` - Shell integration
- `snake_gui.py` - Manual play mode
- `visual_installer.rs` - Rust integration for snakepit

## Benefits

1. **Engaging**: Makes package installation fun to watch
2. **Informative**: Visual progress tracking
3. **Non-intrusive**: Falls back to classic mode if GUI unavailable
4. **Automatic**: No manual control needed
5. **Compatible**: Works with standard pip commands

## Advanced Usage

### Use with Virtual Environments

Works seamlessly:
```bash
python3 -m venv myenv
source myenv/bin/activate
pip install django  # Visual mode works in venv
```

### Use with requirements.txt

```bash
pip install -r requirements.txt
```

All packages will appear in the game!

### Disable for Scripts

```bash
# In your script
export VIP_NO_GUI=1
pip install package
```

Or use `pip-classic` alias.

## Customization

Edit `snake_monitor.py` to customize:
- Window size (`WINDOW_WIDTH`, `WINDOW_HEIGHT`)
- Grid size (`GRID_SIZE`)
- Frame rate (`FPS`)
- Colors
- Obstacle density

Enjoy visual package installations! 🐍
