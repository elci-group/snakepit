use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::native::id;
use crate::native::style::{red, green, yellow, blue, cyan, magenta, bold, dim};
use serde::{Serialize, Deserialize};
use crate::sandbox::VenvSandbox;
use crate::installer::{PackageInstaller, InstallerBackend};
use crate::charmer::SnakeCharmer;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PackageStatus {
    Pending,
    Ingesting,
    Testing,
    Collaborating,
    Failed,
    Approved,
    Conscripted,
    Destroyed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: Option<String>,
    pub status: PackageStatus,
    pub ingest_time: u64,
    pub test_time: Option<u64>,
    pub install_time: Option<u64>,
    pub sandbox_id: String,
    pub error_log: Vec<String>,
    pub success_log: Vec<String>,
}

pub struct SnakepitHandler {
    active_packages: std::collections::HashMap<String, PackageMetadata>,
}

impl SnakepitHandler {
    pub fn new() -> Self {
        Self {
            active_packages: std::collections::HashMap::new(),
        }
    }

    pub async fn handle_package(&mut self, package: &str, version: Option<&str>, test_script: Option<&Path>) -> Result<bool> {
        println!("{}", blue(format!("🐍 Starting Smart Snakepit handling for {}", package)));

        // Start Charmer Task in Parallel
        let package_name = package.to_string();
        let charmer_handle = tokio::spawn(async move {
            if let Ok(charmer) = SnakeCharmer::new() {
                println!("{}", magenta("🐍 CHARMER: Consulting the oracles (PyPI + Gemini)..."));
                charmer.charm_package(&package_name).await
            } else {
                Err(anyhow::anyhow!("Charmer not available"))
            }
        });

        // Phase 1: Ingest
        let mut meta = self.ingest(package, version).await?;
        
        if meta.status == PackageStatus::Failed {
            self.kill_destroy(&meta).await?;
            return Ok(false);
        }

        // Phase 2: Test/Collaborate
        let success = self.test_collaborate(&mut meta, test_script, charmer_handle).await?;
        if !success {
            self.kill_destroy(&meta).await?;
            return Ok(false);
        }

        // Phase 4: Conscript (Phase 3 happens automatically on success)
        let installed = self.conscript_install(&mut meta).await?;
        
        // Cleanup sandbox
        self.kill_destroy(&meta).await?;

        Ok(installed)
    }

    async fn ingest(&mut self, package: &str, version: Option<&str>) -> Result<PackageMetadata> {
        let sandbox_id = id::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let mut meta = PackageMetadata {
            name: package.to_string(),
            version: version.map(|v| v.to_string()),
            status: PackageStatus::Ingesting,
            ingest_time: now,
            test_time: None,
            install_time: None,
            sandbox_id: sandbox_id.clone(),
            error_log: Vec::new(),
            success_log: Vec::new(),
        };

        println!("{}", cyan(format!("🐍 INGEST: Starting ingestion of {}", package)));

        let sandbox = VenvSandbox::new(&sandbox_id);
        
        match sandbox.create().await {
            Ok(_) => {
                match sandbox.install_package(package, version).await {
                    Ok(_) => {
                        meta.status = PackageStatus::Testing;
                        meta.success_log.push("Sandbox created and package installed".to_string());
                        println!("{}", green(format!("✅ INGEST: Successfully ingested {}", package)));
                    }
                    Err(e) => {
                        meta.status = PackageStatus::Failed;
                        meta.error_log.push(format!("Failed to install package: {}", e));
                        println!("{}", red(format!("❌ INGEST: Failed to install {}", package)));
                    }
                }
            }
            Err(e) => {
                meta.status = PackageStatus::Failed;
                meta.error_log.push(format!("Failed to create sandbox: {}", e));
                println!("{}", red(format!("❌ INGEST: Failed to create sandbox: {}", e)));
            }
        }

        Ok(meta)
    }

    async fn test_collaborate(
        &mut self, 
        meta: &mut PackageMetadata, 
        test_script: Option<&Path>,
        charmer_handle: tokio::task::JoinHandle<Result<crate::charmer::TestStrategy>>
    ) -> Result<bool> {
        println!("{}", cyan(format!("🧪 TEST/COLLABORATE: Validating {}", meta.name)));
        meta.status = PackageStatus::Collaborating;
        meta.test_time = Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());

        let sandbox = VenvSandbox::new(&meta.sandbox_id);
        
        // Check if user provided a script
        if let Some(path) = test_script {
            let script_path = path.to_path_buf();
            return self.run_validation_script(&sandbox, &script_path, meta).await;
        }

        // Await Charmer Result
        let strategy = match charmer_handle.await {
            Ok(Ok(s)) => Some(s),
            _ => None,
        };

        match strategy {
            Some(crate::charmer::TestStrategy::SimpleCommand(cmd)) => {
                println!("{}", magenta(format!("🐍 CHARMER: Suggests running command: {}", cmd)));
                let args: Vec<&str> = cmd.split_whitespace().collect();
                match sandbox.run_command(&args).await {
                    Ok((success, stdout, stderr)) => {
                        self.handle_validation_result(success, stdout, stderr, meta)
                    }
                    Err(e) => {
                        println!("{}", red(format!("❌ Validation error: {}", e)));
                        Ok(false)
                    }
                }
            }
            Some(crate::charmer::TestStrategy::PythonScript(code)) => {
                println!("{}", magenta("🐍 CHARMER: Generated a custom Python test script."));
                let test_path = sandbox.get_path().join("test_script.py");
                std::fs::write(&test_path, code)?;
                if self.run_validation_script(&sandbox, &test_path, meta).await? {
                    Ok(true)
                } else {
                    println!("{}", yellow("⚠️  Charmer script failed. Attempting Smart Inspection fallback..."));
                    self.run_smart_inspection(&sandbox, meta).await
                }
            }
            None => {
                println!("{}", yellow("🐍 CHARMER: Silent. Using smart inspection fallback."));
                self.run_smart_inspection(&sandbox, meta).await
            }
        }
    }

    async fn run_smart_inspection(&self, sandbox: &VenvSandbox, meta: &mut PackageMetadata) -> Result<bool> {
        let module_name = sandbox.find_installed_module(&meta.name).await
            .unwrap_or_else(|_| meta.name.replace("-", "_"));
        
        println!("{}", dim(format!("🔍 SMART INSPECT: Detected module name: {}", module_name)));

        let default_test = format!(
            r#"
import sys
try:
    import {}
    print("✅ Successfully imported {}", flush=True)
except ImportError as e:
    print(f"❌ Failed to import {}: {{e}}", file=sys.stderr, flush=True)
    sys.exit(1)
except Exception as e:
    print(f"❌ Unexpected error: {{e}}", file=sys.stderr, flush=True)
    sys.exit(1)
"#,
            module_name, meta.name, meta.name
        );
        let test_path = sandbox.get_path().join("test_script.py");
        std::fs::write(&test_path, default_test)?;
        self.run_validation_script(sandbox, &test_path, meta).await
    }

    async fn run_validation_script(&self, sandbox: &VenvSandbox, script_path: &Path, meta: &mut PackageMetadata) -> Result<bool> {
        match sandbox.run_script(script_path).await {
            Ok((success, stdout, stderr)) => {
                self.handle_validation_result(success, stdout, stderr, meta)
            }
            Err(e) => {
                meta.status = PackageStatus::Failed;
                meta.error_log.push(format!("Test execution error: {}", e));
                println!("{}", red(format!("❌ TEST/COLLABORATE: Error testing {}: {}", meta.name, e)));
                Ok(false)
            }
        }
    }

    fn handle_validation_result(&self, success: bool, stdout: String, stderr: String, meta: &mut PackageMetadata) -> Result<bool> {
        if success {
            meta.status = PackageStatus::Approved;
            meta.success_log.push("Validation passed".to_string());
            println!("{}", green(format!("✅ TEST/COLLABORATE: {} approved for installation", meta.name)));
            if !stdout.is_empty() { println!("   {}", stdout.trim()); }
            Ok(true)
        } else {
            meta.status = PackageStatus::Failed;
            meta.error_log.push(format!("Validation failed: {}", stderr));
            println!("{}", red(format!("❌ TEST/COLLABORATE: {} failed validation", meta.name)));
            if !stderr.is_empty() { println!("   {}", stderr.trim()); }
            
            // Manual Verification Prompt
            println!("{}", yellow("\n⚠️  Automated testing failed, but the package installed successfully."));
            println!("{}", bold("Do you want to manually verify/force install this package? [y/N]"));
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() == "y" {
                println!("{}", green(format!("🛡️  Manual Override: Approving {}", meta.name)));
                meta.status = PackageStatus::Approved;
                meta.success_log.push("Manually approved by user".to_string());
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    async fn conscript_install(&mut self, meta: &mut PackageMetadata) -> Result<bool> {
        println!("{}", cyan(format!("⚔️ CONSCRIPT: Installing {}", meta.name)));
        meta.install_time = Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());

        let installer = PackageInstaller::new();
        
        match installer.install_package(&meta.name, meta.version.as_deref()).await {
            Ok(_) => {
                meta.status = PackageStatus::Conscripted;
                meta.success_log.push("Package installed locally".to_string());
                println!("{}", green(format!("✅ CONSCRIPT: Successfully installed {}", meta.name)));
                Ok(true)
            }
            Err(e) => {
                meta.status = PackageStatus::Failed;
                meta.error_log.push(format!("Installation failed: {}", e));
                println!("{}", red(format!("❌ CONSCRIPT: Failed to install {}: {}", meta.name, e)));
                Ok(false)
            }
        }
    }

    async fn kill_destroy(&mut self, meta: &PackageMetadata) -> Result<()> {
        println!("{}", dim(format!("💀 KILL/DESTROY: Cleaning up {}", meta.name)));
        let sandbox = VenvSandbox::new(&meta.sandbox_id);
        sandbox.destroy().await?;
        Ok(())
    }
}
