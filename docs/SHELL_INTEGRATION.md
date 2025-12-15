# Shell Integration Guide

VIP (Visual Install for Python) supports **bash**, **zsh**, and **fish** shells with full integration including aliases, functions, and tab completion.

## Supported Shells

- **Bash** (4.0+)
- **Zsh** (5.0+)
- **Fish** (3.0+)

## Quick Install

The install script automatically detects your shell and configures integration:

```bash
./install-visual.sh
```

## Manual Integration

### Bash

Add to `~/.bashrc`:

```bash
# Visual Install for Python (VIP)
source /path/to/snakepit/vip-integration.sh
```

Then reload:
```bash
source ~/.bashrc
```

### Zsh

Add to `~/.zshrc`:

```zsh
# Visual Install for Python (VIP)
source /path/to/snakepit/vip-integration.zsh
```

Then reload:
```zsh
source ~/.zshrc
```

### Fish

Add to `~/.config/fish/config.fish`:

```fish
# Visual Install for Python (VIP)
source /path/to/snakepit/vip-integration.fish
```

Then reload:
```fish
source ~/.config/fish/config.fish
```

## Features

### Aliases

All shells include these aliases:

- `pip` → `vip` (visual pip)
- `pip3` → `vip`
- `python3-pip` → `vip`
- `pip-classic` → `python3 -m pip` (original pip)

### Functions

#### `vinstall`
Quick install with visualization:
```bash
vinstall package-name
```

#### `vip-gui-on`
Enable visual mode:
```bash
vip-gui-on
# ✓ Visual pip mode enabled
```

#### `vip-gui-off`
Disable visual mode (fallback to classic pip):
```bash
vip-gui-off
# ✗ Visual pip mode disabled (using classic pip)
```

#### `vip-status`
Check VIP configuration and status:
```bash
vip-status
# VIP (Visual Install for Python) Status:
#   VIP Directory: /home/user/snakepit
#   VIP in PATH: /home/user/snakepit/vip
#   GUI Mode: enabled
#   Pygame available: yes
```

### Tab Completion

#### Bash
Basic command completion for common pip subcommands.

#### Zsh
Full tab completion with descriptions:
- `vip <TAB>` shows available subcommands with descriptions
- Supports: install, uninstall, list, show, freeze, search

#### Fish
Advanced tab completion including:
- Subcommands with descriptions
- Common flags (`-h`, `--help`, `-v`, `--verbose`, etc.)
- Install-specific options (`-U`, `--upgrade`, `--user`, etc.)
- File completion for `-r/--requirement` flag

**Example in Fish:**
```fish
vip ins<TAB>        # completes to 'install'
vip install -<TAB>  # shows all flags with descriptions
vip install -r <TAB> # file browser for requirements.txt
```

## Environment Variables

### `VIP_NO_GUI`
Disable GUI mode without uninstalling:

**Bash/Zsh:**
```bash
export VIP_NO_GUI=1
```

**Fish:**
```fish
set -gx VIP_NO_GUI 1
```

Or use the helper function:
```bash
vip-gui-off
```

## Troubleshooting

### VIP not found after sourcing

**Check PATH:**
```bash
echo $PATH | grep snakepit
```

**Verify file permissions:**
```bash
ls -la /path/to/snakepit/vip*
```

All integration scripts and the vip binary should be executable.

### Aliases not working

**Check if alias exists:**
- Bash/Zsh: `alias pip`
- Fish: `alias | grep pip`

**Reload shell config:**
- Bash: `source ~/.bashrc`
- Zsh: `source ~/.zshrc`
- Fish: `source ~/.config/fish/config.fish`

### Completion not working

**Zsh:**
Ensure compinit is loaded before vip-integration.zsh:
```zsh
autoload -Uz compinit
compinit
source /path/to/snakepit/vip-integration.zsh
```

**Fish:**
Completions load automatically. If issues persist:
```fish
fish_update_completions
```

### GUI doesn't appear

Check pygame installation:
```bash
vip-status
```

If pygame is not available:
```bash
pip-classic install pygame
```

## Shell-Specific Notes

### Bash
- Uses `BASH_SOURCE` for directory detection
- Compatible with bash 4.0+
- No advanced completion features

### Zsh
- Uses zsh-specific path expansion `${0:a:h}`
- Includes `compdef` for completion system
- Shows descriptions in completion menu
- Compatible with Oh My Zsh and Prezto

### Fish
- Uses `status --current-filename` for directory detection
- Native completion system with rich descriptions
- Function-based syntax (no `function` keyword aliasing)
- Automatically integrates with fish_config

## Integration with Shell Frameworks

### Oh My Zsh
Place vip-integration.zsh sourcing **after** Oh My Zsh initialization:
```zsh
# Oh My Zsh
source $ZSH/oh-my-zsh.sh

# VIP
source /path/to/snakepit/vip-integration.zsh
```

### Prezto
Similar to Oh My Zsh, source after Prezto:
```zsh
# Prezto
source "${ZDOTDIR:-$HOME}/.zprezto/init.zsh"

# VIP
source /path/to/snakepit/vip-integration.zsh
```

### Fisher (Fish plugin manager)
Create a Fisher plugin:
```fish
fisher install /path/to/snakepit
```

Or manually in config.fish:
```fish
source /path/to/snakepit/vip-integration.fish
```

## Advanced Configuration

### Quiet Mode
Suppress loading messages by commenting out echo statements in integration files.

### Custom VIP Location
If VIP is not in the integration script directory, set manually:

**Bash/Zsh:**
```bash
export PATH="/custom/vip/location:$PATH"
source /path/to/snakepit/vip-integration.sh
```

**Fish:**
```fish
set -gx PATH /custom/vip/location $PATH
source /path/to/snakepit/vip-integration.fish
```

### Disable Specific Aliases
Comment out unwanted aliases in the integration file.

## Testing Integration

After setup, verify everything works:

```bash
# Check VIP is in PATH
which vip

# Test alias
type pip

# Test function
vip-status

# Test tab completion (shell-dependent)
vip <TAB>

# Test actual installation
pip install requests
```

## Uninstalling

Remove the source line from your shell config:
- `~/.bashrc` (bash)
- `~/.zshrc` (zsh)
- `~/.config/fish/config.fish` (fish)

Then reload your shell or start a new session.

## Contributing

To add support for other shells (e.g., tcsh, ksh):
1. Create `vip-integration.<shell>` following existing patterns
2. Update `install-visual.sh` with detection and integration logic
3. Add documentation to this file
4. Test thoroughly with that shell

## See Also

- [VISUAL_SETUP.md](VISUAL_SETUP.md) - Main VIP documentation
- [README.md](README.md) - Snakepit overview
- [examples/](examples/) - Usage examples
