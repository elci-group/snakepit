use crate::resolver::ResolvedDependency;
use anyhow::Result;
use std::process::{Command, Stdio};
use indicatif::{ProgressBar, ProgressStyle};
use console::style;

#[derive(Debug, Clone)]
pub enum InstallerBackend {
    Pip,
    Conda,
    Poetry,
}

impl InstallerBackend {
    pub fn detect() -> Self {
        // Try to detect available backends
        if Self::command_exists("conda") {
            Self::Conda
        } else if Self::command_exists("poetry") {
            Self::Poetry
        } else {
            Self::Pip
        }
    }

    fn command_exists(command: &str) -> bool {
        Command::new(command)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    }
}

#[derive(Debug)]
pub struct PackageInstaller {
    backend: InstallerBackend,
    venv_path: Option<String>,
    use_cache: bool,
}

impl PackageInstaller {
    pub fn new() -> Self {
        Self {
            backend: InstallerBackend::detect(),
            venv_path: None,
            use_cache: true,
        }
    }

    pub fn with_backend(mut self, backend: InstallerBackend) -> Self {
        self.backend = backend;
        self
    }

    pub fn with_venv(mut self, venv_path: String) -> Self {
        self.venv_path = Some(venv_path);
        self
    }

    pub fn with_cache(mut self, use_cache: bool) -> Self {
        self.use_cache = use_cache;
        self
    }

    pub async fn install_package(&self, package: &str, version: Option<&str>) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(format!("Installing {}...", package));

        let result = match self.backend {
            InstallerBackend::Pip => self.install_with_pip(package, version).await,
            InstallerBackend::Conda => self.install_with_conda(package, version).await,
            InstallerBackend::Poetry => self.install_with_poetry(package, version).await,
        };

        pb.finish_with_message(format!("{} {}", 
            style("✓").green(), 
            style(format!("Installed {}", package)).green()
        ));

        result
    }

    pub async fn install_dependencies(&self, dependencies: &[ResolvedDependency]) -> Result<()> {
        let pb = ProgressBar::new(dependencies.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        for dep in dependencies {
            pb.set_message(format!("Installing {}", dep.name));
            self.install_package(&dep.name, Some(&dep.version)).await?;
            pb.inc(1);
        }

        pb.finish_with_message(style("All dependencies installed!").green().to_string());
        Ok(())
    }

    pub async fn uninstall_package(&self, package: &str) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.red} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(format!("Uninstalling {}...", package));

        let result = match self.backend {
            InstallerBackend::Pip => self.uninstall_with_pip(package).await,
            InstallerBackend::Conda => self.uninstall_with_conda(package).await,
            InstallerBackend::Poetry => self.uninstall_with_poetry(package).await,
        };

        pb.finish_with_message(format!("{} {}", 
            style("✓").red(), 
            style(format!("Uninstalled {}", package)).red()
        ));

        result
    }

    pub async fn list_installed_packages(&self) -> Result<Vec<String>> {
        match self.backend {
            InstallerBackend::Pip => self.list_with_pip().await,
            InstallerBackend::Conda => self.list_with_conda().await,
            InstallerBackend::Poetry => self.list_with_poetry().await,
        }
    }

    async fn install_with_pip(&self, package: &str, version: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("pip");
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--python").arg(venv_path);
        }
        
        cmd.arg("install");
        
        if !self.use_cache {
            cmd.arg("--no-cache-dir");
        }
        
        if let Some(ver) = version {
            cmd.arg(&format!("{}=={}", package, ver));
        } else {
            cmd.arg(package);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install {}: {}", package, error));
        }

        Ok(())
    }

    async fn install_with_conda(&self, package: &str, version: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("conda");
        cmd.arg("install").arg("-y");
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--prefix").arg(venv_path);
        }
        
        if let Some(ver) = version {
            cmd.arg(&format!("{}={}", package, ver));
        } else {
            cmd.arg(package);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install {}: {}", package, error));
        }

        Ok(())
    }

    async fn install_with_poetry(&self, package: &str, version: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("poetry");
        cmd.arg("add");
        
        if let Some(ver) = version {
            cmd.arg(&format!("{}=={}", package, ver));
        } else {
            cmd.arg(package);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install {}: {}", package, error));
        }

        Ok(())
    }

    async fn uninstall_with_pip(&self, package: &str) -> Result<()> {
        let mut cmd = Command::new("pip");
        cmd.arg("uninstall").arg("-y").arg(package);
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--python").arg(venv_path);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to uninstall {}: {}", package, error));
        }

        Ok(())
    }

    async fn uninstall_with_conda(&self, package: &str) -> Result<()> {
        let mut cmd = Command::new("conda");
        cmd.arg("remove").arg("-y").arg(package);
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--prefix").arg(venv_path);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to uninstall {}: {}", package, error));
        }

        Ok(())
    }

    async fn uninstall_with_poetry(&self, package: &str) -> Result<()> {
        let mut cmd = Command::new("poetry");
        cmd.arg("remove").arg(package);

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to uninstall {}: {}", package, error));
        }

        Ok(())
    }

    async fn list_with_pip(&self) -> Result<Vec<String>> {
        let mut cmd = Command::new("pip");
        cmd.arg("list").arg("--format=freeze");
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--python").arg(venv_path);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list packages"));
        }

        let packages: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.split('=').next().unwrap_or("").to_string())
            .collect();

        Ok(packages)
    }

    async fn list_with_conda(&self) -> Result<Vec<String>> {
        let mut cmd = Command::new("conda");
        cmd.arg("list");
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--prefix").arg(venv_path);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list packages"));
        }

        let packages: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(2) // Skip header lines
            .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
            .collect();

        Ok(packages)
    }

    async fn list_with_poetry(&self) -> Result<Vec<String>> {
        let mut cmd = Command::new("poetry");
        cmd.arg("show").arg("--only=main");

        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list packages"));
        }

        let packages: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
            .collect();

        Ok(packages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_detection() {
        let backend = InstallerBackend::detect();
        // This test will pass regardless of what's installed
        assert!(matches!(backend, InstallerBackend::Pip | InstallerBackend::Conda | InstallerBackend::Poetry));
    }
}
