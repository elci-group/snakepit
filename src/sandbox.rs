use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::venv::VirtualEnvironmentManager;

pub struct VenvSandbox {
    id: String,
    path: PathBuf,
    manager: VirtualEnvironmentManager,
}

impl VenvSandbox {
    pub fn new(id: &str) -> Self {
        let manager = VirtualEnvironmentManager::new();
        // Use a temporary directory for sandboxes
        let sandbox_dir = std::env::temp_dir().join("snakepit-sandbox");
        let path = sandbox_dir.join(id);
        
        // Configure manager to use the sandbox directory
        let manager = manager.with_base_path(sandbox_dir);
        
        Self {
            id: id.to_string(),
            path,
            manager,
        }
    }

    pub async fn create(&self) -> Result<PathBuf> {
        // Create the virtual environment
        self.manager.create_venv(&self.id, None).await?;
        Ok(self.path.clone())
    }

    pub async fn install_package(&self, package: &str, version: Option<&str>) -> Result<()> {
        let python_path = self.manager.activate_venv(&self.id).await?;
        let pip_path = if cfg!(target_os = "windows") {
            python_path.parent().unwrap().join("pip.exe")
        } else {
            python_path.parent().unwrap().join("pip")
        };

        let mut cmd = Command::new(pip_path);
        cmd.arg("install");
        
        if let Some(ver) = version {
            cmd.arg(format!("{}=={}", package, ver));
        } else {
            cmd.arg(package);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install package in sandbox: {}", error));
        }

        Ok(())
    }

    pub async fn run_script(&self, script_path: &Path) -> Result<(bool, String, String)> {
        let python_path = self.manager.activate_venv(&self.id).await?;
        
        let output = Command::new(python_path)
            .arg(script_path)
            .output()
            .context("Failed to run script in sandbox")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok((output.status.success(), stdout, stderr))
    }

    pub async fn run_command(&self, args: &[&str]) -> Result<(bool, String, String)> {
        let python_path = self.manager.activate_venv(&self.id).await?;
        // We use the python executable to run commands, assuming modules or scripts
        // But if we want to run the package binary itself, we might need to look in bin/
        // For now, let's assume we run via python -m or just execute python with args
        
        let output = Command::new(python_path)
            .args(args)
            .output()
            .context("Failed to run command in sandbox")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok((output.status.success(), stdout, stderr))
    }

    pub async fn destroy(&self) -> Result<()> {
        self.manager.delete_venv(&self.id).await?;
        Ok(())
    }

    pub async fn find_installed_module(&self, package_name: &str) -> Result<String> {
        let path = self.manager.get_venv_path(&self.id);
        let site_packages = self.manager.get_site_packages_path(&path)?;
        
        if !site_packages.exists() {
            return Ok(package_name.replace("-", "_"));
        }

        let normalized_name = package_name.replace("-", "_").to_lowercase();
        let mut best_match = None;

        for entry in std::fs::read_dir(&site_packages)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip metadata directories
                if name.ends_with(".dist-info") || name.ends_with(".egg-info") || name == "__pycache__" {
                    continue;
                }
                
                let name_lower = name.to_lowercase();
                
                // Exact match (ignoring case)
                if name_lower == normalized_name {
                    return Ok(name.trim_end_matches(".py").to_string());
                }
                
                // Heuristic: if the package name contains the module name
                if normalized_name.contains(&name_lower) || name_lower.contains(&normalized_name) {
                    best_match = Some(name.trim_end_matches(".py").to_string());
                }

                // Special case for google packages
                if normalized_name.starts_with("google") && name_lower == "google" {
                    return Ok("google".to_string());
                }
            }
        }

        Ok(best_match.unwrap_or_else(|| package_name.replace("-", "_")))
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }
}
