#!/bin/bash

# InstallSnake Demo Runner
# Run different themed games for testing

set -e

GAME_BIN="./target/release/snakepit"

if [ ! -f "$GAME_BIN" ]; then
    echo "Building snakepit..."
    cargo build --release
fi

echo "🐍 InstallSnake Theme Showcase"
echo "=============================="
echo ""

show_menu() {
    echo "Select theme:"
    echo "1) Retro (green classic CRT)"
    echo "2) Amber (vintage terminal)"
    echo "3) Matrix (neon glitch)"
    echo "4) Minimal (ASCII monochrome)"
    echo "5) Error (red debug mode)"
    echo "6) All themes (15 sec each)"
    echo ""
    read -p "Choice [1-6]: " choice
}

run_game() {
    local theme=$1
    local duration=${2:-15}
    echo ""
    echo "▶ Running $theme theme for ${duration}s..."
    echo ""
    timeout $duration $GAME_BIN game --theme "$theme" --fps 12 --width 60 || true
    echo ""
}

show_menu

case $choice in
    1) run_game retro ;;
    2) run_game amber ;;
    3) run_game matrix ;;
    4) run_game minimal ;;
    5) run_game error ;;
    6)
        run_game retro 15
        run_game amber 15
        run_game matrix 15
        run_game minimal 15
        run_game error 15
        ;;
    *) echo "Invalid choice"; exit 1 ;;
esac

echo ""
echo "✓ Demo complete!"
