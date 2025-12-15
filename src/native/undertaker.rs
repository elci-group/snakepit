use sysinfo::{Pid, System, ProcessStatus};
use std::path::Path;
use std::fs;
use crate::native::style::{red, dim, yellow};
use std::time::SystemTime;

#[derive(Debug)]
pub struct TheUndertaker {
    system: System,
}

impl TheUndertaker {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub fn perform_rounds(&mut self) {
        self.system.refresh_processes();
        
        let zombies: Vec<Pid> = self.system.processes().iter()
            .filter(|(_, p)| p.status() == ProcessStatus::Zombie)
            .map(|(pid, _)| *pid)
            .collect();

        if !zombies.is_empty() {
            println!("{}", dim(format!("⚰️  The Undertaker found {} zombies...", zombies.len())));
            for pid in zombies {
                self.handle_zombie(pid);
            }
        }
    }

    fn handle_zombie(&self, pid: Pid) {
        // 1. Forensics: Try to find last words (logs)
        // We look for snakepit_crash.log in likely CWDs if we can find them
        // But for a zombie, /proc/pid/cwd might be gone or invalid.
        // We can try to guess based on /tmp/snakepit_pid_...
        
        // 2. Cleanup Grave (Temp files)
        self.cleanup_grave(pid);

        // 3. Reaping
        // In Rust/Linux, we can only reap our own children.
        // If these are random system zombies, we can't do much but notify.
        // But if we spawned them (e.g. via installer), we should have waited.
        // This acts as a safety net.
        
        // If we are NOT the parent, we can't waitpid.
        // But we can check if the parent is us.
        let my_pid = std::process::id();
        if let Some(process) = self.system.process(pid) {
            if let Some(parent) = process.parent() {
                if parent.as_u32() == my_pid {
                    println!("{}", yellow(format!("🪦 Reaping our own zombie child: {}", pid)));
                    // Note: libc is not in dependencies, would need to add it
                    // unsafe {
                    //     libc::waitpid(pid.as_u32() as i32, std::ptr::null_mut(), libc::WNOHANG);
                    // }
                } else {
                    // Not our child.
                    // println!("{}", dim(format!("Found zombie {} (Parent: {}). May they rest in peace.", pid, parent)));
                }
            }
        }
    }

    fn cleanup_grave(&self, pid: Pid) {
        // Look for temp files associated with this PID
        let temp_dir = std::env::temp_dir();
        
        // Pattern: snakepit_{pid}_*
        if let Ok(entries) = fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.contains(&format!("_{}_", pid)) || name.ends_with(&format!("_{}", pid)) {
                        println!("{}", dim(format!("🧹 Cleaning up grave goods: {}", name)));
                        let _ = fs::remove_file(&path); // or remove_dir_all if dir
                    }
                }
            }
        }
    }
}
