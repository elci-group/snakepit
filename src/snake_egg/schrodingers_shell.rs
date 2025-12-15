use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;
use crate::snake_egg::embryo::Embryo;

/// Schrödinger's Shells: Quantum superposition between local nest and git ether
/// 
/// Eggs exist simultaneously in two states:
/// - Observable (local nest) - collapsed for active work
/// - Ethereal (git) - quantum superposition in version control
/// 
/// The observer (Mother) collapses the wave function when evolution is needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchrodingersShell {
    pub egg_name: String,
    pub state: QuantumState,
    pub local_path: Option<PathBuf>,
    pub git_commit: Option<String>,
    pub last_observed: Option<u64>,  // timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuantumState {
    /// Exists only in git (wave function intact)
    Ethereal,
    
    /// Materialized in local nest (collapsed)
    Manifested,
    
    /// Superposition - exists in both states
    Superposition,
    
    /// Never committed, purely local
    Uncollapsed,
}

impl SchrodingersShell {
    pub fn new(egg_name: String) -> Self {
        Self {
            egg_name,
            state: QuantumState::Uncollapsed,
            local_path: None,
            git_commit: None,
            last_observed: None,
        }
    }

    /// Collapse wave function: Materialize egg from git to local nest
    pub async fn collapse(&mut self, nest_root: &Path) -> Result<PathBuf> {
        match self.state {
            QuantumState::Ethereal => {
                // Pull from git
                let local_path = self.materialize_from_git(nest_root).await?;
                self.local_path = Some(local_path.clone());
                self.state = QuantumState::Manifested;
                self.update_observation();
                
                println!("👁️  Collapsed {} from ether → {}", 
                    self.egg_name, local_path.display());
                
                Ok(local_path)
            },
            QuantumState::Manifested => {
                // Already local
                self.update_observation();
                Ok(self.local_path.clone().unwrap())
            },
            QuantumState::Superposition => {
                // Sync from git, keep local too
                self.sync_from_git(nest_root).await?;
                self.update_observation();
                Ok(self.local_path.clone().unwrap())
            },
            QuantumState::Uncollapsed => {
                // Create fresh in local
                let local_path = nest_root.join(&self.egg_name);
                fs::create_dir_all(&local_path).await?;
                self.local_path = Some(local_path.clone());
                self.state = QuantumState::Manifested;
                self.update_observation();
                Ok(local_path)
            }
        }
    }

    /// Decohere: Commit to git and enter superposition
    pub async fn decohere(&mut self) -> Result<()> {
        if self.local_path.is_none() {
            return Err(anyhow!("Cannot decohere non-manifested egg"));
        }

        let commit_hash = self.commit_to_git().await?;
        self.git_commit = Some(commit_hash);
        self.state = QuantumState::Superposition;
        
        println!("🌊 {} entered superposition (git: {})", 
            self.egg_name, self.git_commit.as_ref().unwrap());
        
        Ok(())
    }

    /// Evaporate: Remove local manifestation, remain in git only
    pub async fn evaporate(&mut self) -> Result<()> {
        if let Some(path) = &self.local_path {
            if path.exists() {
                fs::remove_dir_all(path).await?;
                println!("💨 Evaporated {} to ether (local removed)", self.egg_name);
            }
        }
        
        self.local_path = None;
        self.state = QuantumState::Ethereal;
        
        Ok(())
    }

    /// Anchor: Commit and keep local (full materialization)
    pub async fn anchor(&mut self) -> Result<()> {
        self.decohere().await?;
        self.state = QuantumState::Manifested;
        
        println!("⚓ Anchored {} (committed + local)", self.egg_name);
        
        Ok(())
    }

    async fn materialize_from_git(&self, nest_root: &Path) -> Result<PathBuf> {
        let local_path = nest_root.join(&self.egg_name);
        
        // Git sparse checkout or clone specific path
        let output = Command::new("git")
            .args(&["checkout", self.git_commit.as_ref().unwrap(), "--", &self.egg_name])
            .current_dir(nest_root)
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow!("Git checkout failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(local_path)
    }

    async fn sync_from_git(&self, nest_root: &Path) -> Result<()> {
        if let Some(commit) = &self.git_commit {
            let output = Command::new("git")
                .args(&["pull", "origin", commit])
                .current_dir(nest_root)
                .output()
                .await?;
            
            if !output.status.success() {
                return Err(anyhow!("Git sync failed: {}", 
                    String::from_utf8_lossy(&output.stderr)));
            }
        }
        
        Ok(())
    }

    async fn commit_to_git(&self) -> Result<String> {
        let path = self.local_path.as_ref().unwrap();
        
        // Git add
        let output = Command::new("git")
            .args(&["add", path.to_str().unwrap()])
            .current_dir(path.parent().unwrap())
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow!("Git add failed"));
        }
        
        // Git commit
        let message = format!("SnakeEgg: {} gestation checkpoint", self.egg_name);
        let output = Command::new("git")
            .args(&["commit", "-m", &message])
            .current_dir(path.parent().unwrap())
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow!("Git commit failed"));
        }
        
        // Get commit hash
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(path.parent().unwrap())
            .output()
            .await?;
        
        let commit_hash = String::from_utf8(output.stdout)?
            .trim()
            .to_string();
        
        Ok(commit_hash)
    }

    fn update_observation(&mut self) {
        self.last_observed = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }

    /// Check if egg should be evaporated (not observed recently)
    pub fn should_evaporate(&self, max_idle_hours: u64) -> bool {
        if let Some(last_obs) = self.last_observed {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let hours_idle = (now - last_obs) / 3600;
            hours_idle > max_idle_hours && self.state != QuantumState::Ethereal
        } else {
            false
        }
    }
}

/// Quantum Nest Manager: Manages superposition states
#[derive(Debug, Serialize, Deserialize)]
pub struct QuantumNest {
    pub nest_root: PathBuf,
    pub shells: Vec<SchrodingersShell>,
    pub git_repo: String,
    pub max_idle_hours: u64,  // Auto-evaporate after this
}

impl QuantumNest {
    pub fn new(nest_root: PathBuf, git_repo: String) -> Self {
        Self {
            nest_root,
            shells: Vec::new(),
            git_repo,
            max_idle_hours: 24,  // Default: 24 hours idle → evaporate
        }
    }

    pub async fn load_state(&mut self) -> Result<()> {
        let state_path = self.nest_root.join("quantum_state.json");
        if state_path.exists() {
            let content = fs::read_to_string(state_path).await?;
            let loaded: QuantumNest = serde_json::from_str(&content)?;
            self.shells = loaded.shells;
            self.max_idle_hours = loaded.max_idle_hours;
            // nest_root and git_repo are kept from current instance or should be loaded?
            // For now, we trust the loaded shells.
        }
        Ok(())
    }

    pub async fn save_state(&self) -> Result<()> {
        let state_path = self.nest_root.join("quantum_state.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(state_path, content).await?;
        Ok(())
    }

    /// Observe egg (collapse if needed)
    pub async fn observe(&mut self, egg_name: &str) -> Result<PathBuf> {
        if let Some(shell) = self.shells.iter_mut().find(|s| s.egg_name == egg_name) {
            shell.collapse(&self.nest_root).await
        } else {
            // Create new shell
            let mut shell = SchrodingersShell::new(egg_name.to_string());
            let path = shell.collapse(&self.nest_root).await?;
            self.shells.push(shell);
            Ok(path)
        }
    }

    /// Vacuum: Evaporate idle eggs
    pub async fn vacuum(&mut self) -> Result<Vec<String>> {
        let mut evaporated = Vec::new();
        
        for shell in &mut self.shells {
            if shell.should_evaporate(self.max_idle_hours) {
                shell.evaporate().await?;
                evaporated.push(shell.egg_name.clone());
            }
        }
        
        println!("🧹 Vacuum complete: {} eggs evaporated", evaporated.len());
        
        Ok(evaporated)
    }

    /// Checkpoint: Commit all manifested eggs
    pub async fn checkpoint(&mut self) -> Result<()> {
        for shell in &mut self.shells {
            if shell.state == QuantumState::Manifested {
                shell.decohere().await?;
            }
        }
        
        println!("💾 Checkpoint: All eggs committed to ether");
        
        Ok(())
    }

    /// Get storage stats
    pub fn storage_stats(&self) -> StorageStats {
        let manifested = self.shells.iter()
            .filter(|s| s.local_path.is_some())
            .count();
        
        let ethereal = self.shells.iter()
            .filter(|s| s.state == QuantumState::Ethereal)
            .count();
        
        StorageStats {
            total_eggs: self.shells.len(),
            manifested_locally: manifested,
            ethereal_only: ethereal,
            storage_efficiency: if self.shells.is_empty() {
                1.0
            } else {
                1.0 - (manifested as f64 / self.shells.len() as f64)
            },
        }
    }
}

#[derive(Debug)]
pub struct StorageStats {
    pub total_eggs: usize,
    pub manifested_locally: usize,
    pub ethereal_only: usize,
    pub storage_efficiency: f64,  // 0.0 = all local, 1.0 = all ethereal
}
