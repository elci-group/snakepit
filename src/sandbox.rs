use crate::installer::validate_package_name;
use crate::snakegg;
use crate::venv::VirtualEnvironmentManager;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The persistent snakepit environment where packages live after approval.
/// This is the environment that `snakepit run` uses to execute user scripts.
pub struct SnakepitEnv {
    path: PathBuf,
    manager: VirtualEnvironmentManager,
}

impl SnakepitEnv {
    /// Get the persistent environment (at ~/.snakepit/env/)
    pub fn new() -> Self {
        let base_path = Self::get_env_base();
        let manager = VirtualEnvironmentManager::new().with_base_path(base_path.clone());
        Self {
            path: base_path.join("default"),
            manager,
        }
    }

    fn get_env_base() -> PathBuf {
        if let Some(home) = snakegg::native::dirs::home_dir() {
            home.join(".snakepit").join("env")
        } else {
            PathBuf::from(".snakepit").join("env")
        }
    }

    /// Check if the persistent environment exists
    pub fn exists(&self) -> bool {
        self.path.exists() && self.get_python_path().map(|p| p.exists()).unwrap_or(false)
    }

    /// Create the persistent environment if it doesn't exist
    pub async fn ensure_exists(&self) -> Result<()> {
        if self.exists() {
            return Ok(());
        }
        std::fs::create_dir_all(Self::get_env_base())?;
        self.manager.create_venv("default", None).await?;
        Ok(())
    }

    /// Create or recreate the environment with a specific Python version
    pub async fn init(&self, python_version: Option<&str>) -> Result<()> {
        if self.exists() {
            self.manager.delete_venv("default").await?;
        }
        std::fs::create_dir_all(Self::get_env_base())?;
        self.manager.create_venv("default", python_version).await?;
        Ok(())
    }

    /// Destroy the persistent environment
    pub async fn destroy(&self) -> Result<()> {
        if self.exists() {
            self.manager.delete_venv("default").await?;
        }
        Ok(())
    }

    /// Get the Python executable path within the persistent env
    pub fn get_python_path(&self) -> Result<PathBuf> {
        let python = if cfg!(target_os = "windows") {
            self.path.join("Scripts").join("python.exe")
        } else {
            self.path.join("bin").join("python")
        };
        Ok(python)
    }

    /// Get the pip executable path
    pub fn get_pip_path(&self) -> Result<PathBuf> {
        let pip = if cfg!(target_os = "windows") {
            self.path.join("Scripts").join("pip.exe")
        } else {
            self.path.join("bin").join("pip")
        };
        Ok(pip)
    }

    /// Get the env path
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    /// Get site-packages path
    pub fn get_site_packages(&self) -> Result<PathBuf> {
        self.manager.get_site_packages_path(&self.path)
    }

    /// Install a package into the persistent environment
    pub async fn install_package(&self, package: &str, version: Option<&str>) -> Result<()> {
        validate_package_name(package)?;
        self.ensure_exists().await?;
        let pip_path = self.get_pip_path()?;

        let mut cmd = Command::new(&pip_path);
        cmd.arg("install");

        if let Some(ver) = version {
            cmd.arg(format!("{}=={}", package, ver));
        } else {
            cmd.arg(package);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to install package in snakepit env: {}",
                error
            ));
        }

        Ok(())
    }

    /// Run a script using the persistent environment's Python
    pub fn run_script(
        &self,
        script_path: &Path,
        args: &[String],
    ) -> Result<(bool, String, String)> {
        let python = self.get_python_path()?;
        if !python.exists() {
            return Err(anyhow::anyhow!(
                "Snakepit environment not found. Run 'snakepit env init' or 'snakepit install <package>' first."
            ));
        }

        let output = Command::new(&python)
            .arg(script_path)
            .args(args)
            .output()
            .context("Failed to run script in snakepit environment")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((output.status.success(), stdout, stderr))
    }

    /// Run an arbitrary command with the persistent environment activated
    /// (sets PATH and VIRTUAL_ENV so the env's Python/pip are used)
    pub fn run_command(&self, command: &str, args: &[String]) -> Result<i32> {
        let python = self.get_python_path()?;
        if !python.exists() {
            return Err(anyhow::anyhow!(
                "Snakepit environment not found. Run 'snakepit env init' or 'snakepit install <package>' first."
            ));
        }

        let bin_dir = if cfg!(target_os = "windows") {
            self.path.join("Scripts")
        } else {
            self.path.join("bin")
        };

        // Prepend the env's bin/ to PATH
        let current_path = std::env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", bin_dir.display(), current_path);

        let site_packages = self
            .get_site_packages()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        // Determine if the command is "python" or "python3" — use the env's Python
        let actual_command = if command == "python" || command == "python3" {
            python.to_string_lossy().to_string()
        } else {
            command.to_string()
        };

        let status = Command::new(&actual_command)
            .args(args)
            .env("PATH", &new_path)
            .env("VIRTUAL_ENV", &self.path)
            .env("PYTHONPATH", &site_packages)
            .status()
            .context(format!(
                "Failed to run '{}' in snakepit environment",
                command
            ))?;

        Ok(status.code().unwrap_or(1))
    }
}

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
        validate_package_name(package)?;
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
            return Err(anyhow::anyhow!(
                "Failed to install package in sandbox: {}",
                error
            ));
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
                if name.ends_with(".dist-info")
                    || name.ends_with(".egg-info")
                    || name == "__pycache__"
                {
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
