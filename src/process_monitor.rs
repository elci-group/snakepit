use anyhow::Result;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use sysinfo::{Pid, System};
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::sleep;
use crate::native::datetime::DateTime;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub cmd: Vec<String>,
    pub start_time: SystemTime,
    pub last_check: SystemTime,
    pub error_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleError {
    pub module_name: String,
    pub error_message: String,
    pub process_pid: u32,
    pub timestamp: SystemTime,
    pub resolved: bool,
}

#[derive(Debug)]
pub struct ProcessMonitor {
    system: Arc<RwLock<System>>,
    python_processes: Arc<RwLock<HashMap<Pid, ProcessInfo>>>,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            python_processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_monitoring(&self, check_interval: Duration) -> Result<()> {
        println!("🔍 Starting process monitoring...");
        
        loop {
            if let Err(e) = self.scan_processes().await {
                eprintln!("Error scanning processes: {}", e);
            }
            
            sleep(check_interval).await;
        }
    }

    pub async fn scan_processes(&self) -> Result<()> {
        let mut system = self.system.write().await;
        system.refresh_processes();

        let mut python_processes = self.python_processes.write().await;
        let current_time = SystemTime::now();

        // Find all Python processes
        for (pid, process) in system.processes() {
            if self.is_python_process(process) {
                let process_info = ProcessInfo {
                    pid: *pid,
                    name: process.name().to_string(),
                    cmd: process.cmd().to_vec(),
                    start_time: SystemTime::UNIX_EPOCH + Duration::from_secs(process.start_time()),
                    last_check: current_time,
                    error_count: 0,
                };

                python_processes.insert(*pid, process_info);
            }
        }

        // Check for new errors in existing processes
        for (pid, process_info) in python_processes.iter_mut() {
            if let Some(process) = system.process(*pid) {
                if let Some(missing_module) = self.check_process_for_errors(*pid, process).await? {
                    process_info.error_count += 1;
                    println!("🚨 Detected missing module: {} in process {}", missing_module, pid);
                    
                    // Here you would trigger the auto-installation
                    self.handle_missing_module(missing_module, *pid).await?;
                }
            }
        }

        Ok(())
    }

    fn is_python_process(&self, process: &sysinfo::Process) -> bool {
        let name = process.name().to_lowercase();
        let cmd = process.cmd().join(" ");
        
        name.contains("python") || 
        name.contains("python3") || 
        name.contains("python2") ||
        cmd.contains("python") ||
        cmd.contains("python3") ||
        cmd.contains("python2")
    }

    async fn check_process_for_errors(&self, pid: Pid, process: &sysinfo::Process) -> Result<Option<String>> {
        // Method 1: Check stderr output (if available)
        if let Some(missing_module) = self.check_stderr_output(pid).await? {
            return Ok(Some(missing_module));
        }

        // Method 2: Check process status and recent activity
        if let Some(missing_module) = self.check_process_status(pid, process).await? {
            return Ok(Some(missing_module));
        }

        // Method 3: Monitor file system activity for import attempts
        if let Some(missing_module) = self.check_import_activity(pid).await? {
            return Ok(Some(missing_module));
        }

        Ok(None)
    }

    async fn check_stderr_output(&self, _pid: Pid) -> Result<Option<String>> {
        // This is a simplified approach - in reality, you'd need to hook into the process
        // or use more sophisticated monitoring techniques
        
        // For demonstration, we'll simulate checking stderr
        // In a real implementation, you might use:
        // - ptrace to hook into the process
        // - LD_PRELOAD to intercept library calls
        // - strace to monitor system calls
        
        Ok(None)
    }

    async fn check_process_status(&self, _pid: Pid, process: &sysinfo::Process) -> Result<Option<String>> {
        // Check if process is in a problematic state
        if process.status() == sysinfo::ProcessStatus::Zombie {
            return Ok(None);
        }

        // Check CPU usage - if it's very low but the process is still running,
        // it might be waiting for a missing module
        let cpu_usage = process.cpu_usage();
        if cpu_usage < 1.0 && process.run_time() > 1 {
            // This could indicate the process is stuck waiting for a module
            // In a real implementation, you'd need more sophisticated detection
        }

        Ok(None)
    }

    async fn check_import_activity(&self, _pid: Pid) -> Result<Option<String>> {
        // Monitor file system activity for Python import attempts
        // This would involve monitoring /usr/lib/python*/site-packages/
        // and other Python paths for failed import attempts
        
        // For now, this is a placeholder
        Ok(None)
    }

    async fn handle_missing_module(&self, module_name: String, pid: Pid) -> Result<()> {
        println!("🔧 Handling missing module: {} for process {}", module_name, pid);
        
        // Create error record
        let error = ModuleError {
            module_name: module_name.clone(),
            error_message: format!("Missing module detected in process {}", pid),
            process_pid: pid.as_u32(),
            timestamp: SystemTime::now(),
            resolved: false,
        };

        // Log the error
        self.log_module_error(&error).await?;

        // Trigger auto-installation (this would be handled by the daemon)
        println!("📦 Auto-installing module: {}", module_name);
        
        Ok(())
    }

    async fn log_module_error(&self, error: &ModuleError) -> Result<()> {
        let log_entry = format!(
            "[{}] PID: {} - Missing module: {} - Error: {}\n",
            DateTime::now().to_string(),
            error.process_pid,
            error.module_name,
            error.error_message
        );

        // Write to log file
        let log_path = "/tmp/snakepit-errors.log";
        fs::write(log_path, log_entry).await?;

        Ok(())
    }

    pub async fn get_process_stats(&self) -> Result<ProcessStats> {
        let python_processes = self.python_processes.read().await;
        let system = self.system.read().await;

        let total_processes = system.processes().len();
        let python_count = python_processes.len();
        let error_count: u32 = python_processes.values().map(|p| p.error_count).sum();

        Ok(ProcessStats {
            total_processes,
            python_processes: python_count,
            error_count,
            monitored_processes: python_processes.len(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ProcessStats {
    pub total_processes: usize,
    pub python_processes: usize,
    pub error_count: u32,
    pub monitored_processes: usize,
}

// Advanced process monitoring using strace (Linux only)
pub struct StraceMonitor {
    strace_path: String,
    active_traces: Arc<RwLock<HashMap<Pid, std::process::Child>>>,
}

impl StraceMonitor {
    pub fn new() -> Self {
        Self {
            strace_path: "strace".to_string(),
            active_traces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_tracing(&self, pid: Pid) -> Result<()> {
        let mut cmd = Command::new(&self.strace_path);
        cmd.arg("-p").arg(pid.to_string());
        cmd.arg("-e").arg("openat,open");
        cmd.arg("-f");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn()?;
        
        {
            let mut active_traces = self.active_traces.write().await;
            active_traces.insert(pid, child);
        }

        Ok(())
    }

    pub async fn stop_tracing(&self, pid: Pid) -> Result<()> {
        let mut active_traces = self.active_traces.write().await;
        if let Some(mut child) = active_traces.remove(&pid) {
            let _ = child.kill();
        }
        Ok(())
    }

    pub async fn parse_strace_output(&self, pid: Pid) -> Result<Vec<String>> {
        let active_traces = self.active_traces.read().await;
        if let Some(_child) = active_traces.get(&pid) {
            // Parse strace output to find Python import attempts
            // This would involve parsing the strace output for file access patterns
            // that indicate Python module imports
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;



    #[tokio::test]
    async fn test_process_monitor_creation() {
        let monitor = ProcessMonitor::new();
        let stats = monitor.get_process_stats().await.unwrap();
        assert!(stats.total_processes > 0);
    }
}
