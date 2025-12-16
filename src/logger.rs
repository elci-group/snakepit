use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, Duration};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use snakegg::native::style::{dim, green, yellow};

#[derive(Debug)]
pub struct GitLogger {
    log_dir: PathBuf,
    remote_url: Option<String>,
    last_push: SystemTime,
    push_interval: Duration,
}

impl GitLogger {
    pub fn new(log_dir: PathBuf, remote_url: Option<String>) -> Self {
        // Ensure dir exists
        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir).unwrap_or_default();
        }
        
        // Init git if needed
        if !log_dir.join(".git").exists() {
            let _ = Command::new("git").arg("init").current_dir(&log_dir).output();
            // Configure user if needed? Git usually requires it.
            // We'll assume the user's global config works, or we can set local config.
            let _ = Command::new("git").args(&["config", "user.name", "Snakepit Logger"]).current_dir(&log_dir).output();
            let _ = Command::new("git").args(&["config", "user.email", "logger@snakepit.local"]).current_dir(&log_dir).output();
        }
        
        // Configure remote if provided
        if let Some(url) = &remote_url {
            // Check if remote exists
            let remotes = Command::new("git").args(&["remote"]).current_dir(&log_dir).output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();
                
            if remotes.contains("origin") {
                let _ = Command::new("git")
                    .args(&["remote", "set-url", "origin", url])
                    .current_dir(&log_dir)
                    .output();
            } else {
                let _ = Command::new("git")
                    .args(&["remote", "add", "origin", url])
                    .current_dir(&log_dir)
                    .output();
            }
        }

        Self {
            log_dir,
            remote_url,
            last_push: SystemTime::now(),
            push_interval: Duration::from_secs(300), // 5 minutes default
        }
    }

    pub async fn log(&self, message: &str) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let log_file = self.log_dir.join("snakepit.log");
        let line = format!("[{}] {}\n", timestamp, message);
        
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .await?;
        file.write_all(line.as_bytes()).await?;
        
        Ok(())
    }

    pub async fn sync(&mut self) -> Result<()> {
        if self.remote_url.is_none() {
            return Ok(());
        }

        if self.last_push.elapsed().unwrap_or_default() < self.push_interval {
            return Ok(());
        }

        // Git Add
        Command::new("git").args(&["add", "."]).current_dir(&self.log_dir).output()?;
        
        // Git Commit
        let msg = format!("Log update: {:?}", SystemTime::now());
        Command::new("git").args(&["commit", "-m", &msg]).current_dir(&self.log_dir).output()?;
        
        // Git Push
        // Note: This might block if auth is needed and not configured (ssh keys/credential helper)
        // We run it and hope for the best, maybe with a timeout?
        // For now, simple spawn.
        let output = Command::new("git")
            .args(&["push", "origin", "master"]) // or main
            .current_dir(&self.log_dir)
            .output()?;
        
        if output.status.success() {
            println!("{}", dim("📋 Logs pushed to remote repo"));
        } else {
            // If master fails, try main
             let _ = Command::new("git")
                .args(&["push", "origin", "main"])
                .current_dir(&self.log_dir)
                .output();
        }

        self.last_push = SystemTime::now();
        Ok(())
    }
}
