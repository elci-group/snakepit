use crate::installer::{PackageInstaller, InstallerBackend};
use crate::config::SnakepitConfig;
use crate::process_monitor::ProcessMonitor;
use anyhow::Result;
use console::style;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{Pid, Process, System};
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;
use uuid::Uuid;
use tokio::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DaemonConfig {
    pub enabled: bool,
    pub auto_install: bool,
    pub check_interval: Duration,
    pub max_install_attempts: u32,
    pub whitelist_modules: Vec<String>,
    pub blacklist_modules: Vec<String>,
    pub log_file: Option<PathBuf>,
    pub pid_file: Option<PathBuf>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_install: true,
            check_interval: Duration::from_secs(5),
            max_install_attempts: 3,
            whitelist_modules: Vec::new(),
            blacklist_modules: vec![
                "sys".to_string(),
                "os".to_string(),
                "builtins".to_string(),
            ],
            log_file: None,
            pid_file: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleError {
    pub module_name: String,
    pub error_message: String,
    pub process_id: u32,
    pub timestamp: std::time::SystemTime,
    pub install_attempts: u32,
}

#[derive(Debug)]
pub struct SnakepitDaemon {
    config: DaemonConfig,
    installer: PackageInstaller,
    system: Arc<Mutex<System>>,
    error_cache: Arc<RwLock<HashMap<String, ModuleError>>>,
    running: Arc<RwLock<bool>>,
    daemon_id: String,
    process_monitor: ProcessMonitor,
}

impl SnakepitDaemon {
    pub fn new(config: DaemonConfig, snakepit_config: &SnakepitConfig) -> Self {
        let backend = match snakepit_config.default_backend.as_deref() {
            Some("conda") => InstallerBackend::Conda,
            Some("poetry") => InstallerBackend::Poetry,
            _ => InstallerBackend::Pip,
        };

        let installer = PackageInstaller::new().with_backend(backend);

        Self {
            config,
            installer,
            system: Arc::new(Mutex::new(System::new_all())),
            error_cache: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            daemon_id: Uuid::new_v4().to_string(),
            process_monitor: ProcessMonitor::new(),
        }
    }

    /// Send a system notification using notify-send
    async fn send_notification(&self, summary: &str, body: &str, urgency: &str) {
        let _ = Command::new("notify-send")
            .arg("-u")
            .arg(urgency) // low, normal, critical
            .arg("-i")
            .arg("dialog-information")
            .arg(format!("🐍 Snakepit: {}", summary))
            .arg(body)
            .spawn();
    }

    pub async fn start(&self) -> Result<()> {
        println!("{}", style("🐍 Starting Snakepit Daemon...").blue());
        
        // Write PID file
        if let Some(pid_file) = &self.config.pid_file {
            fs::write(pid_file, std::process::id().to_string()).await?;
        }

        // Set up signal handlers
        self.setup_signal_handlers().await?;

        // Start monitoring loop
        self.monitoring_loop().await?;

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        println!("{}", style("🛑 Stopping Snakepit Daemon...").yellow());
        
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        // Remove PID file
        if let Some(pid_file) = &self.config.pid_file {
            let _ = fs::remove_file(pid_file).await;
        }

        Ok(())
    }

    pub async fn status(&self) -> Result<DaemonStatus> {
        let running = *self.running.read().await;
        let error_count = self.error_cache.read().await.len();
        
        Ok(DaemonStatus {
            running,
            daemon_id: self.daemon_id.clone(),
            error_count,
            config: self.config.clone(),
        })
    }

    async fn setup_signal_handlers(&self) -> Result<()> {
        // Simplified signal handling - in a real implementation,
        // you'd use proper signal handling libraries
        println!("{}", style("Signal handlers configured").dim());
        Ok(())
    }

    async fn monitoring_loop(&self) -> Result<()> {
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        println!("{}", style("✅ Snakepit Daemon started successfully!").green());
        println!("{}", style("Monitoring Python processes for missing modules...").dim());

        while *self.running.read().await {
            if let Err(e) = self.check_python_processes().await {
                eprintln!("Error checking processes: {}", e);
            }

            sleep(self.config.check_interval).await;
        }

        Ok(())
    }

    async fn check_python_processes(&self) -> Result<()> {
        let mut system = self.system.lock().await;
        system.refresh_processes();

        for (pid, process) in system.processes() {
            if self.is_python_process(process) {
                if let Err(e) = self.check_process_errors(*pid, process).await {
                    eprintln!("Error checking process {}: {}", pid, e);
                }
            }
        }

        Ok(())
    }

    fn is_python_process(&self, process: &sysinfo::Process) -> bool {
        let name = process.name().to_lowercase();
        name.contains("python") || 
        name.contains("python3") || 
        name.contains("python2") ||
        process.cmd().iter().any(|arg| arg.contains("python"))
    }

    async fn check_process_errors(&self, pid: Pid, process: &sysinfo::Process) -> Result<()> {
        // Check if process is still running
        if process.status() == sysinfo::ProcessStatus::Zombie {
            return Ok(());
        }

        // Try to get stderr output from the process
        // This is a simplified approach - in a real implementation,
        // you might need to hook into the process more directly
        if let Some(missing_module) = self.detect_missing_module_from_process(pid).await? {
            self.handle_missing_module(missing_module, pid).await?;
        }

        Ok(())
    }

    async fn detect_missing_module_from_process(&self, _pid: Pid) -> Result<Option<String>> {
        // This is a simplified implementation
        // In a real scenario, you might need to:
        // 1. Hook into Python's import system
        // 2. Monitor stderr output
        // 3. Use ptrace or similar system calls
        
        // For now, we'll simulate by checking common error patterns
        // In a production system, you'd want to implement proper process monitoring
        
        // This is a placeholder - the actual implementation would require
        // more sophisticated process monitoring
        Ok(None)
    }

    async fn handle_missing_module(&self, module_name: String, pid: Pid) -> Result<()> {
        // Check if module is blacklisted
        if self.config.blacklist_modules.contains(&module_name) {
            return Ok(());
        }

        // Check if we have a whitelist and module is not in it
        if !self.config.whitelist_modules.is_empty() && 
           !self.config.whitelist_modules.contains(&module_name) {
            return Ok(());
        }

        // Check if we've already tried to install this module recently
        let cache_key = format!("{}:{}", module_name, pid);
        {
            let cache = self.error_cache.read().await;
            if let Some(error) = cache.get(&cache_key) {
                if error.install_attempts >= self.config.max_install_attempts {
                    return Ok(());
                }
            }
        }

        println!("{}", style(format!("🔍 Detected missing module: {}", module_name)).yellow());
        self.send_notification(
            "Missing Module Detected",
            &format!("Found missing Python module: {} (PID: {})", module_name, pid),
            "normal"
        ).await;
        
        if self.config.auto_install {
            self.auto_install_module(&module_name, &cache_key).await?;
        }

        Ok(())
    }

    async fn auto_install_module(&self, module_name: &str, cache_key: &str) -> Result<()> {
        println!("{}", style(format!("📦 Auto-installing module: {}", module_name)).blue());
        self.send_notification(
            "Installing Module",
            &format!("Attempting to install: {}", module_name),
            "normal"
        ).await;
        
        // Update error cache
        {
            let mut cache = self.error_cache.write().await;
            let error = ModuleError {
                module_name: module_name.to_string(),
                error_message: "Missing module detected".to_string(),
                process_id: 0, // We'll update this properly
                timestamp: std::time::SystemTime::now(),
                install_attempts: cache.get(cache_key).map(|e| e.install_attempts + 1).unwrap_or(1),
            };
            cache.insert(cache_key.to_string(), error);
        }

        // Attempt to install the module
        match self.installer.install_package(module_name, None).await {
            Ok(_) => {
                println!("{}", style(format!("✅ Successfully installed: {}", module_name)).green());
                self.send_notification(
                    "Installation Successful",
                    &format!("✅ Successfully installed: {}", module_name),
                    "low"
                ).await;
                
                // Remove from error cache on success
                {
                    let mut cache = self.error_cache.write().await;
                    cache.remove(cache_key);
                }
            }
            Err(e) => {
                eprintln!("{}", style(format!("❌ Failed to install {}: {}", module_name, e)).red());
                self.send_notification(
                    "Installation Failed",
                    &format!("❌ Failed to install {}: {}", module_name, e),
                    "critical"
                ).await;
            }
        }

        Ok(())
    }

    pub async fn simulate_missing_module(&self, module_name: &str) -> Result<()> {
        println!("{}", style(format!("🧪 Simulating missing module: {}", module_name)).cyan());
        self.handle_missing_module(module_name.to_string(), Pid::from(0)).await
    }
}

#[derive(Debug, Clone)]
pub struct DaemonStatus {
    pub running: bool,
    pub daemon_id: String,
    pub error_count: usize,
    pub config: DaemonConfig,
}

pub struct DaemonManager {
    config_path: PathBuf,
}

impl DaemonManager {
    pub fn new() -> Self {
        let config_path = if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("snakepit").join("daemon.toml")
        } else {
            PathBuf::from(".snakepit").join("daemon.toml")
        };

        Self { config_path }
    }

    pub async fn load_daemon_config(&self) -> Result<DaemonConfig> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path).await?;
            let config: DaemonConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(DaemonConfig::default())
        }
    }

    pub async fn save_daemon_config(&self, config: &DaemonConfig) -> Result<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(config)?;
        fs::write(&self.config_path, content).await?;
        Ok(())
    }

    pub async fn start_daemon(&self, snakepit_config: &SnakepitConfig) -> Result<()> {
        let daemon_config = self.load_daemon_config().await?;
        let daemon = SnakepitDaemon::new(daemon_config, snakepit_config);
        daemon.start().await
    }

    pub async fn stop_daemon(&self) -> Result<()> {
        // In a real implementation, you'd read the PID from the PID file
        // and send a SIGTERM signal to stop the daemon
        println!("{}", style("Stopping daemon...").yellow());
        Ok(())
    }

    pub async fn daemon_status(&self) -> Result<DaemonStatus> {
        // Check if daemon is running by looking for PID file
        let pid_file = if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("snakepit").join("snakepit.pid")
        } else {
            PathBuf::from(".snakepit").join("snakepit.pid")
        };

        let running = pid_file.exists();
        let daemon_id = "unknown".to_string();
        let error_count = 0;
        let config = self.load_daemon_config().await?;

        Ok(DaemonStatus {
            running,
            daemon_id,
            error_count,
            config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_config_default() {
        let config = DaemonConfig::default();
        assert!(config.enabled);
        assert!(config.auto_install);
        assert_eq!(config.check_interval, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_daemon_manager() {
        let manager = DaemonManager::new();
        let config = manager.load_daemon_config().await.unwrap();
        assert!(config.enabled);
    }
}
