use crate::dependency::{Dependency, ProjectDependencies};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use semver::{Version, VersionReq};
use std::path::PathBuf;
use std::fs;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use crate::native::dirs;

#[derive(Clone)]
struct DiskCache {
    root: PathBuf,
}

impl DiskCache {
    fn new() -> Self {
        let root = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".snakepit_cache"))
            .join("pypi");
        fs::create_dir_all(&root).ok();
        Self { root }
    }

    fn get(&self, package: &str) -> Option<PyPIPackageInfo> {
        let path = self.root.join(format!("{}.json", package));
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(info) = serde_json::from_str(&content) {
                    return Some(info);
                }
            }
        }
        None
    }

    fn set(&self, package: &str, info: &PyPIPackageInfo) {
        let path = self.root.join(format!("{}.json", package));
        if let Ok(content) = serde_json::to_string(info) {
            let _ = fs::write(path, content);
        }
    }
}

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
    pub digests: Option<HashMap<String, String>>,
}

pub struct DependencyResolver {
    client: Client,
    cache: DiskCache,
    mem_cache: Arc<Mutex<HashMap<String, PyPIPackageInfo>>>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            cache: DiskCache::new(),
            mem_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn resolve_dependencies(&mut self, project: &ProjectDependencies) -> Result<ResolvedDependencies> {
        let mut resolved = ResolvedDependencies::new();
        let mut visited = HashSet::new();
        
        // Resolve main dependencies
        for dep in &project.dependencies {
            let resolved_dep = self.resolve_recursive(dep, &mut visited).await?;
            resolved.dependencies.push(resolved_dep);
        }
        
        // Resolve dev dependencies
        for dep in &project.dev_dependencies {
            let mut resolved_dep = self.resolve_recursive(dep, &mut visited).await?;
            resolved_dep.is_dev = true;
            resolved.dev_dependencies.push(resolved_dep);
        }
        
        Ok(resolved)
    }

    fn resolve_recursive<'a>(
        &'a self, 
        dep: &'a Dependency, 
        visited: &'a mut HashSet<String>
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedDependency>> + Send + 'a>> {
        Box::pin(async move {
            // Check for cycles
            if visited.contains(&dep.name) {
                // Return a placeholder or error? 
                // For now, we'll just return the dependency without children to break cycle
                // But we need version info.
                // If we visited it, we assume it's handled up the stack or elsewhere.
                // But we need to return a ResolvedDependency.
                // Let's just fetch info but skip children.
            }
            visited.insert(dep.name.clone());

            let package_info = self.fetch_package_info(&dep.name).await?;
            
            let version = if let Some(requested_version) = &dep.version {
                Self::find_best_version_static(&package_info, requested_version, &dep.version_constraint)?
            } else {
                package_info.info.version.clone()
            };
            
            let mut resolved_dep = ResolvedDependency {
                name: dep.name.clone(),
                version: version.clone(),
                is_dev: dep.is_dev,
                dependencies: Vec::new(),
                source: dep.source.clone(),
            };
            
            // Resolve sub-dependencies
            if let Some(requires) = &package_info.info.requires_dist {
                for req_str in requires {
                    // Filter out extra markers for now (simplified)
                    if req_str.contains("extra ==") {
                        continue;
                    }
                    
                    if let Some(sub_dep) = Self::parse_requirement_string_static(req_str) {
                        if !visited.contains(&sub_dep.name) {
                            let mut sub_visited = visited.clone();
                            if let Ok(sub_resolved) = self.resolve_recursive(&sub_dep, &mut sub_visited).await {
                                resolved_dep.dependencies.push(sub_resolved);
                            }
                        }
                    }
                }
            }
            
            Ok(resolved_dep)
        })
    }

    // Kept for backward compatibility if needed, but redirects to recursive
    async fn resolve_single_dependency(&self, dep: &Dependency) -> Result<ResolvedDependency> {
        let mut visited = HashSet::new();
        self.resolve_recursive(dep, &mut visited).await
    }

    pub async fn fetch_package_info(&self, package_name: &str) -> Result<PyPIPackageInfo> {
        // Check memory cache
        {
            let cache = self.mem_cache.lock().unwrap();
            if let Some(info) = cache.get(package_name) {
                return Ok(info.clone());
            }
        }

        // Check disk cache
        if let Some(info) = self.cache.get(package_name) {
            let mut cache = self.mem_cache.lock().unwrap();
            cache.insert(package_name.to_string(), info.clone());
            return Ok(info);
        }

        // Fetch from network
        let url = format!("https://pypi.org/pypi/{}/json", package_name);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let package_info: PyPIPackageInfo = response.json().await?;
            
            // Update caches
            self.cache.set(package_name, &package_info);
            {
                let mut cache = self.mem_cache.lock().unwrap();
                cache.insert(package_name.to_string(), package_info.clone());
            }
            
            Ok(package_info)
        } else {
            Err(anyhow::anyhow!("Package {} not found on PyPI", package_name))
        }
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


}
