use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub version_constraint: Option<String>,
    pub is_dev: bool,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDependencies {
    pub dependencies: Vec<Dependency>,
    pub dev_dependencies: Vec<Dependency>,
    pub python_version: Option<String>,
    pub project_name: Option<String>,
}

impl ProjectDependencies {
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
            dev_dependencies: Vec::new(),
            python_version: None,
            project_name: None,
        }
    }

    pub fn from_requirements_txt<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut deps = Self::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some(dep) = Self::parse_requirement_line(line) {
                if dep.is_dev {
                    deps.dev_dependencies.push(dep);
                } else {
                    deps.dependencies.push(dep);
                }
            }
        }
        
        Ok(deps)
    }

    pub fn from_pyproject_toml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let toml: toml::Value = toml::from_str(&content)?;
        
        let mut deps = Self::new();
        
        if let Some(project) = toml.get("project") {
            if let Some(name) = project.get("name").and_then(|v| v.as_str()) {
                deps.project_name = Some(name.to_string());
            }
            
            if let Some(requires_python) = project.get("requires-python").and_then(|v| v.as_str()) {
                deps.python_version = Some(requires_python.to_string());
            }
            
            if let Some(dependencies) = project.get("dependencies").and_then(|v| v.as_array()) {
                for dep in dependencies {
                    if let Some(dep_str) = dep.as_str() {
                        if let Some(parsed) = Self::parse_requirement_line(dep_str) {
                            deps.dependencies.push(parsed);
                        }
                    }
                }
            }
            
            if let Some(dev_deps) = project.get("optional-dependencies") {
                if let Some(dev_group) = dev_deps.get("dev").and_then(|v| v.as_array()) {
                    for dep in dev_group {
                        if let Some(dep_str) = dep.as_str() {
                            if let Some(mut parsed) = Self::parse_requirement_line(dep_str) {
                                parsed.is_dev = true;
                                deps.dev_dependencies.push(parsed);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(deps)
    }

    fn parse_requirement_line(line: &str) -> Option<Dependency> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }

        // Handle different requirement formats manually
        // Operators to look for, longest first
        let operators = [">=", "<=", "==", "!=", "~=", ">", "<"];
        
        for op in &operators {
            if let Some(idx) = line.find(op) {
                let name = line[..idx].trim().to_string();
                let constraint = op.to_string();
                let version = line[idx+op.len()..].trim().to_string();
                
                return Some(Dependency {
                    name,
                    version: Some(version),
                    version_constraint: Some(constraint),
                    is_dev: false,
                    source: None,
                });
            }
        }

        // Simple package name without version
        Some(Dependency {
            name: line.to_string(),
            version: None,
            version_constraint: None,
            is_dev: false,
            source: None,
        })
    }

    pub fn to_requirements_txt(&self) -> String {
        let mut output = String::new();
        
        if let Some(project_name) = &self.project_name {
            output.push_str(&format!("# Project: {}\n", project_name));
        }
        
        if let Some(python_version) = &self.python_version {
            output.push_str(&format!("# Python version: {}\n", python_version));
        }
        
        output.push_str("\n# Dependencies\n");
        for dep in &self.dependencies {
            output.push_str(&format!("{}\n", self.format_dependency(dep)));
        }
        
        if !self.dev_dependencies.is_empty() {
            output.push_str("\n# Development dependencies\n");
            for dep in &self.dev_dependencies {
                output.push_str(&format!("{}\n", self.format_dependency(dep)));
            }
        }
        
        output
    }

    fn format_dependency(&self, dep: &Dependency) -> String {
        let mut formatted = dep.name.clone();
        
        if let (Some(constraint), Some(version)) = (&dep.version_constraint, &dep.version) {
            formatted.push_str(&format!("{}{}", constraint, version));
        }
        
        if let Some(source) = &dep.source {
            formatted.push_str(&format!(" @ {}", source));
        }
        
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_requirement_line() {
        let dep = ProjectDependencies::parse_requirement_line("requests>=2.25.0").unwrap();
        assert_eq!(dep.name, "requests");
        assert_eq!(dep.version_constraint, Some(">=".to_string()));
        assert_eq!(dep.version, Some("2.25.0".to_string()));
    }

    #[test]
    fn test_parse_simple_package() {
        let dep = ProjectDependencies::parse_requirement_line("numpy").unwrap();
        assert_eq!(dep.name, "numpy");
        assert_eq!(dep.version, None);
        assert_eq!(dep.version_constraint, None);
    }
}
