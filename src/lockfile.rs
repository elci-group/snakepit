use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use crate::pep440::Version;

#[derive(Debug, Serialize, Deserialize)]
pub struct Lockfile {
    pub metadata: LockfileMetadata,
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockfileMetadata {
    pub version: String,
    pub generator: String,
    pub timestamp: u64,
    pub python_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub hashes: Vec<String>, // SHA256 hashes
    pub source: PackageSource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum PackageSource {
    PyPI { url: String },
    Git { url: String, rev: String },
    Path { path: String },
}

impl Lockfile {
    pub fn new() -> Self {
        Self {
            metadata: LockfileMetadata {
                version: "1.0".to_string(),
                generator: "snakepit".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                python_version: None,
            },
            packages: Vec::new(),
        }
    }

    pub fn add_package(&mut self, package: LockedPackage) {
        self.packages.push(package);
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string).await?;
        Ok(())
    }

    pub async fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let lockfile: Lockfile = toml::from_str(&content)?;
        Ok(lockfile)
    }

    pub fn verify_integrity(&self) -> bool {
        // Check if all packages have at least one hash
        for package in &self.packages {
            if package.hashes.is_empty() {
                return false;
            }
        }
        true
    }

    pub fn from_resolved(resolved: &HashMap<String, Version>) -> Self {
        let mut lockfile = Lockfile::new();
        
        for (name, version) in resolved {
            let locked = LockedPackage {
                name: name.clone(),
                version: version.to_string(),
                dependencies: Vec::new(), // TODO: Extract from solver
                hashes: Vec::new(), // TODO: Fetch from PyPI
                source: PackageSource::PyPI {
                    url: format!("https://pypi.org/simple/{}/", name),
                },
            };
            lockfile.add_package(locked);
        }
        
        lockfile
    }
}

pub struct LockfileGenerator {
    resolver: crate::resolver::DependencyResolver,
}

impl LockfileGenerator {
    pub fn new() -> Self {
        Self {
            resolver: crate::resolver::DependencyResolver::new(),
        }
    }

    pub async fn generate_from_solver(
        &mut self,
        resolved: &HashMap<String, Version>,
    ) -> Result<Lockfile> {
        let mut lockfile = Lockfile::new();

        for (name, version) in resolved {
            let locked = self.create_locked_package(name, version).await?;
            lockfile.add_package(locked);
        }

        Ok(lockfile)
    }

    async fn create_locked_package(
        &mut self,
        name: &str,
        version: &Version,
    ) -> Result<LockedPackage> {
        // Fetch package info to get hashes
        let info = self.resolver.fetch_package_info(name).await?;
        
        let version_str = version.to_string();
        let mut hashes = Vec::new();
        let mut dependencies = Vec::new();

        // Find the release for this version
        if let Some(releases) = info.releases.get(&version_str) {
            for release in releases {
                if let Some(digests) = &release.digests {
                    if let Some(sha256) = digests.get("sha256") {
                        hashes.push(format!("sha256:{}", sha256));
                    }
                }
            }
        }

        // Extract dependencies
        if let Some(requires) = &info.info.requires_dist {
            for req_str in requires {
                if let Ok(spec) = crate::markers::parse_requirement(req_str) {
                    dependencies.push(format!("{}=={}", spec.name, version_str));
                }
            }
        }

        Ok(LockedPackage {
            name: name.to_string(),
            version: version_str,
            dependencies,
            hashes,
            source: PackageSource::PyPI {
                url: format!("https://pypi.org/simple/{}/", name),
            },
        })
    }
}
