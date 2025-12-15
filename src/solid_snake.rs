use anyhow::Result;
use std::process::Command;
use serde::{Serialize, Deserialize};

use crate::native::which;

pub struct SolidSnakeEngine {
    adb_path: String,
    connected_devices: Vec<AndroidDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidDevice {
    pub id: String,
    pub name: String,
    pub connection_type: ConnectionType,
    pub python_version: Option<String>,
    pub termux_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    USB,
    WiFi(String), // IP address
    Bluetooth(String), // MAC address
}

#[derive(Debug)]
pub struct TestResults {
    pub passed: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub struct PerformanceMetrics {
    pub total_time: f64,
    pub function_calls: usize,
    pub profile_data: String,
}

impl SolidSnakeEngine {
    pub fn new() -> Result<Self> {
        println!("🎮 Solid Snake Engine initializing...");
        
        // Find adb in PATH
        let adb_path = which::find_executable("adb")
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                println!("⚠️  ADB not found in PATH, using 'adb'");
                "adb".to_string()
            });
        
        println!("📱 ADB path: {}", adb_path);
        
        Ok(Self {
            adb_path,
            connected_devices: vec![],
        })
    }
    
    pub async fn discover_devices(&mut self) -> Result<Vec<AndroidDevice>> {
        println!("🔍 Discovering Android devices...");
        
        // List devices via ADB
        let output = Command::new(&self.adb_path)
            .args(&["devices", "-l"])
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("ADB command failed: {}", error));
        }
        
        let devices_str = String::from_utf8_lossy(&output.stdout);
        let mut devices = vec![];
        
        for line in devices_str.lines().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "device" {
                let id = parts[0].to_string();
                match self.probe_device(&id).await {
                    Ok(device) => devices.push(device),
                    Err(e) => {
                        println!("⚠️  Failed to probe device {}: {}", id, e);
                    }
                }
            }
        }
        
        self.connected_devices = devices.clone();
        
        if devices.is_empty() {
            println!("❌ No devices found");
            println!("💡 Make sure:");
            println!("   • USB debugging is enabled");
            println!("   • Device is connected via USB or WiFi");
            println!("   • ADB is installed and in PATH");
        } else {
            println!("✅ Found {} device(s)", devices.len());
            for dev in &devices {
                let conn_type = match &dev.connection_type {
                    ConnectionType::USB => "USB",
                    ConnectionType::WiFi(_) => "WiFi",
                    ConnectionType::Bluetooth(_) => "Bluetooth",
                };
                println!("   📱 {} ({}) - {}", dev.name, dev.id, conn_type);
                if let Some(py_ver) = &dev.python_version {
                    println!("      🐍 Python: {}", py_ver);
                } else if !dev.termux_installed {
                    println!("      ⚠️  Termux not installed");
                }
            }
        }
        
        Ok(devices)
    }
    
    async fn probe_device(&self, device_id: &str) -> Result<AndroidDevice> {
        // Get device model
        let output = Command::new(&self.adb_path)
            .args(&["-s", device_id, "shell", "getprop", "ro.product.model"])
            .output()?;
        
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Check if Termux is installed
        let termux_check = Command::new(&self.adb_path)
            .args(&["-s", device_id, "shell", "pm", "list", "packages", "com.termux"])
            .output()?;
        
        let termux_installed = String::from_utf8_lossy(&termux_check.stdout)
            .contains("com.termux");
        
        // Check Python version (if Termux installed)
        let python_version = if termux_installed {
            let py_output = Command::new(&self.adb_path)
                .args(&["-s", device_id, "shell", "python3", "--version"])
                .output();
            
            if let Ok(output) = py_output {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Determine connection type
        let connection_type = if device_id.contains(":") {
            ConnectionType::WiFi(device_id.to_string())
        } else {
            ConnectionType::USB
        };
        
        Ok(AndroidDevice {
            id: device_id.to_string(),
            name,
            connection_type,
            python_version,
            termux_installed,
        })
    }
    
    pub async fn connect_wifi(&self, ip_address: &str, port: u16) -> Result<()> {
        println!("📡 Connecting to {}:{}...", ip_address, port);
        
        // First, enable tcpip mode on USB-connected device
        println!("   Enabling TCP/IP mode...");
        let output = Command::new(&self.adb_path)
            .args(&["tcpip", &port.to_string()])
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to enable TCP/IP mode: {}", error));
        }
        
        // Wait for device to restart in TCP/IP mode
        println!("   Waiting for device to restart...");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Connect via WiFi
        println!("   Connecting...");
        let output = Command::new(&self.adb_path)
            .args(&["connect", &format!("{}:{}", ip_address, port)])
            .output()?;
        
        let result = String::from_utf8_lossy(&output.stdout);
        
        if result.contains("connected") {
            println!("✅ Connected via WiFi to {}:{}", ip_address, port);
            println!("💡 You can now disconnect USB cable");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to connect: {}", result))
        }
    }
    
    pub async fn disconnect_wifi(&self, ip_address: &str, port: u16) -> Result<()> {
        println!("🔌 Disconnecting from {}:{}...", ip_address, port);
        
        let output = Command::new(&self.adb_path)
            .args(&["disconnect", &format!("{}:{}", ip_address, port)])
            .output()?;
        
        if output.status.success() {
            println!("✅ Disconnected");
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to disconnect: {}", error))
        }
    }
    
    pub async fn install_package(&self, device_id: &str, package: &str) -> Result<()> {
        println!("📦 Installing {} on device {}...", package, device_id);
        
        // Check if Termux is installed
        let device = self.connected_devices.iter()
            .find(|d| d.id == device_id)
            .ok_or_else(|| anyhow::anyhow!("Device not found: {}", device_id))?;
        
        if !device.termux_installed {
            return Err(anyhow::anyhow!(
                "Termux is not installed on this device. Please install Termux from F-Droid."
            ));
        }
        
        // Install via pip in Termux
        let output = Command::new(&self.adb_path)
            .args(&[
                "-s", device_id,
                "shell",
                "su", "-c",
                &format!("pip3 install {}", package)
            ])
            .output()?;
        
        if output.status.success() {
            println!("✅ Installed {}", package);
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Installation failed: {}", error))
        }
    }
    
    pub async fn run_tests(&self, device_id: &str, test_file: &str) -> Result<TestResults> {
        println!("🧪 Running tests on device {}...", device_id);
        
        let start = std::time::Instant::now();
        
        // Push test file to device
        println!("   Pushing test file...");
        let remote_path = "/data/local/tmp/test.py";
        Command::new(&self.adb_path)
            .args(&["-s", device_id, "push", test_file, remote_path])
            .output()?;
        
        // Run pytest
        println!("   Running tests...");
        let output = Command::new(&self.adb_path)
            .args(&[
                "-s", device_id,
                "shell",
                "python3", "-m", "pytest", remote_path, "-v"
            ])
            .output()?;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        let passed = output.status.success();
        
        if passed {
            println!("✅ All tests passed ({} ms)", duration_ms);
        } else {
            println!("❌ Some tests failed ({} ms)", duration_ms);
        }
        
        Ok(TestResults {
            passed,
            stdout,
            stderr,
            duration_ms,
        })
    }
    
    pub async fn profile_performance(&self, device_id: &str, script: &str) -> Result<PerformanceMetrics> {
        println!("⚡ Profiling performance on device {}...", device_id);
        
        // Push script to device
        let remote_path = "/data/local/tmp/profile_script.py";
        Command::new(&self.adb_path)
            .args(&["-s", device_id, "push", script, remote_path])
            .output()?;
        
        // Run with cProfile
        let output = Command::new(&self.adb_path)
            .args(&[
                "-s", device_id,
                "shell",
                "python3", "-m", "cProfile", "-s", "cumtime", remote_path
            ])
            .output()?;
        
        let profile_output = String::from_utf8_lossy(&output.stdout).to_string();
        
        // Parse metrics (simplified - would need proper parsing)
        println!("📊 Profile complete");
        
        Ok(PerformanceMetrics {
            total_time: 0.0, // Would parse from output
            function_calls: 0,
            profile_data: profile_output,
        })
    }
    
    pub async fn stream_logs(&self, device_id: &str) -> Result<()> {
        println!("📜 Streaming logs from device {}...", device_id);
        println!("   Press Ctrl+C to stop");
        
        // Stream logcat
        let mut child = Command::new(&self.adb_path)
            .args(&["-s", device_id, "logcat", "-v", "time"])
            .spawn()?;
        
        child.wait()?;
        
        Ok(())
    }
    
    pub fn get_connected_devices(&self) -> &[AndroidDevice] {
        &self.connected_devices
    }
}
