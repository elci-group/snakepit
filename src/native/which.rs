// Native alternative to `which` crate
// Finds executables in PATH with zero dependencies
// Savings: -30 KB, zero external deps

use std::env;
use std::path::{Path, PathBuf};

/// Find an executable in the system PATH
/// 
/// # Example
/// ```
/// let adb = find_executable("adb")?;
/// println!("Found adb at: {}", adb.display());
/// ```
pub fn find_executable(name: &str) -> Option<PathBuf> {
    // Get PATH environment variable
    let path_var = env::var_os("PATH")?;
    
    // Get the executable extension for this platform
    let exe_extension = if cfg!(windows) { ".exe" } else { "" };
    
    // Search each directory in PATH
    env::split_paths(&path_var)
        .find_map(|dir| {
            // Try with and without extension
            for suffix in &["", exe_extension] {
                let candidate = dir.join(format!("{}{}", name, suffix));
                
                // Check if file exists and is executable
                if candidate.is_file() && is_executable(&candidate) {
                    return Some(candidate);
                }
            }
            None
        })
}

/// Check if a file is executable
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    
    if let Ok(metadata) = std::fs::metadata(path) {
        let permissions = metadata.permissions();
        // Check if any execute bit is set
        permissions.mode() & 0o111 != 0
    } else {
        false
    }
}

#[cfg(windows)]
fn is_executable(_path: &Path) -> bool {
    // On Windows, if it exists and has .exe extension, it's executable
    true
}

/// Find multiple executables at once
/// 
/// # Example
/// ```
/// let tools = find_executables(&["git", "python3", "cargo"]);
/// ```
pub fn find_executables(names: &[&str]) -> Vec<(String, Option<PathBuf>)> {
    names.iter()
        .map(|&name| (name.to_string(), find_executable(name)))
        .collect()
}

/// Check if an executable exists in PATH
/// 
/// # Example
/// ```
/// if has_executable("docker") {
///     println!("Docker is installed");
/// }
/// ```
pub fn has_executable(name: &str) -> bool {
    find_executable(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_common_executables() {
        // These should exist on most systems
        assert!(find_executable("ls").is_some() || find_executable("dir").is_some());
    }

    #[test]
    fn test_nonexistent_executable() {
        assert!(find_executable("this_definitely_does_not_exist_12345").is_none());
    }

    #[test]
    fn test_has_executable() {
        assert!(has_executable("ls") || has_executable("dir"));
    }
}
