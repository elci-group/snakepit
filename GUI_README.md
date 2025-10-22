# InstallSnake GUI

A modern, visually appealing snake game built with pygame that visualizes Python package installation as an engaging game experience.

## Features

- **Smooth Graphics**: Clean pygame-based GUI with grid-based movement
- **Visual Feedback**: Color-coded packages showing installation states:
  - White: Queued
  - Cyan: Downloading
  - Orange: Building
  - Green: Ready to install
  - Red: Failed installation
- **Progress Bars**: Real-time progress indicators for downloading/building packages
- **Obstacles**: Maze-like barriers for added challenge
- **Snake Eyes**: Animated eyes that follow the direction of movement
- **Pause/Resume**: Press Space to pause the game
- **Score Tracking**: Keep track of your score, length, and crashes
- **Speed Boost**: Temporary speed increase after eating packages

## Installation

1. Make sure you have Python 3 installed
2. Install pygame:
   ```bash
   pip install pygame
   ```

## Running the Game

```bash
python snake_gui.py
# or
./snake_gui.py
```

## Controls

- **Arrow Keys**: Move the snake (Up, Down, Left, Right)
- **Space**: Pause/Resume game
- **R**: Restart after game over
- **ESC**: Quit the game

## Gameplay

1. Navigate the snake using arrow keys
2. Collect package pellets to grow your snake and increase your score
3. Avoid hitting:
   - Walls (screen boundaries)
   - Obstacles (gray blocks)
   - Your own body
4. Watch out for red failed packages - they'll crash your game!
5. Packages progress through states: Queued → Downloading → Building → Ready
6. Collect as many packages as possible to achieve a high score

## Game Elements

- **Green Snake**: Your character (bright green head with eyes, dark green body)
- **Colored Circles**: Package pellets in various installation states
- **Gray Blocks**: Obstacles to avoid
- **Progress Bars**: Yellow bars showing download/build progress

## Tips

- Failed packages (red) are dangerous - avoid them!
- Speed boost activates briefly after eating a package
- Plan your path around obstacles
- The snake grows by 1 segment each time you collect a package

Enjoy playing InstallSnake!
