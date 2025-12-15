use sysinfo::System;
use std::path::Path;
use std::process::Command;

pub struct HardwareCapabilities {
    pub has_nvidia_gpu: bool,
    pub total_memory_gb: u64,
    pub available_memory_gb: u64,
}

impl HardwareCapabilities {
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_memory();

        let total_memory_gb = sys.total_memory() / 1024 / 1024 / 1024;
        let available_memory_gb = sys.available_memory() / 1024 / 1024 / 1024;

        let has_nvidia_gpu = Self::check_nvidia_gpu();

        Self {
            has_nvidia_gpu,
            total_memory_gb,
            available_memory_gb,
        }
    }

    fn check_nvidia_gpu() -> bool {
        // Check for device file
        if Path::new("/dev/nvidia0").exists() {
            return true;
        }

        // Check via nvidia-smi
        if let Ok(output) = Command::new("nvidia-smi").arg("-L").output() {
            if output.status.success() {
                return true;
            }
        }

        false
    }

    pub fn can_run_local_llm(&self) -> bool {
        // Basic heuristic: Needs GPU OR >16GB RAM for decent CPU inference
        // For smaller models (like tinyllama), 8GB might suffice.
        // Let's be conservative: GPU or >12GB RAM.
        
        if self.has_nvidia_gpu {
            return true;
        }

        if self.available_memory_gb >= 8 {
            return true;
        }

        false
    }
}
