use crate::installsnake::{InstallSnake, SnakeConfig, Theme, InstallEvent};
use anyhow::Result;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};
use console::style;

/// High-level game runner that manages the game loop and subprocess integration
pub struct GameRunner {
    game: InstallSnake,
    config: SnakeConfig,
    fps: u32,
}

impl GameRunner {
    pub fn new(config: SnakeConfig) -> Self {
        let game = InstallSnake::new(config.clone());
        let fps = config.fps;
        Self { game, config, fps }
    }

    /// Run game with simulated events for demo/testing
    pub fn run_demo(&mut self, duration_secs: u64) -> Result<()> {
        println!("{}", style("🐍 InstallSnake - Demo Mode").cyan().bold());
        println!("{}", style("Spawning mock packages...").dim());

        let packages = vec!["numpy", "pandas", "scikit-learn", "tensorflow", "pytorch"];
        for pkg in &packages {
            self.game.handle_event(InstallEvent::PackageQueued(pkg.to_string()))?;
        }

        let start = Instant::now();
        let frame_time = Duration::from_millis(1000 / (self.fps as u64).max(1));
        let mut frame: u32 = 0;

        loop {
            self.game.update();
            self.render_frame()?;

            // Simulate install events
            self.simulate_events(&packages, frame)?;

            frame += 1;
            std::thread::sleep(frame_time);

            if start.elapsed() > Duration::from_secs(duration_secs) {
                break;
            }
        }

        self.print_summary()?;
        Ok(())
    }

    /// Run game with real pip subprocess events
    pub fn run_with_subprocess(&mut self, event_rx: Receiver<InstallEvent>, timeout_secs: u64) -> Result<()> {
        println!("{}", style("🐍 InstallSnake - Live Install").cyan().bold());

        let start = Instant::now();
        let frame_time = Duration::from_millis(1000 / (self.fps as u64).max(1));
        let mut last_frame = Instant::now();

        loop {
            // Process all pending events from subprocess
            while let Ok(event) = event_rx.try_recv() {
                self.game.handle_event(event)?;
            }

            // Update and render at fixed framerate
            if last_frame.elapsed() >= frame_time {
                self.game.update();
                self.render_frame()?;
                last_frame = Instant::now();
            }

            if !self.game.is_running() && start.elapsed() > Duration::from_secs(5) {
                break;
            }

            if start.elapsed() > Duration::from_secs(timeout_secs) {
                println!("{}", style("⏱ Timeout reached").yellow());
                break;
            }

            std::thread::sleep(Duration::from_millis(10));
        }

        self.print_summary()?;
        Ok(())
    }

    fn simulate_events(&mut self, packages: &[&str], frame: u32) -> Result<()> {
        for (idx, pkg) in packages.iter().enumerate() {
            let pkg_start = (idx as u32) * 30;  // Stagger package starts
            let pkg_frame = if frame > pkg_start { frame - pkg_start } else { 0 };

            // Phase 1: Download (frames 5-25)
            if pkg_frame >= 5 && pkg_frame < 25 {
                if pkg_frame == 5 {
                    self.game.handle_event(InstallEvent::DownloadStarted {
                        name: pkg.to_string(),
                        total_bytes: Some(5_000_000),
                    })?;
                } else if pkg_frame % 3 == 0 {
                    let progress = ((pkg_frame - 5) as f32) / 20.0;
                    let current = (progress * 5_000_000.0) as u64;
                    self.game.handle_event(InstallEvent::DownloadProgress {
                        name: pkg.to_string(),
                        current,
                        total: 5_000_000,
                    })?;
                }
            }

            // Phase 2: Build (frames 25-45)
            if pkg_frame >= 25 && pkg_frame < 45 {
                if pkg_frame == 25 {
                    self.game.handle_event(InstallEvent::BuildStarted(pkg.to_string()))?
                } else if pkg_frame % 4 == 0 {
                    let progress = ((pkg_frame - 25) as f32) / 20.0;
                    self.game.handle_event(InstallEvent::BuildProgress {
                        name: pkg.to_string(),
                        pct: progress.min(1.0),
                    })?
                }
            }

            // Phase 3: Install complete (frame 45)
            if pkg_frame == 45 {
                self.game.handle_event(InstallEvent::InstallComplete(pkg.to_string()))?
            }
        }

        Ok(())
    }

    fn render_frame(&mut self) -> Result<()> {
        let output = self.game.render()?;
        print!("{}", output);
        std::io::Write::flush(&mut std::io::stdout())?;
        Ok(())
    }

    fn print_summary(&self) -> Result<()> {
        let (successes, crashes, total) = self.game.get_stats();
        println!();
        println!("{}", style("═".repeat(50)).dim());
        println!("{}", style(format!("🐍 Game Over!")).cyan().bold());
        println!("{}", style(format!("Packages Completed: {}/{}", successes, total)).green());
        println!("{}", style(format!("Build Failures: {}", crashes)).yellow());
        println!("{}", style("═".repeat(50)).dim());
        Ok(())
    }
}

/// Parse pip output into game events
pub fn parse_pip_output(line: &str) -> Option<InstallEvent> {
    // Match pip's typical output patterns
    if line.contains("Collecting") {
        if let Some(pkg_name) = line.split_whitespace().nth(1) {
            return Some(InstallEvent::PackageQueued(pkg_name.trim_matches(',').to_string()));
        }
    }

    if line.contains("Downloading") {
        // "Downloading package-1.0.0-py3-none-any.whl (123kB)"
        if let Some(start) = line.find("Downloading ") {
            if let Some(end) = line[start..].find(' ') {
                let pkg_name = &line[start + 12..start + end];
                return Some(InstallEvent::DownloadStarted {
                    name: pkg_name.to_string(),
                    total_bytes: extract_size(line),
                });
            }
        }
    }

    if line.contains("Building wheel for") {
        if let Some(start) = line.find("for ") {
            if let Some(end) = line[start..].find(' ') {
                let pkg_name = &line[start + 4..start + end];
                return Some(InstallEvent::BuildStarted(pkg_name.to_string()));
            }
        }
    }

    if line.contains("Successfully built") {
        for pkg in line.split_whitespace().skip(2) {
            return Some(InstallEvent::InstallComplete(pkg.to_string()));
        }
    }

    if line.contains("ERROR:") || line.contains("error:") {
        return Some(InstallEvent::Error(line.to_string()));
    }

    if line.contains("Failed building wheel for") {
        if let Some(start) = line.find("for ") {
            if let Some(end) = line[start..].find(' ') {
                let pkg_name = &line[start + 4..start + end];
                return Some(InstallEvent::BuildFailed {
                    name: pkg_name.to_string(),
                    error: "Build failed".to_string(),
                });
            }
        }
    }

    None
}

/// Extract file size from pip output (e.g., "123kB", "1.5MB")
fn extract_size(line: &str) -> Option<u64> {
    if let Some(start) = line.rfind('(') {
        if let Some(end) = line[start..].find(')') {
            let size_str = &line[start + 1..start + end].trim();
            let (num_part, unit) = if size_str.ends_with("MB") {
                (&size_str[..size_str.len() - 2], 1_000_000u64)
            } else if size_str.ends_with("kB") {
                (&size_str[..size_str.len() - 2], 1_000u64)
            } else if size_str.ends_with("B") {
                (&size_str[..size_str.len() - 1], 1u64)
            } else {
                return None;
            };

            if let Ok(num) = num_part.parse::<f64>() {
                return Some((num * unit as f64) as u64);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_collecting() {
        let line = "Collecting numpy==1.21.0";
        if let Some(InstallEvent::PackageQueued(name)) = parse_pip_output(line) {
            assert_eq!(name, "numpy==1.21.0");
        }
    }

    #[test]
    fn test_parse_downloading() {
        let line = "Downloading numpy-1.21.0-cp39-cp39-linux_x86_64.whl (14.6MB)";
        if let Some(event) = parse_pip_output(line) {
            match event {
                InstallEvent::DownloadStarted { name, total_bytes } => {
                    assert_eq!(name, "numpy-1.21.0-cp39-cp39-linux_x86_64.whl");
                    assert!(total_bytes.is_some());
                }
                _ => panic!("Wrong event type"),
            }
        }
    }

    #[test]
    fn test_extract_size() {
        assert_eq!(extract_size("file.whl (14.6MB)"), Some(14_600_000));
        assert_eq!(extract_size("file.whl (123kB)"), Some(123_000));
    }
}
