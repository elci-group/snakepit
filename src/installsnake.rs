use std::collections::{VecDeque, HashMap};
use std::time::Instant;
use anyhow::Result;

/// Game configuration and theme settings
#[derive(Clone, Debug)]
pub struct SnakeConfig {
    pub width: usize,
    pub height: usize,
    pub fps: u32,
    pub theme: Theme,
    pub sound: bool,
    pub show_debug: bool,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Theme {
    RetroGreen,
    AmberTerminal,
    Matrix,
    MonochromeMinimal,
    ErrorPunk,
}

impl Theme {
    pub fn head_color(&self) -> &'static str {
        match self {
            Theme::RetroGreen => "\x1b[92m",      // bright green
            Theme::AmberTerminal => "\x1b[33m",   // yellow/amber
            Theme::Matrix => "\x1b[32m",          // green
            Theme::MonochromeMinimal => "\x1b[37m", // white
            Theme::ErrorPunk => "\x1b[91m",       // bright red
        }
    }

    pub fn body_color(&self) -> &'static str {
        match self {
            Theme::RetroGreen => "\x1b[32m",      // green
            Theme::AmberTerminal => "\x1b[33m",   // amber
            Theme::Matrix => "\x1b[32m",          // green
            Theme::MonochromeMinimal => "\x1b[37m", // white
            Theme::ErrorPunk => "\x1b[31m",       // red
        }
    }

    pub fn pellet_char(&self) -> char {
        match self {
            Theme::RetroGreen => '●',
            Theme::AmberTerminal => '◆',
            Theme::Matrix => '●',
            Theme::MonochromeMinimal => '*',
            Theme::ErrorPunk => '◆',
        }
    }

    pub fn body_char(&self) -> char {
        '█'
    }
}

impl Default for SnakeConfig {
    fn default() -> Self {
        Self {
            width: 60,
            height: 15,
            fps: 12,
            theme: Theme::RetroGreen,
            sound: false,
            show_debug: false,
        }
    }
}

/// Events from pip/python subprocess that drive game updates
#[derive(Clone, Debug)]
pub enum InstallEvent {
    PackageQueued(String),
    DownloadStarted { name: String, total_bytes: Option<u64> },
    DownloadProgress { name: String, current: u64, total: u64 },
    BuildStarted(String),
    BuildProgress { name: String, pct: f32 },
    BuildFailed { name: String, error: String },
    InstallComplete(String),
    AllDone { succeeded: u32, failed: u32 },
    Error(String),
}

/// 2D position on game board
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// Direction the snake moves
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn apply(&self, pos: Position, width: usize, height: usize) -> Option<Position> {
        let (new_x, new_y) = match self {
            Direction::Up => (pos.x, pos.y.saturating_sub(1)),
            Direction::Down => (pos.x, (pos.y + 1).min(height - 1)),
            Direction::Left => (pos.x.saturating_sub(1), pos.y),
            Direction::Right => ((pos.x + 1).min(width - 1), pos.y),
        };

        if new_x < width && new_y < height {
            Some(Position { x: new_x, y: new_y })
        } else {
            None
        }
    }

    pub fn symbol(&self) -> char {
        match self {
            Direction::Up => '▲',
            Direction::Down => '▼',
            Direction::Left => '◄',
            Direction::Right => '►',
        }
    }
}

/// A target package pellet with download/build metadata
#[derive(Clone, Debug)]
pub struct Pellet {
    pub pos: Position,
    pub package_name: String,
    pub state: PelletState,
    pub progress: f32, // 0.0 to 1.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PelletState {
    Queued,
    Downloading,
    Building,
    Ready,
    Eaten,
    Failed,
}

/// Persistent render buffer to minimize terminal writes
#[derive(Clone)]
struct FrameBuffer {
    width: usize,
    height: usize,
    chars: Vec<char>,
    colors: Vec<u32>, // simple color codes
}

impl FrameBuffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            chars: vec![' '; width * height],
            colors: vec![0; width * height],
        }
    }

    fn set(&mut self, pos: Position, ch: char, color: u32) {
        if pos.x < self.width && pos.y < self.height {
            let idx = pos.y * self.width + pos.x;
            self.chars[idx] = ch;
            self.colors[idx] = color;
        }
    }

    fn get(&self, pos: Position) -> (char, u32) {
        if pos.x < self.width && pos.y < self.height {
            let idx = pos.y * self.width + pos.x;
            (self.chars[idx], self.colors[idx])
        } else {
            (' ', 0)
        }
    }

    fn clear(&mut self) {
        self.chars.fill(' ');
        self.colors.fill(0);
    }

    /// Generate delta updates (only changed cells)
    fn diff(&self, other: &FrameBuffer, width: usize, height: usize) -> Vec<(Position, char)> {
        let mut changes = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let pos = Position { x, y };
                let idx = y * width + x;
                if idx < self.chars.len() && idx < other.chars.len() {
                    if self.chars[idx] != other.chars[idx] {
                        changes.push((pos, self.chars[idx]));
                    }
                }
            }
        }
        changes
    }
}

/// Main game engine state
pub struct InstallSnake {
    config: SnakeConfig,
    snake: VecDeque<Position>,
    direction: Direction,
    next_direction: Direction,
    pellets: Vec<Pellet>,
    board: FrameBuffer,
    last_frame: FrameBuffer,
    game_state: GameState,
    frame_count: u64,
    last_update: Instant,
    crashes: u32,
    successes: u32,
    queued_packages: HashMap<String, u32>,
    crash_animation_frames: u32,
    rng_state: u64,
    ai_path: Vec<Direction>,
    ai_recalc_counter: u32,
    obstacles: Vec<Position>,  // Wall positions for maze-like gameplay
    speed_boost: u32,          // Frames left for speed boost
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Initializing,
    Playing,
    Error { frames_remaining: u32 },
    Won,
    Lost,
}

impl InstallSnake {
    pub fn new(config: SnakeConfig) -> Self {
        let width = config.width;
        let height = config.height;
        
        // Start snake in center
        let center_x = width / 2;
        let center_y = height / 2;
        let mut snake = VecDeque::new();
        snake.push_back(Position { x: center_x, y: center_y });

        let mut instance = Self {
            config,
            snake,
            direction: Direction::Right,
            next_direction: Direction::Right,
            pellets: Vec::new(),
            board: FrameBuffer::new(width, height),
            last_frame: FrameBuffer::new(width, height),
            game_state: GameState::Playing,
            frame_count: 0,
            last_update: Instant::now(),
            crashes: 0,
            successes: 0,
            queued_packages: HashMap::new(),
            crash_animation_frames: 0,
            rng_state: 12345,
            ai_path: Vec::new(),
            ai_recalc_counter: 0,
            obstacles: Vec::new(),
            speed_boost: 0,
        };
        instance.generate_obstacles();
        instance
    }

    /// Process an install event and update game state
    pub fn handle_event(&mut self, event: InstallEvent) -> Result<()> {
        match event {
            InstallEvent::PackageQueued(name) => {
                self.queued_packages.insert(name.clone(), 0);
                self.spawn_pellet(&name);
            }
            InstallEvent::DownloadStarted { name, total_bytes: _ } => {
                self.update_pellet_state(&name, PelletState::Downloading);
            }
            InstallEvent::DownloadProgress { name, current, total } => {
                if total > 0 {
                    let pct = current as f32 / total as f32;
                    self.update_pellet_progress(&name, PelletState::Downloading, pct);
                }
            }
            InstallEvent::BuildStarted(name) => {
                self.update_pellet_state(&name, PelletState::Building);
            }
            InstallEvent::BuildProgress { name, pct } => {
                self.update_pellet_progress(&name, PelletState::Building, pct);
            }
            InstallEvent::BuildFailed { name, error: _ } => {
                self.update_pellet_state(&name, PelletState::Failed);
                self.crashes += 1;
                self.trigger_crash_animation();
            }
            InstallEvent::InstallComplete(name) => {
                self.eat_pellet(&name);
                self.successes += 1;
                self.grow_snake(2);
            }
            InstallEvent::AllDone { succeeded: _, failed: _ } => {
                if self.pellets.iter().all(|p| p.state == PelletState::Eaten || p.state == PelletState::Failed) {
                    self.game_state = GameState::Won;
                }
            }
            InstallEvent::Error(msg) => {
                if self.config.show_debug {
                    eprintln!("Game error: {}", msg);
                }
                self.crashes += 1;
                self.trigger_crash_animation();
            }
        }
        Ok(())
    }

    fn spawn_pellet(&mut self, name: &str) {
        let pos = self.random_free_position();
        self.pellets.push(Pellet {
            pos,
            package_name: name.to_string(),
            state: PelletState::Queued,
            progress: 0.0,
        });
    }

    fn update_pellet_state(&mut self, name: &str, state: PelletState) {
        if let Some(pellet) = self.pellets.iter_mut().find(|p| p.package_name == name) {
            pellet.state = state;
        }
    }

    fn update_pellet_progress(&mut self, name: &str, state: PelletState, progress: f32) {
        if let Some(pellet) = self.pellets.iter_mut().find(|p| p.package_name == name) {
            pellet.state = state;
            pellet.progress = progress.clamp(0.0, 1.0);
        }
    }

    fn eat_pellet(&mut self, name: &str) {
        if let Some(pellet) = self.pellets.iter_mut().find(|p| p.package_name == name) {
            pellet.state = PelletState::Eaten;
        }
    }

    fn grow_snake(&mut self, segments: usize) {
        for _ in 0..segments {
            if let Some(tail) = self.snake.back().copied() {
                self.snake.push_back(tail);
            }
        }
    }

    fn random_free_position(&mut self) -> Position {
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let x = (self.rng_state as usize) % self.config.width;
        let y = ((self.rng_state >> 16) as usize) % self.config.height;
        Position { x, y }
    }

    fn trigger_crash_animation(&mut self) {
        self.crash_animation_frames = 6; // 6 frames of red flash
        self.game_state = GameState::Error {
            frames_remaining: 6,
        };
    }

    /// Generate maze-like obstacles for challenging gameplay
    fn generate_obstacles(&mut self) {
        // Create a few vertical and horizontal barriers for maze-like feel
        let width = self.config.width as i32;
        let height = self.config.height as i32;
        
        // Vertical obstacle lines (every 15-20 chars)
        if width > 30 {
            for x in [width / 4, width * 3 / 4] {
                for y in 2..(height - 2) {
                    if self.random_bool(0.4) {
                        self.obstacles.push(Position { x: x as usize, y: y as usize });
                    }
                }
            }
        }
        
        // Horizontal obstacle lines
        if height > 12 {
            for y in [height / 3, height * 2 / 3] {
                for x in 3..(width - 3) {
                    if self.random_bool(0.3) {
                        self.obstacles.push(Position { x: x as usize, y: y as usize });
                    }
                }
            }
        }
    }

    /// Pseudo-random boolean with probability
    fn random_bool(&mut self, probability: f32) -> bool {
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        ((self.rng_state as f32 / u64::MAX as f32).abs()) < probability
    }

    /// Check if position is blocked (wall or snake body)
    fn is_blocked(&self, pos: Position) -> bool {
        // Check obstacles
        if self.obstacles.contains(&pos) {
            return true;
        }
        
        // Check snake body (except head, which can't collide with tail)
        for (i, seg) in self.snake.iter().enumerate() {
            if i > 0 && *seg == pos {
                return true;
            }
        }
        
        // Check board boundaries
        pos.x == 0 || pos.x >= self.config.width - 1 || pos.y == 0 || pos.y >= self.config.height - 1
    }

    /// BFS pathfinding for intelligent AI
    fn calculate_ai_path_bfs(&mut self) {
        if self.pellets.is_empty() {
            return;
        }

        let head = match self.snake.front() {
            Some(h) => *h,
            None => return,
        };

        // Find nearest uneaten pellet
        let target = self.pellets
            .iter()
            .filter(|p| p.state != PelletState::Eaten && p.state != PelletState::Failed)
            .min_by_key(|p| {
                let dx = (head.x as i32 - p.pos.x as i32).abs();
                let dy = (head.y as i32 - p.pos.y as i32).abs();
                dx + dy
            });

        if let Some(target_pellet) = target {
            let target_pos = target_pellet.pos;
            
            // Simple A* style movement (towards target while avoiding obstacles)
            self.ai_path.clear();
            let mut current = head;
            let mut attempts = 0;
            let max_attempts = 50;

            while current != target_pos && attempts < max_attempts {
                let mut best_move = Direction::Right;
                let mut best_distance = i32::MAX;

                // Try all four directions, pick one that gets closest to target and isn't blocked
                for &dir in &[Direction::Right, Direction::Left, Direction::Down, Direction::Up] {
                    if let Some(next_pos) = dir.apply(current, self.config.width, self.config.height) {
                        if !self.is_blocked(next_pos) {
                            let dx = (target_pos.x as i32 - next_pos.x as i32).abs();
                            let dy = (target_pos.y as i32 - next_pos.y as i32).abs();
                            let distance = dx + dy;

                            if distance < best_distance {
                                best_distance = distance;
                                best_move = dir;
                            }
                        }
                    }
                }

                if let Some(next_pos) = best_move.apply(current, self.config.width, self.config.height) {
                    if !self.is_blocked(next_pos) {
                        self.ai_path.push(best_move);
                        current = next_pos;
                    }
                }
                attempts += 1;
            }
        }
    }

    /// Calculate AI path to nearest pellet (delegates to BFS)
    fn calculate_ai_path(&mut self) {
        self.calculate_ai_path_bfs();
    }

    /// Update game logic (called once per frame)
    pub fn update(&mut self) {
        self.frame_count += 1;

        // AI autonomously navigates snake
        self.ai_recalc_counter += 1;
        if self.ai_recalc_counter > 10 {
            // Recalculate path every 10 frames
            self.calculate_ai_path();
            self.ai_recalc_counter = 0;
        }

        // Follow AI path
        if !self.ai_path.is_empty() {
            self.direction = self.ai_path.remove(0);
        } else {
            // If no path, move forward
            self.direction = self.direction;
        }

        self.next_direction = self.direction;

        // Move snake
        if let Some(head) = self.snake.front().copied() {
            if let Some(new_head) = self.direction.apply(head, self.config.width, self.config.height) {
                // Check collision with obstacles or boundaries
                let hits_wall = new_head.x == 0 || new_head.x >= self.config.width - 1 
                             || new_head.y == 0 || new_head.y >= self.config.height - 1;
                let hits_obstacle = self.obstacles.contains(&new_head);
                let hits_self = self.snake.iter().skip(1).any(|&seg| seg == new_head);

                if hits_wall || hits_obstacle || hits_self {
                    // Collision detected - trigger crash
                    self.crashes += 1;
                    self.trigger_crash_animation();
                    // Respawn snake shorter
                    while self.snake.len() > 3 {
                        self.snake.pop_back();
                    }
                } else {
                    // Safe to move
                    self.snake.push_front(new_head);
                    if self.snake.len() > 1 {
                        self.snake.pop_back();
                    }

                    // Check pellet collision
                    let mut eaten_idx = None;
                    for (i, pellet) in self.pellets.iter().enumerate() {
                        if pellet.pos == new_head && pellet.state != PelletState::Eaten {
                            eaten_idx = Some(i);
                            break;
                        }
                    }
                    if let Some(i) = eaten_idx {
                        self.pellets[i].state = PelletState::Eaten;
                        self.successes += 1;
                        self.grow_snake(3);
                        // Speed boost after eating pellet
                        self.speed_boost = 5;
                    }
                }
            }
        }

        // Update crash animation
        match self.game_state {
            GameState::Error { frames_remaining } => {
                if frames_remaining > 0 {
                    self.game_state = GameState::Error {
                        frames_remaining: frames_remaining - 1,
                    };
                } else {
                    self.game_state = GameState::Playing;
                }
            }
            _ => {}
        }
    }

    /// Render frame line-by-line (smooth, efficient, no flicker)
    pub fn render(&mut self) -> Result<String> {
        let mut new_board = FrameBuffer::new(self.config.width, self.config.height);

        // Draw border
        for x in 0..self.config.width {
            new_board.set(Position { x, y: 0 }, '─', 1);
            new_board.set(Position { x, y: self.config.height - 1 }, '─', 1);
        }
        for y in 0..self.config.height {
            new_board.set(Position { x: 0, y }, '│', 1);
            new_board.set(Position { x: self.config.width - 1, y }, '│', 1);
        }
        new_board.set(Position { x: 0, y: 0 }, '┌', 1);
        new_board.set(Position { x: self.config.width - 1, y: 0 }, '┐', 1);
        new_board.set(Position { x: 0, y: self.config.height - 1 }, '└', 1);
        new_board.set(Position { x: self.config.width - 1, y: self.config.height - 1 }, '┘', 1);

        // Draw obstacles
        for obs_pos in &self.obstacles {
            new_board.set(*obs_pos, '▓', 4);  // Dark block
        }

        // Draw pellets
        for pellet in &self.pellets {
            if pellet.state != PelletState::Eaten {
                let ch = match pellet.state {
                    PelletState::Failed => '✗',
                    _ => self.config.theme.pellet_char(),
                };
                new_board.set(pellet.pos, ch, 2);
            }
        }

        // Draw snake
        for (i, pos) in self.snake.iter().enumerate() {
            let is_head = i == 0;
            let ch = if is_head {
                self.direction.symbol()
            } else {
                self.config.theme.body_char()
            };
            let color = if is_head { 10 } else { 3 };
            new_board.set(*pos, ch, color);
        }

        let mut output = String::new();

        if self.frame_count == 1 {
            // First frame: full board redraw
            output.push_str("\x1b[?25l"); // Hide cursor
            output.push_str("\x1b[2J\x1b[H"); // Clear + home
            for y in 0..self.config.height {
                for x in 0..self.config.width {
                    let (ch, _) = new_board.get(Position { x, y });
                    output.push(ch);
                }
                output.push('\n');
            }
        } else {
            // Subsequent frames: update only changed lines
            for y in 0..self.config.height {
                let mut line_changed = false;
                for x in 0..self.config.width {
                    let pos = Position { x, y };
                    let (new_ch, _) = new_board.get(pos);
                    let (old_ch, _) = self.board.get(pos);
                    if new_ch != old_ch {
                        line_changed = true;
                        break;
                    }
                }

                if line_changed {
                    // Redraw entire line if any cell changed
                    output.push_str(&format!("\x1b[{};1H", y + 1));
                    for x in 0..self.config.width {
                        let (ch, _) = new_board.get(Position { x, y });
                        output.push(ch);
                    }
                }
            }
        }

        // Update status lines
        let status_line = self.config.height + 1;
        output.push_str(&format!("\x1b[{};1H", status_line));
        output.push_str(&format!("Packages: {}/{}  ", self.successes, self.pellets.len()));
        
        let line2 = status_line + 1;
        output.push_str(&format!("\x1b[{};1H", line2));
        output.push_str(&format!("Snake: {} | Crashes: {}  ", self.snake.len(), self.crashes));
        
        output.push_str("\x1b[?25h"); // Show cursor

        self.board = new_board;
        Ok(output)
    }

    pub fn is_running(&self) -> bool {
        matches!(self.game_state, GameState::Playing | GameState::Error { .. })
    }

    pub fn get_stats(&self) -> (u32, u32, usize) {
        (self.successes, self.crashes, self.pellets.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_apply() {
        let pos = Position { x: 5, y: 5 };
        let new_pos = Direction::Right.apply(pos, 10, 10).unwrap();
        assert_eq!(new_pos, Position { x: 6, y: 5 });
    }

    #[test]
    fn test_snake_creation() {
        let config = SnakeConfig::default();
        let snake = InstallSnake::new(config);
        assert_eq!(snake.snake.len(), 1);
        assert_eq!(snake.successes, 0);
    }

    #[test]
    fn test_pellet_spawn() {
        let config = SnakeConfig::default();
        let mut game = InstallSnake::new(config);
        game.spawn_pellet("numpy");
        assert_eq!(game.pellets.len(), 1);
        assert_eq!(game.pellets[0].package_name, "numpy");
    }
}
