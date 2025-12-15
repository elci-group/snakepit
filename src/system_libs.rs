use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLibrary {
    pub name: String,
    pub package_name: String,
    pub os: String,
}

pub struct SystemLibDetector {
    library_map: HashMap<String, HashMap<String, String>>,
}

impl SystemLibDetector {
    pub fn new() -> Self {
        let mut library_map = HashMap::new();
        
        // PostgreSQL
        let mut libpq = HashMap::new();
        libpq.insert("ubuntu".to_string(), "libpq-dev".to_string());
        libpq.insert("debian".to_string(), "libpq-dev".to_string());
        libpq.insert("fedora".to_string(), "postgresql-devel".to_string());
        libpq.insert("arch".to_string(), "postgresql-libs".to_string());
        libpq.insert("macos".to_string(), "postgresql".to_string());
        library_map.insert("libpq.so".to_string(), libpq.clone());
        library_map.insert("libpq.so.5".to_string(), libpq);
        
        // OpenSSL
        let mut libssl = HashMap::new();
        libssl.insert("ubuntu".to_string(), "libssl-dev".to_string());
        libssl.insert("debian".to_string(), "libssl-dev".to_string());
        libssl.insert("fedora".to_string(), "openssl-devel".to_string());
        libssl.insert("arch".to_string(), "openssl".to_string());
        libssl.insert("macos".to_string(), "openssl".to_string());
        library_map.insert("libssl.so".to_string(), libssl.clone());
        library_map.insert("libssl.so.1.1".to_string(), libssl.clone());
        library_map.insert("libssl.so.3".to_string(), libssl);
        
        // MySQL
        let mut libmysql = HashMap::new();
        libmysql.insert("ubuntu".to_string(), "libmysqlclient-dev".to_string());
        libmysql.insert("debian".to_string(), "libmysqlclient-dev".to_string());
        libmysql.insert("fedora".to_string(), "mysql-devel".to_string());
        libmysql.insert("arch".to_string(), "mariadb-libs".to_string());
        libmysql.insert("macos".to_string(), "mysql".to_string());
        library_map.insert("libmysqlclient.so".to_string(), libmysql);
        
        // SQLite
        let mut libsqlite = HashMap::new();
        libsqlite.insert("ubuntu".to_string(), "libsqlite3-dev".to_string());
        libsqlite.insert("debian".to_string(), "libsqlite3-dev".to_string());
        libsqlite.insert("fedora".to_string(), "sqlite-devel".to_string());
        libsqlite.insert("arch".to_string(), "sqlite".to_string());
        libsqlite.insert("macos".to_string(), "sqlite".to_string());
        library_map.insert("libsqlite3.so".to_string(), libsqlite);
        
        Self { library_map }
    }

    pub fn detect_os(&self) -> String {
        if cfg!(target_os = "macos") {
            return "macos".to_string();
        }
        
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("ID=") {
                    let id = line.replace("ID=", "").replace("\"", "").trim().to_string();
                    return id;
                }
            }
        }
        
        "unknown".to_string()
    }

    pub fn find_package(&self, library: &str) -> Option<SystemLibrary> {
        let os = self.detect_os();
        
        // Try exact match first
        if let Some(packages) = self.library_map.get(library) {
            if let Some(package_name) = packages.get(&os) {
                return Some(SystemLibrary {
                    name: library.to_string(),
                    package_name: package_name.clone(),
                    os: os.clone(),
                });
            }
        }
        
        // Try without version suffix (e.g., libpq.so.5 -> libpq.so)
        let base_name = library.split('.').take(2).collect::<Vec<_>>().join(".");
        if let Some(packages) = self.library_map.get(&base_name) {
            if let Some(package_name) = packages.get(&os) {
                return Some(SystemLibrary {
                    name: library.to_string(),
                    package_name: package_name.clone(),
                    os: os.clone(),
                });
            }
        }
        
        None
    }

    pub fn get_install_command(&self, library: &SystemLibrary) -> Option<String> {
        match library.os.as_str() {
            "ubuntu" | "debian" => Some(format!("sudo apt install {}", library.package_name)),
            "fedora" => Some(format!("sudo dnf install {}", library.package_name)),
            "arch" => Some(format!("sudo pacman -S {}", library.package_name)),
            "macos" => Some(format!("brew install {}", library.package_name)),
            _ => None,
        }
    }

    pub fn extract_library_from_error(&self, error: &str) -> Option<String> {
        // Common patterns:
        // "libpq.so.5: cannot open shared object file"
        // "ImportError: libssl.so.1.1: cannot open shared object file"
        // "OSError: libmysqlclient.so: cannot open shared object file"
        
        for line in error.lines() {
            if line.contains("cannot open shared object file") || line.contains("No such file or directory") {
                // Extract library name
                for word in line.split_whitespace() {
                    if word.starts_with("lib") && (word.ends_with(".so") || word.contains(".so.")) {
                        let lib = word.trim_end_matches(':').trim_end_matches(',').to_string();
                        return Some(lib);
                    }
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_library() {
        let detector = SystemLibDetector::new();
        let error = "ImportError: libpq.so.5: cannot open shared object file: No such file or directory";
        assert_eq!(detector.extract_library_from_error(error), Some("libpq.so.5".to_string()));
    }

    #[test]
    fn test_find_package() {
        let detector = SystemLibDetector::new();
        if let Some(lib) = detector.find_package("libpq.so.5") {
            assert!(!lib.package_name.is_empty());
        }
    }
}
