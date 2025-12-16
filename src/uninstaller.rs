use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use snakegg::native::style::{red, green, yellow, blue, cyan, magenta, bold, dim};
use snakegg::native::dirs;
use snakegg::native::datetime::DateTime;
use snakegg::charmer::SnakeCharmer;
use crate::installer::{PackageInstaller, InstallerBackend};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub package: String,
    pub dependents: Vec<String>,
    pub risk_score: u8, // 0-100
    pub ai_analysis: Option<String>,
    pub breaking_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub timestamp: String,
    pub package: String,
    pub version: String,
    pub files_path: PathBuf,
}

pub struct Uninstaller {
    installer: PackageInstaller,
    charmer: Option<SnakeCharmer>,
    snapshots_dir: PathBuf,
}

impl Uninstaller {
    pub fn new() -> Result<Self> {
        let snapshots_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?
            .join("snakepit")
            .join("snapshots");

        std::fs::create_dir_all(&snapshots_dir)?;

        Ok(Self {
            installer: PackageInstaller::new(),
            charmer: SnakeCharmer::new().ok(),
            snapshots_dir,
        })
    }

    pub async fn analyze_impact(&self, package: &str) -> Result<ImpactReport> {
        println!("{}", dim(format!("🔍 Analyzing impact of removing '{}'...", package)));

        // 1. Find dependents (packages that depend on this one)
        let dependents = self.find_dependents(package).await?;
        
        let mut report = ImpactReport {
            package: package.to_string(),
            dependents: dependents.clone(),
            risk_score: if dependents.is_empty() { 10 } else { 80 },
            ai_analysis: None,
            breaking_changes: !dependents.is_empty(),
        };

        // 2. AI Analysis
        if let Some(charmer) = &self.charmer {
            println!("{}", magenta("🧠 Consulting Snake Charmer for risk prediction..."));
            if let Ok(analysis) = charmer.analyze_uninstall_risk(package, &dependents).await {
                report.ai_analysis = Some(analysis);
            }
        }

        Ok(report)
    }

    async fn find_dependents(&self, package: &str) -> Result<Vec<String>> {
        // Use Python's importlib.metadata to find reverse dependencies
        // This is robust and works across venvs
        let script = format!(
            "import importlib.metadata; \
            package = '{}'; \
            dependents = []; \
            for dist in importlib.metadata.distributions(): \
                try: \
                    requires = dist.requires or []; \
                    if any(package == r.split(' ')[0] for r in requires): \
                        dependents.append(dist.metadata['Name']); \
                except: pass; \
            print(','.join(dependents))",
            package
        );

        let output = std::process::Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new()); // Assume no dependents or python not found
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let deps: Vec<String> = output_str.trim()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(deps)
    }

    pub async fn create_snapshot(&self, package: &str) -> Result<Snapshot> {
        println!("{}", blue(format!("📸 Creating snapshot of '{}'...", package)));
        
        // 1. Find package location
        let script = format!(
            "import importlib.metadata; \
            try: \
                files = importlib.metadata.files('{}'); \
                if files: \
                    print(files[0].locate().parent); \
            except: pass",
            package
        );

        let output = std::process::Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()?;
            
        let location = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if location.is_empty() {
            return Err(anyhow::anyhow!("Could not locate package '{}'", package));
        }
        let package_path = PathBuf::from(&location);

        // 2. Zip the package directory
        let id = snakegg::native::id::new();
        let timestamp = DateTime::now().to_string();
        let snapshot_path = self.snapshots_dir.join(format!("{}_{}.zip", package, id));
        
        let file = std::fs::File::create(&snapshot_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Recursively add files
        let prefix = package_path.parent().unwrap_or(&package_path);
        let mut buffer = Vec::new();
        
        // Simple recursive walker
        let mut stack = vec![package_path.clone()];
        while let Some(dir) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path.clone());
                        let name = path.strip_prefix(prefix)?.to_string_lossy();
                        zip.add_directory(name, options)?;
                    } else {
                        let name = path.strip_prefix(prefix)?.to_string_lossy();
                        zip.start_file(name, options)?;
                        use std::io::Read;
                        let mut f = std::fs::File::open(&path)?;
                        f.read_to_end(&mut buffer)?;
                        use std::io::Write;
                        zip.write_all(&buffer)?;
                        buffer.clear();
                    }
                }
            }
        }
        zip.finish()?;
        
        Ok(Snapshot {
            id,
            timestamp,
            package: package.to_string(),
            version: "unknown".to_string(), 
            files_path: snapshot_path,
        })
    }

    pub async fn restore_snapshot(&self, snapshot_id: &str) -> Result<()> {
        println!("{}", green(format!("⏪ Restoring snapshot '{}'...", snapshot_id)));
        
        // Find the snapshot file
        let mut snapshot_path = None;
        if let Ok(entries) = std::fs::read_dir(&self.snapshots_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.contains(snapshot_id) && name.ends_with(".zip") {
                        snapshot_path = Some(path);
                        break;
                    }
                }
            }
        }

        let snapshot_path = snapshot_path.ok_or_else(|| anyhow::anyhow!("Snapshot '{}' not found", snapshot_id))?;
        
        // Determine restore location (site-packages)
        // We assume the zip structure preserves the relative path from site-packages
        // But we need to find site-packages first.
        // We can use python to find it.
        let output = std::process::Command::new("python3")
            .arg("-c")
            .arg("import site; print(site.getsitepackages()[0])")
            .output()?;
        let site_packages = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if site_packages.is_empty() {
             return Err(anyhow::anyhow!("Could not locate site-packages"));
        }
        let target_dir = PathBuf::from(site_packages);

        // Unzip
        let file = std::fs::File::open(&snapshot_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => target_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        println!("{}", green("✓ Restore complete"));
        Ok(())
    }

    pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        let mut snapshots = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.snapshots_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "zip") {
                    let name = path.file_stem().unwrap().to_string_lossy();
                    // name format: package_id
                    let parts: Vec<&str> = name.split('_').collect();
                    if parts.len() >= 2 {
                        let package = parts[0..parts.len()-1].join("_");
                        let id = parts.last().unwrap().to_string();
                        
                        snapshots.push(Snapshot {
                            id,
                            timestamp: "unknown".to_string(), // Metadata not stored in filename
                            package,
                            version: "unknown".to_string(),
                            files_path: path,
                        });
                    }
                }
            }
        }
        Ok(snapshots)
    }

    pub async fn uninstall(&self, package: &str) -> Result<()> {
        self.installer.uninstall_package(package).await
    }
}
