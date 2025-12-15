# 🐍 InstallSnake Quick Start

## TL;DR

```bash
cd /home/adminx/snakepit
cargo build --release
./target/release/snakepit game
```

**Done!** Snake will appear in your terminal, moving right with pellets spawning randomly.

---

## 5 Themes to Try

```bash
# 1. Classic green (default)
./target/release/snakepit game --theme retro

# 2. Vintage amber terminal
./target/release/snakepit game --theme amber

# 3. Neon Matrix style  
./target/release/snakepit game --theme matrix

# 4. Minimal ASCII (works on any terminal)
./target/release/snakepit game --theme minimal

# 5. Red debug mode
./target/release/snakepit game --theme error
```

---

## Customization

### Adjust Speed (FPS)

```bash
# Slow (10 FPS - easier to watch)
./target/release/snakepit game --fps 10

# Fast (20 FPS - action packed)
./target/release/snakepit game --fps 20

# Turbo (30 FPS - maximum chaos)
./target/release/snakepit game --fps 30
```

### Change Board Size

```bash
# Small board (40 chars wide)
./target/release/snakepit game --width 40

# Large board (80 chars wide)
./target/release/snakepit game --width 80
```

### Mix & Match

```bash
# Amber theme, high speed, large board
./target/release/snakepit game --theme amber --fps 18 --width 70

# Matrix style, slow motion, tiny board
./target/release/snakepit game --theme matrix --fps 8 --width 35
```

---

## What You're Seeing

```
┌──●───────────────────────────────────┐
│                                      │
│           ●                          │
│                                      │
│         ●     ●●                     │
│                ►                     │
│           ●                          │
│                                      │
└──────────────────────────────────────┘
Packages: 2/5
Snake length: 6 segments
Crashes: 0
```

- **►** = Snake head (moving right)
- **█** = Snake body
- **●** = Package pellet (target)
- **✗** = Failed package
- **Segments grow** = Each completed package adds 3 body segments
- **Crashes** = When a build fails, the snake crashes (red flash)

---

## How It Works

1. **5 packages spawn** as pellets (●) at random positions on the board
2. **AI plays itself** - Snake autonomously pathfinds to the nearest pellet
   - Calculates shortest path (Manhattan distance)
   - Recalculates every 10 frames
   - Intelligently navigates obstacles
3. **When snake eats pellet** → Grows 3 segments, package marked complete
4. **Realistic pip events**:
   - Download phase → pellet appears & grows
   - Build phase → pellet animates
   - Install complete → snake eats pellet
5. **Demo lasts 15 seconds** → Final summary shows completion rate

**Each package gets staggered timing**, so the install lifecycle plays out realistically!

Eventually this will integrate with real `pip install`, but the AI engine already makes for entertaining autonomous play.

---

## For Developers

### Build from source
```bash
cargo build --release
# Binary at: target/release/snakepit
```

### Run tests
```bash
cargo test --release 2>&1 | grep "test game"
```

### Debug mode
```bash
cargo run -- game --theme retro --fps 5 --width 50
```

### Files

- `src/installsnake.rs` - Core game engine (481 LOC)
- `src/game_runner.rs` - Event system & pip parser (253 LOC)
- `INSTALLSNAKE.md` - Full documentation
- `INSTALLSNAKE_SUMMARY.md` - Implementation details

---

## Next Features (Roadmap)

- [ ] Real pip subprocess integration
- [ ] Keyboard controls (arrow keys to steer)
- [ ] High score persistence
- [ ] Sound effects
- [ ] Multiplayer mode
- [ ] Config file (~/.config/snakepit/game.toml)

---

## Troubleshooting

**Game won't start:**
```bash
cargo build --release  # Rebuild first
```

**Graphics look wrong:**
```bash
export TERM=xterm-256color
./target/release/snakepit game --theme minimal  # Use minimal theme
```

**Too fast/slow:**
```bash
./target/release/snakepit game --fps 10  # Adjust FPS
```

---

**Enjoy!** 🐍🎮
