use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone)]
pub enum VenvBackend {
    Venv,
    Virtualenv,
    Conda,
    Poetry,
}

impl VenvBackend {
    pub fn detect() -> Self {
        if Self::command_exists("conda") {
            Self::Conda
        } else if Self::command_exists("poetry") {
            Self::Poetry
        } else if Self::command_exists("virtualenv") {
            Self::Virtualenv
        } else {
            Self::Venv
        }
    }

    fn command_exists(command: &str) -> bool {
        Command::new(command)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
    }
}

pub struct VirtualEnvironmentManager {
    backend: VenvBackend,
    base_path: PathBuf,
}

impl VirtualEnvironmentManager {
    pub fn new() -> Self {
        Self {
            backend: VenvBackend::detect(),
            base_path: Self::get_default_venv_path(),
        }
    }

    pub fn with_backend(mut self, backend: VenvBackend) -> Self {
        self.backend = backend;
        self
    }

    pub fn with_base_path(mut self, path: PathBuf) -> Self {
        self.base_path = path;
        self
    }

    fn get_default_venv_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".snakepit").join("venvs")
        } else {
            PathBuf::from(".snakepit").join("venvs")
        }
    }

    pub async fn create_venv(&self, name: &str, python_version: Option<&str>) -> Result<PathBuf> {
        let venv_path = self.base_path.join(name);
        
        if venv_path.exists() {
            return Err(anyhow::anyhow!("Virtual environment '{}' already exists", name));
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(format!("Creating virtual environment '{}'...", name));

        let result = match self.backend {
            VenvBackend::Venv => self.create_with_venv(&venv_path, python_version).await,
            VenvBackend::Virtualenv => self.create_with_virtualenv(&venv_path, python_version).await,
            VenvBackend::Conda => self.create_with_conda(&venv_path, python_version).await,
            VenvBackend::Poetry => self.create_with_poetry(&venv_path, python_version).await,
        };

        pb.finish_with_message(format!("{} {}", 
            style("✓").green(), 
            style(format!("Created virtual environment '{}'", name)).green()
        ));

        result
    }

    pub async fn activate_venv(&self, name: &str) -> Result<PathBuf> {
        let venv_path = self.base_path.join(name);
        
        if !venv_path.exists() {
            return Err(anyhow::anyhow!("Virtual environment '{}' does not exist", name));
        }

        let python_path = self.get_python_path(&venv_path)?;
        
        println!("{}", style(format!("Virtual environment '{}' activated", name)).green());
        println!("Python path: {}", python_path.display());
        
        Ok(python_path)
    }

    pub async fn delete_venv(&self, name: &str) -> Result<()> {
        let venv_path = self.base_path.join(name);
        
        if !venv_path.exists() {
            return Err(anyhow::anyhow!("Virtual environment '{}' does not exist", name));
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.red} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(format!("Deleting virtual environment '{}'...", name));

        std::fs::remove_dir_all(&venv_path)?;

        pb.finish_with_message(format!("{} {}", 
            style("✓").red(), 
            style(format!("Deleted virtual environment '{}'", name)).red()
        ));

        Ok(())
    }

    pub async fn list_venvs(&self) -> Result<Vec<String>> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let mut venvs = Vec::new();
        
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    venvs.push(name.to_string());
                }
            }
        }

        venvs.sort();
        Ok(venvs)
    }

    pub fn get_venv_path(&self, name: &str) -> PathBuf {
        self.base_path.join(name)
    }

    fn get_python_path(&self, venv_path: &Path) -> Result<PathBuf> {
        let python_path = match self.backend {
            VenvBackend::Venv | VenvBackend::Virtualenv => {
                if cfg!(target_os = "windows") {
                    venv_path.join("Scripts").join("python.exe")
                } else {
                    venv_path.join("bin").join("python")
                }
            }
            VenvBackend::Conda => {
                venv_path.join("bin").join("python")
            }
            VenvBackend::Poetry => {
                // Poetry manages its own virtual environments
                venv_path.join("bin").join("python")
            }
        };

        if python_path.exists() {
            Ok(python_path)
        } else {
            Err(anyhow::anyhow!("Python executable not found in virtual environment"))
        }
    }

    async fn create_with_venv(&self, venv_path: &Path, python_version: Option<&str>) -> Result<PathBuf> {
        let mut cmd = Command::new("python3");
        cmd.arg("-m").arg("venv");
        
        if let Some(version) = python_version {
            // Try to use specific Python version
            let python_cmd = format!("python{}", version);
            if Command::new(&python_cmd).arg("--version").status().is_ok() {
                cmd = Command::new(&python_cmd);
                cmd.arg("-m").arg("venv");
            }
        }
        
        cmd.arg(venv_path);

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to create virtual environment: {}", error));
        }

        Ok(venv_path.to_path_buf())
    }

    async fn create_with_virtualenv(&self, venv_path: &Path, python_version: Option<&str>) -> Result<PathBuf> {
        let mut cmd = Command::new("virtualenv");
        
        if let Some(version) = python_version {
            cmd.arg("-p").arg(&format!("python{}", version));
        }
        
        cmd.arg(venv_path);

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to create virtual environment: {}", error));
        }

        Ok(venv_path.to_path_buf())
    }

    async fn create_with_conda(&self, venv_path: &Path, python_version: Option<&str>) -> Result<PathBuf> {
        let mut cmd = Command::new("conda");
        cmd.arg("create").arg("-y").arg("--prefix").arg(venv_path);
        
        if let Some(version) = python_version {
            cmd.arg(&format!("python={}", version));
        } else {
            cmd.arg("python");
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to create conda environment: {}", error));
        }

        Ok(venv_path.to_path_buf())
    }

    async fn create_with_poetry(&self, venv_path: &Path, python_version: Option<&str>) -> Result<PathBuf> {
        // Poetry manages its own virtual environments
        // We'll create a new project directory and initialize it
        let mut cmd = Command::new("poetry");
        cmd.arg("new").arg(venv_path);
        
        if let Some(version) = python_version {
            cmd.arg("--python").arg(&format!("python{}", version));
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to create poetry project: {}", error));
        }

        Ok(venv_path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_detection() {
        let backend = VenvBackend::detect();
        assert!(matches!(backend, VenvBackend::Venv | VenvBackend::Virtualenv | VenvBackend::Conda | VenvBackend::Poetry));
    }

    #[test]
    fn test_default_venv_path() {
        let path = VirtualEnvironmentManager::get_default_venv_path();
        assert!(path.to_string_lossy().contains("snakepit"));
    }
}
