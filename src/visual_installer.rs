use anyhow::Result;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use console::style;

/// Visual installer that uses the snake game GUI
pub struct VisualInstaller {
    vip_path: PathBuf,
    use_gui: bool,
}

impl VisualInstaller {
    pub fn new() -> Self {
        let vip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vip");
        
        // Check if GUI should be used (DISPLAY available and pygame installed)
        let use_gui = std::env::var("DISPLAY").is_ok() 
            && std::env::var("VIP_NO_GUI").is_err()
            && vip_path.exists();
        
        Self {
            vip_path,
            use_gui,
        }
    }

    /// Install a package with visual feedback
    pub async fn install_package(&self, package: &str, version: Option<&str>) -> Result<()> {
        if self.use_gui {
            self.install_with_gui(package, version).await
        } else {
            self.install_classic(package, version).await
        }
    }

    /// Install using the visual snake game interface
    async fn install_with_gui(&self, package: &str, version: Option<&str>) -> Result<()> {
        let package_spec = if let Some(ver) = version {
            format!("{}=={}", package, ver)
        } else {
            package.to_string()
        };

        println!("{}", style("🐍 Launching visual installer...").cyan());

        let mut cmd = Command::new(&self.vip_path);
        cmd.arg("install")
           .arg(&package_spec)
           .stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        
        // Stream output
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("{}", line);
                }
            }
        }

        let status = child.wait()?;

        if status.success() {
            println!("{}", style("✓ Package installed successfully!").green());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Installation failed with exit code: {}", status.code().unwrap_or(-1)))
        }
    }

    /// Fallback to classic pip installation
    async fn install_classic(&self, package: &str, version: Option<&str>) -> Result<()> {
        let package_spec = if let Some(ver) = version {
            format!("{}=={}", package, ver)
        } else {
            package.to_string()
        };

        println!("{}", style("Installing package (classic mode)...").blue());

        let output = Command::new("python3")
            .args(&["-m", "pip", "install", &package_spec])
            .output()?;

        if output.status.success() {
            println!("{}", style("✓ Package installed successfully!").green());
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Installation failed: {}", error_msg))
        }
    }

    /// Install multiple packages with visual feedback
    pub async fn install_packages(&self, packages: &[&str]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        if self.use_gui && packages.len() > 1 {
            // Use VIP for batch installation
            println!("{}", style(format!("🐍 Installing {} packages with visualization...", packages.len())).cyan());

            let mut cmd = Command::new(&self.vip_path);
            cmd.arg("install");
            for pkg in packages {
                cmd.arg(pkg);
            }
            cmd.stdin(Stdio::null())
               .stdout(Stdio::inherit())
               .stderr(Stdio::inherit());

            let status = cmd.status()?;

            if status.success() {
                println!("{}", style("✓ All packages installed successfully!").green());
                Ok(())
            } else {
                Err(anyhow::anyhow!("Batch installation failed"))
            }
        } else {
            // Install one by one
            for package in packages {
                self.install_package(package, None).await?;
            }
            Ok(())
        }
    }

    /// Check if visual mode is available
    pub fn is_visual_available(&self) -> bool {
        self.use_gui
    }
}

impl Default for VisualInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_installer_creation() {
        let installer = VisualInstaller::new();
        // Just verify it can be created
        assert!(installer.vip_path.to_string_lossy().contains("vip"));
    }
}
