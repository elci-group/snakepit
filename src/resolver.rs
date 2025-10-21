use crate::dependency::{Dependency, ProjectDependencies};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use semver::{Version, VersionReq};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyPIPackageInfo {
    pub info: PyPIInfo,
    pub releases: HashMap<String, Vec<PyPIRelease>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyPIInfo {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub home_page: Option<String>,
    pub requires_dist: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyPIRelease {
    pub filename: String,
    pub url: String,
    pub size: Option<u64>,
    pub upload_time: Option<String>,
}

pub struct DependencyResolver {
    client: Client,
    cache: HashMap<String, PyPIPackageInfo>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            cache: HashMap::new(),
        }
    }

    pub async fn resolve_dependencies(&mut self, project: &ProjectDependencies) -> Result<ResolvedDependencies> {
        let mut resolved = ResolvedDependencies::new();
        
        // Resolve main dependencies
        for dep in &project.dependencies {
            let resolved_dep = self.resolve_single_dependency(dep).await?;
            resolved.dependencies.push(resolved_dep);
        }
        
        // Resolve dev dependencies
        for dep in &project.dev_dependencies {
            let mut resolved_dep = self.resolve_single_dependency(dep).await?;
            resolved_dep.is_dev = true;
            resolved.dev_dependencies.push(resolved_dep);
        }
        
        Ok(resolved)
    }

    async fn resolve_single_dependency(&mut self, dep: &Dependency) -> Result<ResolvedDependency> {
        let package_info = self.fetch_package_info(&dep.name).await?;
        
        let version = if let Some(requested_version) = &dep.version {
            Self::find_best_version_static(&package_info, requested_version, &dep.version_constraint)?
        } else {
            package_info.info.version.clone()
        };
        
        let resolved_dep = ResolvedDependency {
            name: dep.name.clone(),
            version,
            is_dev: dep.is_dev,
            dependencies: Vec::new(),
            source: dep.source.clone(),
        };
        
        // For now, skip sub-dependencies to avoid recursion
        // TODO: Implement proper dependency resolution without recursion
        
        Ok(resolved_dep)
    }

    async fn fetch_package_info(&mut self, package_name: &str) -> Result<&PyPIPackageInfo> {
        if !self.cache.contains_key(package_name) {
            let url = format!("https://pypi.org/pypi/{}/json", package_name);
            let response = self.client.get(&url).send().await?;
            
            if response.status().is_success() {
                let package_info: PyPIPackageInfo = response.json().await?;
                self.cache.insert(package_name.to_string(), package_info);
            } else {
                return Err(anyhow::anyhow!("Package {} not found on PyPI", package_name));
            }
        }
        
        Ok(self.cache.get(package_name).unwrap())
    }

    fn find_best_version_static(package_info: &PyPIPackageInfo, requested_version: &str, constraint: &Option<String>) -> Result<String> {
        let available_versions: Vec<&String> = package_info.releases.keys().collect();
        
        if let Some(constraint) = constraint {
            match constraint.as_str() {
                ">=" => {
                    // Find the latest version that satisfies >= constraint
                    let req = VersionReq::parse(&format!(">={}", requested_version))?;
                    let mut best_version = None;
                    
                    for version_str in available_versions {
                        if let Ok(version) = Version::parse(version_str) {
                            if req.matches(&version) {
                                if let Some(ref current_best) = best_version {
                                    if version > *current_best {
                                        best_version = Some(version);
                                    }
                                } else {
                                    best_version = Some(version);
                                }
                            }
                        }
                    }
                    
                    return Ok(best_version.map(|v| v.to_string()).unwrap_or_else(|| requested_version.to_string()));
                }
                "==" => {
                    // Exact version match
                    if available_versions.contains(&&requested_version.to_string()) {
                        return Ok(requested_version.to_string());
                    } else {
                        return Err(anyhow::anyhow!("Exact version {} not found", requested_version));
                    }
                }
                _ => {
                    // Default to latest version
                    return Ok(package_info.info.version.clone());
                }
            }
        } else {
            // No constraint, return latest
            Ok(package_info.info.version.clone())
        }
    }

    fn parse_requirement_string_static(req: &str) -> Option<Dependency> {
        // Simple parsing for now - can be enhanced
        let parts: Vec<&str> = req.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        
        let name = parts[0].to_string();
        let version = if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        };
        
        Some(Dependency {
            name,
            version,
            version_constraint: None,
            is_dev: false,
            source: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedDependencies {
    pub dependencies: Vec<ResolvedDependency>,
    pub dev_dependencies: Vec<ResolvedDependency>,
}

impl ResolvedDependencies {
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
            dev_dependencies: Vec::new(),
        }
    }

    pub fn to_requirements_txt(&self) -> String {
        let mut output = String::new();
        
        output.push_str("# Resolved dependencies\n");
        for dep in &self.dependencies {
            output.push_str(&format!("{}=={}\n", dep.name, dep.version));
        }
        
        if !self.dev_dependencies.is_empty() {
            output.push_str("\n# Development dependencies\n");
            for dep in &self.dev_dependencies {
                output.push_str(&format!("{}=={}\n", dep.name, dep.version));
            }
        }
        
        output
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub is_dev: bool,
    pub dependencies: Vec<ResolvedDependency>,
    pub source: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_requirement_string() {
        let resolver = DependencyResolver::new();
        let dep = resolver.parse_requirement_string("requests>=2.25.0").unwrap();
        assert_eq!(dep.name, "requests");
    }
}
