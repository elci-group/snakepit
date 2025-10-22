#!/usr/bin/env python3
"""
InstallSnake Monitor - Automated GUI that visualizes real pip installations
Monitors subprocess output and automatically controls the snake
"""

import pygame
import random
import sys
import re
import threading
import queue
from enum import Enum
from dataclasses import dataclass
from typing import List, Optional
from collections import deque
import time


# Constants
WINDOW_WIDTH = 1000
WINDOW_HEIGHT = 700
GRID_SIZE = 20
GRID_WIDTH = WINDOW_WIDTH // GRID_SIZE
GRID_HEIGHT = (WINDOW_HEIGHT - 100) // GRID_SIZE  # Reserve space for status bar
FPS = 15

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
GRAY = (50, 50, 50)
DARK_GRAY = (30, 30, 30)
BLUE = (0, 100, 255)


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
    size_mb: float = 0.0
    
    def get_color(self):
        return {
            PelletState.QUEUED: WHITE,
            PelletState.DOWNLOADING: CYAN,
            PelletState.BUILDING: ORANGE,
            PelletState.READY: BRIGHT_GREEN,
            PelletState.EATEN: BLACK,
            PelletState.FAILED: RED,
        }.get(self.state, WHITE)


class InstallEvent:
    """Events from pip subprocess"""
    def __init__(self, event_type, **kwargs):
        self.type = event_type
        self.data = kwargs


class SnakeMonitor:
    def __init__(self, event_queue: queue.Queue):
        pygame.init()
        self.screen = pygame.display.set_mode((WINDOW_WIDTH, WINDOW_HEIGHT))
        pygame.display.set_caption("InstallSnake - Live Package Monitor")
        self.clock = pygame.time.Clock()
        self.font = pygame.font.Font(None, 20)
        self.title_font = pygame.font.Font(None, 36)
        self.small_font = pygame.font.Font(None, 16)
        
        # Event queue from pip subprocess
        self.event_queue = event_queue
        
        # Game state
        self.reset_game()
        
        # AI control
        self.ai_enabled = True
        self.ai_path = []
        self.ai_recalc_counter = 0
        
        # Status messages
        self.status_messages = deque(maxlen=5)
        self.completion_message = None
        self.start_time = time.time()
    
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
        self.total_packages = 0
        self.completed_packages = 0
        self.failed_packages = 0
        self.running = True
        self.finished = False
        self.frame_count = 0
        
        # Generate obstacles
        self.generate_obstacles()
    
    def generate_obstacles(self):
        """Create minimal obstacles"""
        # Very minimal obstacles for smooth automated play
        for x in [GRID_WIDTH // 3, GRID_WIDTH * 2 // 3]:
            for y in range(5, GRID_HEIGHT - 5, 5):
                if random.random() < 0.08:
                    self.obstacles.append(Position(x, y))
    
    def add_status_message(self, message: str):
        """Add a status message to the display"""
        self.status_messages.append(f"[{time.strftime('%H:%M:%S')}] {message}")
    
    def handle_event(self, event: InstallEvent):
        """Process installation events"""
        if event.type == "package_queued":
            pkg_name = event.data.get("name", "unknown")
            self.add_status_message(f"📦 Queued: {pkg_name}")
            self.spawn_pellet(pkg_name, PelletState.QUEUED)
            self.total_packages += 1
        
        elif event.type == "download_started":
            pkg_name = event.data.get("name", "")
            size_mb = event.data.get("size_mb", 0)
            self.add_status_message(f"⬇️  Downloading: {pkg_name} ({size_mb:.1f}MB)")
            self.update_pellet_state(pkg_name, PelletState.DOWNLOADING, size_mb=size_mb)
        
        elif event.type == "download_progress":
            pkg_name = event.data.get("name", "")
            progress = event.data.get("progress", 0)
            self.update_pellet_progress(pkg_name, progress)
        
        elif event.type == "build_started":
            pkg_name = event.data.get("name", "")
            self.add_status_message(f"🔨 Building: {pkg_name}")
            self.update_pellet_state(pkg_name, PelletState.BUILDING)
        
        elif event.type == "build_progress":
            pkg_name = event.data.get("name", "")
            progress = event.data.get("progress", 0)
            self.update_pellet_progress(pkg_name, progress)
        
        elif event.type == "install_complete":
            pkg_name = event.data.get("name", "")
            self.add_status_message(f"✅ Installed: {pkg_name}")
            self.update_pellet_state(pkg_name, PelletState.READY)
            self.completed_packages += 1
        
        elif event.type == "install_failed":
            pkg_name = event.data.get("name", "")
            error = event.data.get("error", "Unknown error")
            self.add_status_message(f"❌ Failed: {pkg_name}")
            self.update_pellet_state(pkg_name, PelletState.FAILED)
            self.failed_packages += 1
        
        elif event.type == "all_complete":
            elapsed = time.time() - self.start_time
            self.completion_message = f"Installation complete in {elapsed:.1f}s"
            self.add_status_message(f"🎉 {self.completion_message}")
            # Keep running until all pellets are collected
            threading.Timer(8.0, self.signal_finish).start()
    
    def signal_finish(self):
        """Signal that we should finish"""
        self.finished = True
    
    def spawn_pellet(self, name: str, state: PelletState):
        """Spawn a new package pellet"""
        # Check if pellet already exists
        for pellet in self.pellets:
            if pellet.package_name == name:
                return
        
        # Find free position
        for _ in range(50):  # Try 50 times
            x = random.randint(2, GRID_WIDTH - 3)
            y = random.randint(2, GRID_HEIGHT - 3)
            pos = Position(x, y)
            
            if (pos not in self.snake and 
                pos not in self.obstacles and
                not any(p.pos == pos for p in self.pellets)):
                
                pellet = Pellet(pos, name, state)
                self.pellets.append(pellet)
                break
    
    def update_pellet_state(self, name: str, state: PelletState, size_mb: float = 0.0):
        """Update pellet state"""
        for pellet in self.pellets:
            if pellet.package_name == name or name in pellet.package_name:
                pellet.state = state
                if size_mb > 0:
                    pellet.size_mb = size_mb
                break
    
    def update_pellet_progress(self, name: str, progress: float):
        """Update pellet progress"""
        for pellet in self.pellets:
            if pellet.package_name == name or name in pellet.package_name:
                pellet.progress = max(0.0, min(1.0, progress))
                break
    
    def calculate_ai_path(self):
        """Calculate path to nearest pellet using simple greedy approach"""
        if not self.snake or not self.pellets:
            return
        
        head = self.snake[0]
        
        # Find nearest uneaten pellet (prioritize READY state)
        target = None
        min_dist = float('inf')
        
        for pellet in self.pellets:
            if pellet.state in [PelletState.READY, PelletState.DOWNLOADING, PelletState.BUILDING, PelletState.QUEUED]:
                dx = abs(head.x - pellet.pos.x)
                dy = abs(head.y - pellet.pos.y)
                dist = dx + dy
                
                # Prioritize READY pellets strongly
                if pellet.state == PelletState.READY:
                    dist *= 0.3
                elif pellet.state == PelletState.BUILDING:
                    dist *= 0.7
                
                if dist < min_dist:
                    min_dist = dist
                    target = pellet
        
        if not target:
            return
        
        # Clear and calculate immediate next move only
        self.ai_path.clear()
        
        dx = target.pos.x - head.x
        dy = target.pos.y - head.y
        
        # Determine best direction
        possible_moves = []
        
        # Add moves that get us closer
        if dx > 0:
            possible_moves.append((Direction.RIGHT, abs(dx - 1) + abs(dy)))
        elif dx < 0:
            possible_moves.append((Direction.LEFT, abs(dx + 1) + abs(dy)))
        
        if dy > 0:
            possible_moves.append((Direction.DOWN, abs(dx) + abs(dy - 1)))
        elif dy < 0:
            possible_moves.append((Direction.UP, abs(dx) + abs(dy + 1)))
        
        # Sort by distance (lower is better)
        possible_moves.sort(key=lambda x: x[1])
        
        # Try each move in order
        for direction, _ in possible_moves:
            next_x = head.x + direction.value[0]
            next_y = head.y + direction.value[1]
            next_pos = Position(next_x, next_y)
            
            # Check if valid move
            if (0 <= next_x < GRID_WIDTH and 
                0 <= next_y < GRID_HEIGHT and
                next_pos not in self.obstacles and
                next_pos not in list(self.snake)[1:]):
                
                self.ai_path.append(direction)
                return
        
        # If stuck, try any valid direction
        for direction in [Direction.RIGHT, Direction.LEFT, Direction.DOWN, Direction.UP]:
            next_x = head.x + direction.value[0]
            next_y = head.y + direction.value[1]
            next_pos = Position(next_x, next_y)
            
            if (0 <= next_x < GRID_WIDTH and 
                0 <= next_y < GRID_HEIGHT and
                next_pos not in self.obstacles and
                next_pos not in list(self.snake)[1:]):
                
                self.ai_path.append(direction)
                return
    
    def handle_input(self):
        """Process events"""
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                self.running = False
                return False
            
            if event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    self.running = False
                    return False
        
        return True
    
    def update(self):
        """Update game state"""
        # Process events from queue
        try:
            while True:
                event = self.event_queue.get_nowait()
                self.handle_event(event)
        except queue.Empty:
            pass
        
        self.frame_count += 1
        
        # AI pathfinding - recalculate every frame for responsiveness
        if self.ai_enabled and self.snake and self.pellets:
            self.calculate_ai_path()
            
            if self.ai_path:
                self.direction = self.ai_path.pop(0)
            # If no path, keep current direction
        
        self.next_direction = self.direction
        
        # Move snake
        if self.snake:
            head = self.snake[0]
            dx, dy = self.direction.value
            new_head = Position(head.x + dx, head.y + dy)
            
            # Check collisions
            if (new_head.x < 0 or new_head.x >= GRID_WIDTH or
                new_head.y < 0 or new_head.y >= GRID_HEIGHT or
                new_head in self.snake or
                new_head in self.obstacles):
                # Respawn shorter snake on collision
                if len(self.snake) > 3:
                    while len(self.snake) > 3:
                        self.snake.pop()
                # Recalculate path
                self.ai_path.clear()
                return
            
            self.snake.appendleft(new_head)
            
            # Check pellet collision
            eaten_pellet = None
            for pellet in self.pellets:
                if pellet.pos == new_head and pellet.state != PelletState.EATEN:
                    eaten_pellet = pellet
                    break
            
            if eaten_pellet:
                if eaten_pellet.state == PelletState.READY:
                    self.score += 1
                    eaten_pellet.state = PelletState.EATEN
                    self.add_status_message(f"✅ Collected: {eaten_pellet.package_name}")
                    # Grow snake
                    for _ in range(3):
                        if self.snake:
                            self.snake.append(self.snake[-1])
                elif eaten_pellet.state == PelletState.FAILED:
                    # Shrink on failed package
                    self.add_status_message(f"💥 Avoided failed: {eaten_pellet.package_name}")
                    if len(self.snake) > 2:
                        self.snake.pop()
                    eaten_pellet.state = PelletState.EATEN
                else:
                    # Can also eat downloading/building packages
                    eaten_pellet.state = PelletState.EATEN
            else:
                # Remove tail only if not growing
                if len(self.snake) > 1:
                    self.snake.pop()
    
    def draw(self):
        """Render the game"""
        self.screen.fill(BLACK)
        
        # Draw grid
        for x in range(0, GRID_WIDTH * GRID_SIZE, GRID_SIZE):
            pygame.draw.line(self.screen, DARK_GRAY, (x, 0), (x, GRID_HEIGHT * GRID_SIZE), 1)
        for y in range(0, GRID_HEIGHT * GRID_SIZE, GRID_SIZE):
            pygame.draw.line(self.screen, DARK_GRAY, (0, y), (GRID_WIDTH * GRID_SIZE, y), 1)
        
        # Draw obstacles
        for obs in self.obstacles:
            rect = pygame.Rect(obs.x * GRID_SIZE, obs.y * GRID_SIZE, GRID_SIZE, GRID_SIZE)
            pygame.draw.rect(self.screen, GRAY, rect)
        
        # Draw pellets
        for pellet in self.pellets:
            if pellet.state != PelletState.EATEN:
                rect = pygame.Rect(pellet.pos.x * GRID_SIZE, pellet.pos.y * GRID_SIZE, 
                                 GRID_SIZE, GRID_SIZE)
                
                color = pellet.get_color()
                pygame.draw.circle(self.screen, color, rect.center, GRID_SIZE // 3)
                
                # Progress bar
                if pellet.state in [PelletState.DOWNLOADING, PelletState.BUILDING]:
                    progress_width = int((GRID_SIZE - 4) * pellet.progress)
                    if progress_width > 0:
                        progress_rect = pygame.Rect(pellet.pos.x * GRID_SIZE + 2,
                                                   pellet.pos.y * GRID_SIZE + GRID_SIZE - 4,
                                                   progress_width, 2)
                        pygame.draw.rect(self.screen, YELLOW, progress_rect)
                
                # Package name
                if pellet.state in [PelletState.DOWNLOADING, PelletState.BUILDING]:
                    name_text = self.small_font.render(pellet.package_name[:8], True, WHITE)
                    self.screen.blit(name_text, (pellet.pos.x * GRID_SIZE + 2, 
                                                 pellet.pos.y * GRID_SIZE - 12))
        
        # Draw snake
        for i, segment in enumerate(self.snake):
            rect = pygame.Rect(segment.x * GRID_SIZE, segment.y * GRID_SIZE, 
                             GRID_SIZE, GRID_SIZE)
            
            if i == 0:  # Head
                pygame.draw.rect(self.screen, BRIGHT_GREEN, rect)
                pygame.draw.rect(self.screen, LIME, rect, 2)
            else:  # Body
                pygame.draw.rect(self.screen, DARK_GREEN, rect)
                pygame.draw.rect(self.screen, LIME, rect, 1)
        
        # Draw status bar
        self.draw_status_bar()
        
        pygame.display.flip()
    
    def draw_status_bar(self):
        """Draw status information"""
        status_y = GRID_HEIGHT * GRID_SIZE + 10
        
        # Background
        pygame.draw.rect(self.screen, DARK_GRAY, 
                        (0, GRID_HEIGHT * GRID_SIZE, WINDOW_WIDTH, 100))
        
        # Title
        title = self.title_font.render("InstallSnake Monitor", True, BRIGHT_GREEN)
        self.screen.blit(title, (10, status_y))
        
        # Stats
        stats = [
            f"Packages: {self.completed_packages}/{self.total_packages}",
            f"Failed: {self.failed_packages}",
            f"Score: {self.score}",
            f"Length: {len(self.snake)}"
        ]
        
        x_offset = 350
        for stat in stats:
            stat_text = self.font.render(stat, True, WHITE)
            self.screen.blit(stat_text, (x_offset, status_y + 5))
            x_offset += 180
        
        # Status messages
        msg_y = status_y + 35
        for i, msg in enumerate(list(self.status_messages)[-3:]):
            msg_text = self.small_font.render(msg, True, CYAN)
            self.screen.blit(msg_text, (10, msg_y + i * 18))
        
        # Completion message
        if self.completion_message:
            complete_text = self.font.render(self.completion_message, True, YELLOW)
            self.screen.blit(complete_text, (WINDOW_WIDTH - 350, status_y + 5))
        
        # Victory message when all collected
        remaining = len([p for p in self.pellets 
                        if p.state in [PelletState.READY, PelletState.BUILDING, PelletState.DOWNLOADING, PelletState.QUEUED]])
        if self.finished and remaining == 0:
            victory_text = self.title_font.render("VICTORY!", True, BRIGHT_GREEN)
            victory_rect = victory_text.get_rect(center=(WINDOW_WIDTH // 2, 30))
            self.screen.blit(victory_text, victory_rect)
    
    def run(self):
        """Main game loop"""
        while self.running and not self.finished:
            if not self.handle_input():
                break
            
            self.update()
            self.draw()
            self.clock.tick(FPS)
        
        # Keep running until all READY pellets are collected
        if self.finished:
            # Count remaining collectible pellets
            remaining = [p for p in self.pellets 
                        if p.state in [PelletState.READY, PelletState.BUILDING, PelletState.DOWNLOADING, PelletState.QUEUED]]
            
            timeout = time.time() + 10  # Max 10 seconds to collect
            
            while remaining and time.time() < timeout and self.running:
                if not self.handle_input():
                    break
                
                self.update()
                self.draw()
                self.clock.tick(FPS)
                
                # Recount remaining
                remaining = [p for p in self.pellets 
                            if p.state in [PelletState.READY, PelletState.BUILDING, PelletState.DOWNLOADING, PelletState.QUEUED]]
            
            # Final victory screen for 3 seconds
            for _ in range(3 * FPS):
                if not self.handle_input():
                    break
                self.draw()
                self.clock.tick(FPS)
        
        pygame.quit()


def main():
    """Test mode - demonstrate with simulated events"""
    event_queue = queue.Queue()
    
    # Simulate package installations
    packages = ["numpy", "pandas", "matplotlib", "requests", "flask"]
    
    def simulate_installs():
        time.sleep(1)
        for pkg in packages:
            event_queue.put(InstallEvent("package_queued", name=pkg))
            time.sleep(0.5)
        
        for pkg in packages:
            event_queue.put(InstallEvent("download_started", name=pkg, size_mb=random.uniform(1, 10)))
            
            for i in range(10):
                time.sleep(0.3)
                event_queue.put(InstallEvent("download_progress", name=pkg, progress=i/10))
            
            event_queue.put(InstallEvent("build_started", name=pkg))
            
            for i in range(10):
                time.sleep(0.2)
                event_queue.put(InstallEvent("build_progress", name=pkg, progress=i/10))
            
            if random.random() < 0.1:
                event_queue.put(InstallEvent("install_failed", name=pkg, error="Build error"))
            else:
                event_queue.put(InstallEvent("install_complete", name=pkg))
        
        event_queue.put(InstallEvent("all_complete"))
    
    # Start simulation thread
    sim_thread = threading.Thread(target=simulate_installs, daemon=True)
    sim_thread.start()
    
    # Run monitor
    monitor = SnakeMonitor(event_queue)
    monitor.run()


if __name__ == "__main__":
    main()
