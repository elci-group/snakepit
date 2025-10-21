use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::Result;
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnakepitConfig {
    pub default_backend: Option<String>,
    pub default_venv_backend: Option<String>,
    pub venv_path: Option<String>,
    pub cache_enabled: Option<bool>,
    pub python_version: Option<String>,
    pub mirrors: Option<Vec<String>>,
    pub timeout: Option<u64>,
    pub retries: Option<u32>,
    pub user_agent: Option<String>,
}

impl Default for SnakepitConfig {
    fn default() -> Self {
        Self {
            default_backend: Some("pip".to_string()),
            default_venv_backend: Some("venv".to_string()),
            venv_path: None,
            cache_enabled: Some(true),
            python_version: None,
            mirrors: None,
            timeout: Some(30),
            retries: Some(3),
            user_agent: Some("snakepit/0.1.0".to_string()),
        }
    }
}

impl SnakepitConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: SnakepitConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }

    pub fn get_config_path() -> Result<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            Ok(config_dir.join("snakepit").join("config.toml"))
        } else {
            Ok(PathBuf::from(".snakepit").join("config.toml"))
        }
    }

    pub fn get_venv_path(&self) -> PathBuf {
        if let Some(path) = &self.venv_path {
            PathBuf::from(path)
        } else {
            if let Some(home) = dirs::home_dir() {
                home.join(".snakepit").join("venvs")
            } else {
                PathBuf::from(".snakepit").join("venvs")
            }
        }
    }

    pub fn get_cache_path(&self) -> PathBuf {
        if let Some(cache_dir) = dirs::cache_dir() {
            cache_dir.join("snakepit")
        } else {
            PathBuf::from(".snakepit").join("cache")
        }
    }

    pub fn with_backend(mut self, backend: &str) -> Self {
        self.default_backend = Some(backend.to_string());
        self
    }

    pub fn with_venv_backend(mut self, backend: &str) -> Self {
        self.default_venv_backend = Some(backend.to_string());
        self
    }

    pub fn with_venv_path(mut self, path: &str) -> Self {
        self.venv_path = Some(path.to_string());
        self
    }

    pub fn with_python_version(mut self, version: &str) -> Self {
        self.python_version = Some(version.to_string());
        self
    }

    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = Some(enabled);
        self
    }

    pub fn with_mirrors(mut self, mirrors: Vec<String>) -> Self {
        self.mirrors = Some(mirrors);
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = Some(retries);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub python_version: Option<String>,
    pub backend: Option<String>,
    pub venv_name: Option<String>,
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
    pub scripts: Option<std::collections::HashMap<String, String>>,
}

impl ProjectConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: None,
            description: None,
            python_version: None,
            backend: None,
            venv_name: None,
            dependencies: Vec::new(),
            dev_dependencies: Vec::new(),
            scripts: None,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_python_version(mut self, version: &str) -> Self {
        self.python_version = Some(version.to_string());
        self
    }

    pub fn with_backend(mut self, backend: &str) -> Self {
        self.backend = Some(backend.to_string());
        self
    }

    pub fn with_venv_name(mut self, name: &str) -> Self {
        self.venv_name = Some(name.to_string());
        self
    }

    pub fn add_dependency(mut self, dependency: &str) -> Self {
        self.dependencies.push(dependency.to_string());
        self
    }

    pub fn add_dev_dependency(mut self, dependency: &str) -> Self {
        self.dev_dependencies.push(dependency.to_string());
        self
    }

    pub fn add_script(mut self, name: &str, command: &str) -> Self {
        if self.scripts.is_none() {
            self.scripts = Some(std::collections::HashMap::new());
        }
        if let Some(ref mut scripts) = self.scripts {
            scripts.insert(name.to_string(), command.to_string());
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SnakepitConfig::default();
        assert_eq!(config.default_backend, Some("pip".to_string()));
        assert_eq!(config.cache_enabled, Some(true));
    }

    #[test]
    fn test_project_config() {
        let config = ProjectConfig::new("test-project".to_string());
        assert_eq!(config.name, "test-project");
        assert!(config.dependencies.is_empty());
    }

    #[test]
    fn test_config_builder() {
        let config = SnakepitConfig::new()
            .with_backend("conda")
            .with_python_version("3.9")
            .with_cache(false);
        
        assert_eq!(config.default_backend, Some("conda".to_string()));
        assert_eq!(config.python_version, Some("3.9".to_string()));
        assert_eq!(config.cache_enabled, Some(false));
    }
}
