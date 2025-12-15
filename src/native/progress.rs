// Native alternative to `indicatif` crate
// Progress bars with zero dependencies
// Savings: -150 KB, zero external deps

use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Progress bar
pub struct ProgressBar {
    total: u64,
    current: u64,
    message: String,
    start_time: Instant,
    last_draw: Instant,
    width: usize,
    is_spinner: bool,
    spinner_frame: usize,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(total: u64) -> Self {
        let now = Instant::now();
        Self {
            total,
            current: 0,
            message: String::new(),
            start_time: now,
            last_draw: now,
            width: 40,
            is_spinner: false,
            spinner_frame: 0,
        }
    }

    /// Create a new spinner
    pub fn new_spinner() -> Self {
        let now = Instant::now();
        Self {
            total: 0,
            current: 0,
            message: String::new(),
            start_time: now,
            last_draw: now,
            width: 40,
            is_spinner: true,
            spinner_frame: 0,
        }
    }
    
    /// Set current position
    pub fn set_position(&mut self, pos: u64) {
        self.current = pos;
        self.draw();
    }
    
    /// Increment position
    pub fn inc(&mut self, delta: u64) {
        self.current += delta;
        self.draw();
    }
    
    /// Set message
    pub fn set_message(&mut self, msg: String) {
        self.message = msg;
        self.draw();
    }
    
    /// Finish and clear
    pub fn finish(&self) {
        print!("\r{}\r", " ".repeat(80));
        io::stdout().flush().ok();
    }
    
    /// Finish with message
    pub fn finish_with_message(&self, msg: &str) {
        print!("\r{}\r", " ".repeat(80));
        println!("{}", msg);
    }
    
    /// Draw the progress bar
    fn draw(&mut self) {
        // Rate limit drawing to avoid flickering
        if self.last_draw.elapsed() < Duration::from_millis(100) {
            return;
        }
        self.last_draw = Instant::now();

        if self.is_spinner {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            print!("\r{} {}", frames[self.spinner_frame], self.message);
            self.spinner_frame = (self.spinner_frame + 1) % frames.len();
        } else {
            let percent = if self.total > 0 {
                (self.current as f64 / self.total as f64 * 100.0) as u64
            } else {
                0
            };
            
            let filled = (self.width as f64 * self.current as f64 / self.total as f64) as usize;
            let empty = self.width.saturating_sub(filled);
            
            let bar = format!(
                "{}{}",
                "█".repeat(filled),
                "░".repeat(empty)
            );
            
            let elapsed = self.start_time.elapsed();
            let speed = if elapsed.as_secs() > 0 {
                self.current / elapsed.as_secs()
            } else {
                0
            };
            
            let eta = if speed > 0 && self.current < self.total {
                let remaining = self.total - self.current;
                Duration::from_secs(remaining / speed)
            } else {
                Duration::from_secs(0)
            };
            
            print!(
                "\r[{}] {}% {}/{} {} ETA: {}s",
                bar,
                percent,
                format_bytes(self.current),
                format_bytes(self.total),
                self.message,
                eta.as_secs()
            );
        }
        
        io::stdout().flush().ok();
    }
}

/// Spinner for indeterminate progress
pub struct Spinner {
    message: String,
    frames: Vec<&'static str>,
    current_frame: usize,
    last_draw: Instant,
}

impl Spinner {
    /// Create a new spinner
    pub fn new() -> Self {
        Self {
            message: String::new(),
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
            last_draw: Instant::now(),
        }
    }
    
    /// Set message
    pub fn set_message(&mut self, msg: String) {
        self.message = msg;
        self.tick();
    }
    
    /// Advance spinner
    pub fn tick(&mut self) {
        if self.last_draw.elapsed() < Duration::from_millis(80) {
            return;
        }
        self.last_draw = Instant::now();
        
        print!("\r{} {}", self.frames[self.current_frame], self.message);
        io::stdout().flush().ok();
        
        self.current_frame = (self.current_frame + 1) % self.frames.len();
    }
    
    /// Finish spinner
    pub fn finish(&self) {
        print!("\r{}\r", " ".repeat(80));
        io::stdout().flush().ok();
    }
    
    /// Finish with message
    pub fn finish_with_message(&self, msg: &str) {
        print!("\r{}\r", " ".repeat(80));
        println!("{}", msg);
    }
}

/// Multi-progress bar (simplified)
pub struct MultiProgress {
    bars: Vec<ProgressBar>,
}

impl MultiProgress {
    pub fn new() -> Self {
        Self { bars: Vec::new() }
    }
    
    pub fn add(&mut self, pb: ProgressBar) {
        self.bars.push(pb);
    }
    
    pub fn clear(&self) {
        print!("\r{}\r", " ".repeat(80));
        io::stdout().flush().ok();
    }
}

/// Format bytes as human-readable
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        let mut pb = ProgressBar::new(100);
        pb.set_position(50);
        assert_eq!(pb.current, 50);
        pb.finish();
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_spinner() {
        let mut spinner = Spinner::new();
        spinner.set_message("Testing".to_string());
        spinner.tick();
        spinner.finish();
    }
}
