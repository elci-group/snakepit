# Shell Integration Quick Reference

## Installation Commands

| Shell | Install Command |
|-------|----------------|
| Bash  | `./install-visual.sh` (auto-detects) |
| Zsh   | `./install-visual.sh` (auto-detects) |
| Fish  | `./install-visual.sh` (auto-detects) |

## Shell Config Files

| Shell | Config File | Integration Script |
|-------|-------------|-------------------|
| Bash  | `~/.bashrc` | `vip-integration.sh` |
| Zsh   | `~/.zshrc`  | `vip-integration.zsh` |
| Fish  | `~/.config/fish/config.fish` | `vip-integration.fish` |

## Reload Commands

| Shell | Reload Command |
|-------|---------------|
| Bash  | `source ~/.bashrc` |
| Zsh   | `source ~/.zshrc` or `exec zsh` |
| Fish  | `source ~/.config/fish/config.fish` |

## Universal Commands (All Shells)

| Command | Description |
|---------|-------------|
| `pip install <pkg>` | Install with snake animation |
| `pip-classic install <pkg>` | Original pip (no animation) |
| `vinstall <pkg>` | Shorthand for visual install |
| `vip-status` | Check VIP configuration |
| `vip-gui-on` | Enable visual mode |
| `vip-gui-off` | Disable visual mode |

## Shell-Specific Syntax

### Environment Variable (Disable GUI)

**Bash/Zsh:**
```bash
export VIP_NO_GUI=1
```

**Fish:**
```fish
set -gx VIP_NO_GUI 1
```

### Check Alias

**Bash/Zsh:**
```bash
alias pip
# Output: alias pip='vip'
```

**Fish:**
```fish
alias | grep pip
# Output: alias pip='vip'
```

### Check Function

**Bash/Zsh:**
```bash
type vip-status
# Output: vip-status is a function
```

**Fish:**
```fish
type vip-status
# Output: vip-status is a function with definition...
```

## Tab Completion Support

| Shell | Completion Level |
|-------|-----------------|
| Bash  | Basic subcommands |
| Zsh   | Subcommands + descriptions |
| Fish  | Full (subcommands, flags, file paths) |

### Try Tab Completion

**All Shells:**
```bash
vip <TAB>          # Show subcommands
pip ins<TAB>       # Complete to 'install'
```

**Fish Only:**
```fish
pip install -<TAB>  # Show all flags with descriptions
pip install -r <TAB> # File browser
```

## PATH Verification

**All Shells:**
```bash
echo $PATH | grep snakepit
which vip
```

## Troubleshooting Quick Fixes

### VIP not found
```bash
# Add manually (replace with your path)
export PATH="/home/user/snakepit:$PATH"
```

### Aliases not working
```bash
# Re-source config
source ~/.bashrc  # or ~/.zshrc or ~/.config/fish/config.fish
```

### Completion not working (Zsh)
```zsh
# Add before vip-integration.zsh
autoload -Uz compinit && compinit
```

### Check Pygame
```bash
python3 -c "import pygame; print('Pygame OK')"
```

## Manual Integration (Copy-Paste)

### Bash
```bash
echo 'source ~/snakepit/vip-integration.sh' >> ~/.bashrc
source ~/.bashrc
```

### Zsh
```zsh
echo 'source ~/snakepit/vip-integration.zsh' >> ~/.zshrc
source ~/.zshrc
```

### Fish
```fish
echo 'source ~/snakepit/vip-integration.fish' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

## Testing One-Liner

```bash
vip-status && pip install colorama
```

Expected output:
- Status showing VIP configuration
- Snake animation during install
- Success message

## See Full Documentation

- [SHELL_INTEGRATION.md](SHELL_INTEGRATION.md) - Complete guide
- [VISUAL_SETUP.md](VISUAL_SETUP.md) - VIP setup and usage
