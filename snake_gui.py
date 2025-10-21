#!/usr/bin/env python3
"""
InstallSnake GUI - A modern pygame-based snake game
Visualizes Python package installation as an engaging game
"""

import pygame
import random
import sys
from enum import Enum
from dataclasses import dataclass
from typing import List, Tuple, Optional
from collections import deque


# Constants
WINDOW_WIDTH = 800
WINDOW_HEIGHT = 600
GRID_SIZE = 20
GRID_WIDTH = WINDOW_WIDTH // GRID_SIZE
GRID_HEIGHT = WINDOW_HEIGHT // GRID_SIZE
FPS = 12

# Colors
BLACK = (0, 0, 0)
WHITE = (255, 255, 255)
DARK_GREEN = (0, 100, 0)
BRIGHT_GREEN = (0, 255, 0)
LIME = (50, 205, 50)
RED = (255, 0, 0)
DARK_RED = (139, 0, 0)
YELLOW = (255, 255, 0)
ORANGE = (255, 165, 0)
CYAN = (0, 255, 255)
PURPLE = (128, 0, 128)
GRAY = (50, 50, 50)
DARK_GRAY = (30, 30, 30)


class Direction(Enum):
    UP = (0, -1)
    DOWN = (0, 1)
    LEFT = (-1, 0)
    RIGHT = (1, 0)


class PelletState(Enum):
    QUEUED = 1
    DOWNLOADING = 2
    BUILDING = 3
    READY = 4
    EATEN = 5
    FAILED = 6


@dataclass
class Position:
    x: int
    y: int
    
    def __eq__(self, other):
        return self.x == other.x and self.y == other.y
    
    def __hash__(self):
        return hash((self.x, self.y))


@dataclass
class Pellet:
    pos: Position
    package_name: str
    state: PelletState
    progress: float = 0.0
    
    def get_color(self):
        return {
            PelletState.QUEUED: WHITE,
            PelletState.DOWNLOADING: CYAN,
            PelletState.BUILDING: ORANGE,
            PelletState.READY: BRIGHT_GREEN,
            PelletState.EATEN: BLACK,
            PelletState.FAILED: RED,
        }.get(self.state, WHITE)


class SnakeGame:
    def __init__(self):
        pygame.init()
        self.screen = pygame.display.set_mode((WINDOW_WIDTH, WINDOW_HEIGHT))
        pygame.display.set_caption("InstallSnake - Package Manager Game")
        self.clock = pygame.time.Clock()
        self.font = pygame.font.Font(None, 24)
        self.title_font = pygame.font.Font(None, 48)
        self.small_font = pygame.font.Font(None, 18)
        
        # Game state
        self.reset_game()
        
        # Sample packages for the game
        self.package_pool = [
            "numpy", "pandas", "scipy", "matplotlib", "scikit-learn",
            "tensorflow", "pytorch", "flask", "django", "requests",
            "pillow", "opencv", "keras", "sqlalchemy", "pytest"
        ]
        
        # Spawn initial pellets
        for _ in range(5):
            self.spawn_pellet()
    
    def reset_game(self):
        """Initialize/reset game state"""
        center_x = GRID_WIDTH // 2
        center_y = GRID_HEIGHT // 2
        self.snake = deque([Position(center_x, center_y)])
        self.direction = Direction.RIGHT
        self.next_direction = Direction.RIGHT
        self.pellets: List[Pellet] = []
        self.obstacles: List[Position] = []
        self.score = 0
        self.crashes = 0
        self.game_over = False
        self.paused = False
        self.speed_boost = 0
        self.frame_count = 0
        
        # Generate obstacles
        self.generate_obstacles()
    
    def generate_obstacles(self):
        """Create maze-like obstacles"""
        # Vertical barriers
        for x in [GRID_WIDTH // 4, GRID_WIDTH * 3 // 4]:
            for y in range(2, GRID_HEIGHT - 2):
                if random.random() < 0.3:
                    self.obstacles.append(Position(x, y))
        
        # Horizontal barriers
        for y in [GRID_HEIGHT // 3, GRID_HEIGHT * 2 // 3]:
            for x in range(3, GRID_WIDTH - 3):
                if random.random() < 0.25:
                    self.obstacles.append(Position(x, y))
    
    def spawn_pellet(self):
        """Spawn a new package pellet"""
        while True:
            x = random.randint(1, GRID_WIDTH - 2)
            y = random.randint(1, GRID_HEIGHT - 2)
            pos = Position(x, y)
            
            # Check if position is free
            if (pos not in self.snake and 
                pos not in self.obstacles and
                not any(p.pos == pos for p in self.pellets)):
                
                package_name = random.choice(self.package_pool)
                state = random.choice([
                    PelletState.QUEUED,
                    PelletState.DOWNLOADING,
                    PelletState.BUILDING,
                    PelletState.READY
                ])
                
                pellet = Pellet(pos, package_name, state, random.random())
                self.pellets.append(pellet)
                break
    
    def handle_input(self):
        """Process keyboard input"""
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                return False
            
            if event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    return False
                
                if event.key == pygame.K_SPACE:
                    self.paused = not self.paused
                
                if event.key == pygame.K_r and self.game_over:
                    self.reset_game()
                    for _ in range(5):
                        self.spawn_pellet()
                
                # Direction controls
                if not self.paused and not self.game_over:
                    if event.key == pygame.K_UP and self.direction != Direction.DOWN:
                        self.next_direction = Direction.UP
                    elif event.key == pygame.K_DOWN and self.direction != Direction.UP:
                        self.next_direction = Direction.DOWN
                    elif event.key == pygame.K_LEFT and self.direction != Direction.RIGHT:
                        self.next_direction = Direction.LEFT
                    elif event.key == pygame.K_RIGHT and self.direction != Direction.LEFT:
                        self.next_direction = Direction.RIGHT
        
        return True
    
    def update(self):
        """Update game state"""
        if self.paused or self.game_over:
            return
        
        self.frame_count += 1
        self.direction = self.next_direction
        
        # Move snake
        head = self.snake[0]
        dx, dy = self.direction.value
        new_head = Position(head.x + dx, head.y + dy)
        
        # Check collisions
        if (new_head.x < 0 or new_head.x >= GRID_WIDTH or
            new_head.y < 0 or new_head.y >= GRID_HEIGHT or
            new_head in self.snake or
            new_head in self.obstacles):
            self.game_over = True
            self.crashes += 1
            return
        
        self.snake.appendleft(new_head)
        
        # Check pellet collision
        eaten_pellet = None
        for pellet in self.pellets:
            if pellet.pos == new_head and pellet.state != PelletState.EATEN:
                if pellet.state == PelletState.FAILED:
                    # Eating a failed package causes crash
                    self.crashes += 1
                    self.game_over = True
                    return
                else:
                    eaten_pellet = pellet
                    break
        
        if eaten_pellet:
            self.score += 1
            eaten_pellet.state = PelletState.EATEN
            self.speed_boost = 5
            
            # Spawn new pellet
            if len([p for p in self.pellets if p.state != PelletState.EATEN]) < 8:
                self.spawn_pellet()
        else:
            # Remove tail if no pellet eaten
            self.snake.pop()
        
        # Update pellet states (simulate package installation progress)
        if self.frame_count % 10 == 0:
            for pellet in self.pellets:
                if pellet.state == PelletState.QUEUED:
                    pellet.state = PelletState.DOWNLOADING
                elif pellet.state == PelletState.DOWNLOADING:
                    pellet.progress = min(1.0, pellet.progress + 0.1)
                    if pellet.progress >= 1.0:
                        pellet.state = PelletState.BUILDING
                        pellet.progress = 0.0
                elif pellet.state == PelletState.BUILDING:
                    pellet.progress = min(1.0, pellet.progress + 0.15)
                    if pellet.progress >= 1.0:
                        # Random chance of failure
                        if random.random() < 0.05:
                            pellet.state = PelletState.FAILED
                        else:
                            pellet.state = PelletState.READY
    
    def draw(self):
        """Render the game"""
        self.screen.fill(BLACK)
        
        # Draw grid background
        for x in range(0, WINDOW_WIDTH, GRID_SIZE):
            pygame.draw.line(self.screen, DARK_GRAY, (x, 0), (x, WINDOW_HEIGHT), 1)
        for y in range(0, WINDOW_HEIGHT, GRID_SIZE):
            pygame.draw.line(self.screen, DARK_GRAY, (0, y), (WINDOW_WIDTH, y), 1)
        
        # Draw obstacles
        for obs in self.obstacles:
            rect = pygame.Rect(obs.x * GRID_SIZE, obs.y * GRID_SIZE, GRID_SIZE, GRID_SIZE)
            pygame.draw.rect(self.screen, GRAY, rect)
            pygame.draw.rect(self.screen, WHITE, rect, 1)
        
        # Draw pellets
        for pellet in self.pellets:
            if pellet.state != PelletState.EATEN:
                rect = pygame.Rect(pellet.pos.x * GRID_SIZE, pellet.pos.y * GRID_SIZE, 
                                 GRID_SIZE, GRID_SIZE)
                
                color = pellet.get_color()
                pygame.draw.circle(self.screen, color, rect.center, GRID_SIZE // 3)
                
                # Draw progress bar for downloading/building
                if pellet.state in [PelletState.DOWNLOADING, PelletState.BUILDING]:
                    progress_width = int((GRID_SIZE - 4) * pellet.progress)
                    progress_rect = pygame.Rect(pellet.pos.x * GRID_SIZE + 2,
                                               pellet.pos.y * GRID_SIZE + GRID_SIZE - 4,
                                               progress_width, 2)
                    pygame.draw.rect(self.screen, YELLOW, progress_rect)
        
        # Draw snake
        for i, segment in enumerate(self.snake):
            rect = pygame.Rect(segment.x * GRID_SIZE, segment.y * GRID_SIZE, 
                             GRID_SIZE, GRID_SIZE)
            
            if i == 0:  # Head
                pygame.draw.rect(self.screen, BRIGHT_GREEN, rect)
                pygame.draw.rect(self.screen, LIME, rect, 2)
                
                # Draw eyes
                eye_size = 3
                if self.direction == Direction.UP:
                    eye_left = (segment.x * GRID_SIZE + 5, segment.y * GRID_SIZE + 5)
                    eye_right = (segment.x * GRID_SIZE + 15, segment.y * GRID_SIZE + 5)
                elif self.direction == Direction.DOWN:
                    eye_left = (segment.x * GRID_SIZE + 5, segment.y * GRID_SIZE + 15)
                    eye_right = (segment.x * GRID_SIZE + 15, segment.y * GRID_SIZE + 15)
                elif self.direction == Direction.LEFT:
                    eye_left = (segment.x * GRID_SIZE + 5, segment.y * GRID_SIZE + 5)
                    eye_right = (segment.x * GRID_SIZE + 5, segment.y * GRID_SIZE + 15)
                else:  # RIGHT
                    eye_left = (segment.x * GRID_SIZE + 15, segment.y * GRID_SIZE + 5)
                    eye_right = (segment.x * GRID_SIZE + 15, segment.y * GRID_SIZE + 15)
                
                pygame.draw.circle(self.screen, BLACK, eye_left, eye_size)
                pygame.draw.circle(self.screen, BLACK, eye_right, eye_size)
            else:  # Body
                pygame.draw.rect(self.screen, DARK_GREEN, rect)
                pygame.draw.rect(self.screen, LIME, rect, 1)
        
        # Draw UI
        self.draw_ui()
        
        pygame.display.flip()
    
    def draw_ui(self):
        """Draw score, instructions, and status"""
        # Score
        score_text = self.font.render(f"Score: {self.score}", True, WHITE)
        self.screen.blit(score_text, (10, 10))
        
        # Snake length
        length_text = self.font.render(f"Length: {len(self.snake)}", True, WHITE)
        self.screen.blit(length_text, (10, 40))
        
        # Crashes
        crashes_text = self.font.render(f"Crashes: {self.crashes}", True, RED)
        self.screen.blit(crashes_text, (10, 70))
        
        # Active packages
        active = len([p for p in self.pellets if p.state != PelletState.EATEN])
        packages_text = self.small_font.render(f"Active Packages: {active}", True, CYAN)
        self.screen.blit(packages_text, (WINDOW_WIDTH - 200, 10))
        
        # Instructions
        instructions = [
            "Arrow Keys: Move",
            "Space: Pause",
            "ESC: Quit"
        ]
        for i, text in enumerate(instructions):
            inst_text = self.small_font.render(text, True, GRAY)
            self.screen.blit(inst_text, (WINDOW_WIDTH - 200, 40 + i * 20))
        
        # Paused overlay
        if self.paused:
            overlay = pygame.Surface((WINDOW_WIDTH, WINDOW_HEIGHT))
            overlay.set_alpha(128)
            overlay.fill(BLACK)
            self.screen.blit(overlay, (0, 0))
            
            pause_text = self.title_font.render("PAUSED", True, YELLOW)
            text_rect = pause_text.get_rect(center=(WINDOW_WIDTH // 2, WINDOW_HEIGHT // 2))
            self.screen.blit(pause_text, text_rect)
        
        # Game over overlay
        if self.game_over:
            overlay = pygame.Surface((WINDOW_WIDTH, WINDOW_HEIGHT))
            overlay.set_alpha(180)
            overlay.fill(DARK_RED)
            self.screen.blit(overlay, (0, 0))
            
            game_over_text = self.title_font.render("GAME OVER", True, WHITE)
            score_final = self.font.render(f"Final Score: {self.score}", True, WHITE)
            restart_text = self.font.render("Press R to Restart", True, YELLOW)
            
            y_offset = WINDOW_HEIGHT // 2 - 60
            for text in [game_over_text, score_final, restart_text]:
                text_rect = text.get_rect(center=(WINDOW_WIDTH // 2, y_offset))
                self.screen.blit(text, text_rect)
                y_offset += 50
    
    def run(self):
        """Main game loop"""
        running = True
        
        while running:
            running = self.handle_input()
            self.update()
            self.draw()
            
            current_fps = FPS + (self.speed_boost if self.speed_boost > 0 else 0)
            self.clock.tick(current_fps)
            
            if self.speed_boost > 0:
                self.speed_boost -= 1
        
        pygame.quit()
        sys.exit()


def main():
    game = SnakeGame()
    game.run()


if __name__ == "__main__":
    main()
