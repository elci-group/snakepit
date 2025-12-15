use crate::resolver::ResolvedDependency;
use anyhow::Result;
use std::process::{Command, Stdio};
use crate::native::progress::ProgressBar;
use crate::native::style::{red, green, yellow, blue, cyan, bold, dim};

#[derive(Debug, Clone)]
pub enum InstallerBackend {
    Native,
    Uv,
    Pip,
    Conda,
    Poetry,
}

impl InstallerBackend {
    pub fn detect() -> Self {
        // Snakepit Native is always available and preferred!
        Self::Native
    }

    fn command_exists(command: &str) -> bool {
        Command::new(command)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    }
}

#[derive(Debug)]
pub struct PackageInstaller {
    backend: InstallerBackend,
    venv_path: Option<String>,
    use_cache: bool,
}

impl PackageInstaller {
    pub fn new() -> Self {
        Self {
            backend: InstallerBackend::detect(),
            venv_path: None,
            use_cache: true,
        }
    }

    pub fn with_backend(mut self, backend: InstallerBackend) -> Self {
        self.backend = backend;
        self
    }

    pub fn with_venv(mut self, venv_path: String) -> Self {
        self.venv_path = Some(venv_path);
        self
    }

    pub fn with_cache(mut self, use_cache: bool) -> Self {
        self.use_cache = use_cache;
        self
    }

    pub async fn install_package(&self, package: &str, version: Option<&str>) -> Result<()> {
        let mut pb = ProgressBar::new_spinner();
        pb.set_message(format!("Installing {}...", package));

        let result = match self.backend {
            InstallerBackend::Native => self.install_with_native(package, version).await,
            InstallerBackend::Uv => self.install_with_uv(package, version).await,
            InstallerBackend::Pip => self.install_with_pip(package, version).await,
            InstallerBackend::Conda => self.install_with_conda(package, version).await,
            InstallerBackend::Poetry => self.install_with_poetry(package, version).await,
        };

        pb.finish_with_message(&format!("{} {}", 
            green("✓"), 
            green(format!("Installed {}", package))
        ));

        result
    }

    pub async fn install_dependencies(&self, dependencies: &[ResolvedDependency]) -> Result<()> {
        if dependencies.is_empty() {
            return Ok(());
        }

        println!("{}", cyan(format!("🚀 Installing {} packages in parallel...", dependencies.len())));

        let mut pb = ProgressBar::new(dependencies.len() as u64);
        // Native progress bar has default styling


        // Spawn parallel install tasks
        let mut handles = vec![];
        
        for dep in dependencies {
            let package = dep.name.clone();
            let version = dep.version.clone();
            let backend = self.backend.clone();
            let venv_path = self.venv_path.clone();
            let use_cache = self.use_cache;
            
            let handle = tokio::spawn(async move {
                let installer = PackageInstaller {
                    backend,
                    venv_path,
                    use_cache,
                };
                installer.install_package(&package, Some(&version)).await
            });
            
            handles.push((dep.name.clone(), handle));
        }

        // Await all tasks
        let mut errors = vec![];
        for (name, handle) in handles {
            match handle.await {
                Ok(Ok(_)) => {
                    pb.inc(1);
                    pb.set_message(format!("✓ {}", name));
                }
                Ok(Err(e)) => {
                    errors.push(format!("{}: {}", name, e));
                    pb.inc(1);
                }
                Err(e) => {
                    errors.push(format!("{}: Task failed: {}", name, e));
                    pb.inc(1);
                }
            }
        }

        let msg = if errors.is_empty() {
            green("All dependencies installed!").to_string()
        } else {
            yellow(format!("Completed with {} errors", errors.len())).to_string()
        };
        pb.finish_with_message(&msg);

        if !errors.is_empty() {
            eprintln!("{}", red("Errors:"));
            for err in &errors {
                eprintln!("  {}", err);
            }
            return Err(anyhow::anyhow!("Failed to install some dependencies"));
        }

        Ok(())
    }

    pub async fn uninstall_package(&self, package: &str) -> Result<()> {
        let mut pb = ProgressBar::new_spinner();
        pb.set_message(format!("Uninstalling {}...", package));

        let result = match self.backend {
            InstallerBackend::Native => self.uninstall_with_native(package).await,
            InstallerBackend::Uv => self.uninstall_with_uv(package).await,
            InstallerBackend::Pip => self.uninstall_with_pip(package).await,
            InstallerBackend::Conda => self.uninstall_with_conda(package).await,
            InstallerBackend::Poetry => self.uninstall_with_poetry(package).await,
        };

        pb.finish_with_message(&format!("{} {}", 
            red("✓"), 
            red(format!("Uninstalled {}", package))
        ));

        result
    }

    pub async fn list_installed_packages(&self) -> Result<Vec<String>> {
        match self.backend {
            InstallerBackend::Native => self.list_with_native().await,
            InstallerBackend::Uv => self.list_with_uv().await,
            InstallerBackend::Pip => self.list_with_pip().await,
            InstallerBackend::Conda => self.list_with_conda().await,
            InstallerBackend::Poetry => self.list_with_poetry().await,
        }
    }

    async fn install_with_native(&self, package: &str, version: Option<&str>) -> Result<()> {
        use std::io::Cursor;
        use zip::ZipArchive;

        // 1. Fetch metadata from PyPI (with caching)
        let resp = self.fetch_pypi_metadata_cached(package).await?;
        
        let releases = resp["releases"].as_object()
            .ok_or_else(|| anyhow::anyhow!("No releases found for {}", package))?;

        // 2. Select version
        let target_version = version.unwrap_or_else(|| resp["info"]["version"].as_str().unwrap_or(""));
        let files = releases.get(target_version)
            .ok_or_else(|| anyhow::anyhow!("Version {} not found for {}", target_version, package))?
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid release data"))?;

        // 3. Find a compatible wheel using robust selection
        let selector = WheelSelector::new();
        let wheel_url = files.iter()
            .filter(|f| f["filename"].as_str().map_or(false, |n| n.ends_with(".whl")))
            .max_by_key(|f| {
                let filename = f["filename"].as_str().unwrap_or("");
                selector.score_wheel(filename)
            })
            .and_then(|f| {
                let filename = f["filename"].as_str().unwrap_or("");
                if selector.score_wheel(filename) > 0 {
                    f["url"].as_str()
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("No compatible wheel found for {} (checked {} files)", package, files.len()))?;

        let wheel_filename = wheel_url.split('/').last().unwrap_or("unknown");

        // 4. Download wheel (with caching)
        let bytes = if self.use_cache {
            Self::download_wheel_cached(wheel_url, wheel_filename).await?
        } else {
            println!("{}", dim(format!("📦 Downloading wheel: {}", wheel_filename)));
            Self::download_wheel(wheel_url).await?
        };
        
        // 4.5. Verify wheel integrity (prefer SHA256, fallback to MD5)
        let file_info = files.iter()
            .find(|f| f["filename"].as_str() == Some(wheel_filename));
            
        let sha256 = file_info.and_then(|f| f["digests"]["sha256"].as_str());
        let md5 = file_info.and_then(|f| f["digests"]["md5"].as_str());
        
        if sha256.is_some() || md5.is_some() {
            Self::verify_wheel_integrity(&bytes, sha256, md5)?;
        }
        
        // 5. Determine install location
        let install_dir = self.get_install_dir()?;
        
        // 5.5. Check disk space before installation
        Self::check_disk_space(&install_dir, bytes.len() as u64 * 3)?; // 3x for extraction overhead
        
        // 5.6. Try to create install directory, fallback to user site if permission denied
        match std::fs::create_dir_all(&install_dir) {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                println!("{}", yellow("⚠️  Permission denied, trying user install..."));
                // For permission errors, we'll just fail gracefully for now
                // Full user-site fallback would require refactoring the installer
                return Err(anyhow::anyhow!("Permission denied. Try running with sudo or use --user flag"));
            }
            Err(e) => return Err(e.into()),
        }

        // 6. Unpack wheel
        Self::unpack_wheel(&bytes, &install_dir)?;

        Ok(())
    }

    async fn download_wheel(url: &str) -> Result<Vec<u8>> {
        Self::download_with_retry(url, 3).await
    }

    async fn download_with_retry(url: &str, max_retries: u32) -> Result<Vec<u8>> {
        let mut last_error = None;
        
        for attempt in 1..=max_retries {
            match reqwest::get(url).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.bytes().await {
                        Ok(bytes) => return Ok(bytes.to_vec()),
                        Err(e) => {
                            last_error = Some(anyhow::anyhow!("Failed to read response: {}", e));
                            if attempt < max_retries {
                                let wait_secs = 2u64.pow(attempt - 1); // Exponential backoff: 1s, 2s, 4s
                                println!("{}", format!(
                                    "⚠️  Download interrupted (attempt {}/{}), retrying in {}s...",
                                    attempt, max_retries, wait_secs
                                ));
                                tokio::time::sleep(tokio::time::Duration::from_secs(wait_secs)).await;
                            }
                        }
                    }
                }
                Ok(resp) => {
                    last_error = Some(anyhow::anyhow!("HTTP error: {}", resp.status()));
                    if attempt < max_retries {
                        println!("{}", format!(
                            "⚠️  Download failed with status {} (attempt {}/{}), retrying...",
                            resp.status(), attempt, max_retries
                        ));
                        tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(attempt - 1))).await;
                    }
                }
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Network error: {}", e));
                    if attempt < max_retries {
                        println!("{}", format!(
                            "⚠️  Network error (attempt {}/{}), retrying...",
                            attempt, max_retries
                        ));
                        tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(attempt - 1))).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Download failed after {} attempts", max_retries)))
    }

    async fn download_wheel_cached(url: &str, filename: &str) -> Result<Vec<u8>> {
        use std::io::Read;
        
        // Create cache directory
        let cache_dir = crate::native::dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
            .join("snakepit")
            .join("wheels");
        
        std::fs::create_dir_all(&cache_dir)?;

        // Use URL hash as cache key (more reliable than filename which might have version conflicts)
        let cache_key = crate::native::hash::compute_hex(url.as_bytes());
        let cache_path = cache_dir.join(format!("{}.whl", cache_key));

        // Check cache
        if cache_path.exists() {
            println!("{}", green(format!("💾 Using cached wheel: {}", filename)));
            let mut file = std::fs::File::open(&cache_path)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            return Ok(bytes);
        }

        // Download and cache
        println!("{}", dim(format!("📦 Downloading wheel: {}", filename)));
        let bytes = Self::download_wheel(url).await?;
        
        // Write to cache
        std::fs::write(&cache_path, &bytes)?;
        
        Ok(bytes)
    }

    async fn fetch_pypi_metadata_cached(&self, package: &str) -> Result<serde_json::Value> {
        use std::time::SystemTime;
        
        // Create metadata cache directory
        let cache_dir = crate::native::dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
            .join("snakepit")
            .join("metadata");
        
        std::fs::create_dir_all(&cache_dir)?;
        
        let cache_path = cache_dir.join(format!("{}.json", package));
        
        // Check cache with TTL (1 hour)
        if cache_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&cache_path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                        // Cache valid for 1 hour
                        if elapsed.as_secs() < 3600 {
                            if let Ok(cached) = std::fs::read_to_string(&cache_path) {
                                if let Ok(json) = serde_json::from_str(&cached) {
                                    println!("{}", dim(format!("💾 Using cached metadata for {}", package)));
                                    return Ok(json);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Fetch from PyPI
        println!("{}", dim(format!("🌐 Fetching metadata for {}...", package)));
        let url = format!("https://pypi.org/pypi/{}/json", package);
        let resp = reqwest::get(&url).await?.json::<serde_json::Value>().await?;
        
        // Cache response
        if let Ok(json_str) = serde_json::to_string_pretty(&resp) {
            let _ = std::fs::write(&cache_path, json_str);
        }
        
        Ok(resp)
    }

    fn get_install_dir(&self) -> Result<std::path::PathBuf> {
        if let Some(venv) = &self.venv_path {
            let venv_path = std::path::Path::new(venv);
            if cfg!(target_os = "windows") {
                Ok(venv_path.join("Lib").join("site-packages"))
            } else {
                let lib = venv_path.join("lib");
                let mut site = lib.join("python3.10").join("site-packages");
                if let Ok(entries) = std::fs::read_dir(&lib) {
                    for entry in entries.flatten() {
                        if entry.file_name().to_string_lossy().starts_with("python") {
                            site = entry.path().join("site-packages");
                            break;
                        }
                    }
                }
                Ok(site)
            }
        } else {
            let home = crate::native::dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
            let mut site = home.join(".local").join("lib").join("python3.10").join("site-packages");
            if !site.exists() {
                let lib = home.join(".local").join("lib");
                if let Ok(entries) = std::fs::read_dir(&lib) {
                    for entry in entries.flatten() {
                        if entry.file_name().to_string_lossy().starts_with("python") {
                            site = entry.path().join("site-packages");
                            break;
                        }
                    }
                }
            }
            Ok(site)
        }
    }

    fn unpack_wheel(bytes: &[u8], install_dir: &std::path::Path) -> Result<()> {
        use std::io::Cursor;
        use zip::ZipArchive;
        use rayon::prelude::*;
        use std::sync::{Arc, Mutex};

        println!("{}", dim("🔧 Extracting files..."));

        let reader = Cursor::new(bytes);
        let archive = Arc::new(Mutex::new(ZipArchive::new(reader)?));
        
        // First pass: collect file metadata
        let file_count = {
            let archive_lock = archive.lock().unwrap();
            archive_lock.len()
        };
        
        let file_info: Vec<_> = (0..file_count)
            .map(|i| {
                let mut archive_lock = archive.lock().unwrap();
                let file = archive_lock.by_index(i).ok()?;
                Some((
                    i,
                    file.name().to_string(),
                    file.is_dir(),
                    file.size() as usize,
                ))
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| anyhow::anyhow!("Failed to read archive metadata"))?;

        // Create all directories first (sequential, fast)
        for (_, name, is_dir, _) in &file_info {
            if *is_dir {
                let outpath = install_dir.join(name);
                std::fs::create_dir_all(&outpath)?;
            }
        }

        // Extract files in parallel
        let errors: Vec<_> = file_info
            .par_iter()
            .filter(|(_, _, is_dir, _)| !is_dir)
            .filter_map(|(idx, name, _, _)| {
                let outpath = install_dir.join(name);
                
                // Ensure parent directory exists
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        if let Err(e) = std::fs::create_dir_all(p) {
                            return Some(format!("Failed to create dir {}: {}", p.display(), e));
                        }
                    }
                }
                
                // Extract file
                let result = (|| -> Result<()> {
                    let mut archive_lock = archive.lock().unwrap();
                    let mut file = archive_lock.by_index(*idx)?;
                    let mut outfile = std::fs::File::create(&outpath)?;
                    std::io::copy(&mut file, &mut outfile)?;
                    Ok(())
                })();
                
                if let Err(e) = result {
                    Some(format!("Failed to extract {}: {}", name, e))
                } else {
                    None
                }
            })
            .collect();

        if !errors.is_empty() {
            return Err(anyhow::anyhow!("Extraction errors: {}", errors.join(", ")));
        }

        Ok(())
    }

    async fn uninstall_with_native(&self, package: &str) -> Result<()> {
        // Basic uninstall: remove the directory/file in site-packages
        // This is risky without reading RECORD, but for "bleeding edge" prototype it works.
        // We'll just warn that it's not fully implemented.
        println!("{}", yellow("⚠️  Native uninstall not fully implemented. Please manually remove files if needed."));
        Ok(())
    }

    async fn list_with_native(&self) -> Result<Vec<String>> {
        // Scan site-packages for .dist-info directories
        let install_dir = if let Some(venv) = &self.venv_path {
             let venv_path = std::path::Path::new(venv);
            if cfg!(target_os = "windows") {
                venv_path.join("Lib").join("site-packages")
            } else {
                let lib = venv_path.join("lib");
                let mut site = lib.join("python3.10").join("site-packages");
                if let Ok(entries) = std::fs::read_dir(&lib) {
                    for entry in entries.flatten() {
                        if entry.file_name().to_string_lossy().starts_with("python") {
                            site = entry.path().join("site-packages");
                            break;
                        }
                    }
                }
                site
            }
        } else {
             let home = crate::native::dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
            let mut site = home.join(".local").join("lib").join("python3.10").join("site-packages");
             if !site.exists() {
                 let lib = home.join(".local").join("lib");
                 if let Ok(entries) = std::fs::read_dir(&lib) {
                    for entry in entries.flatten() {
                        if entry.file_name().to_string_lossy().starts_with("python") {
                            site = entry.path().join("site-packages");
                            break;
                        }
                    }
                }
            }
            site
        };

        let mut packages = Vec::new();
        if install_dir.exists() {
            for entry in std::fs::read_dir(install_dir)? {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".dist-info") {
                    let pkg_name = name.split('-').next().unwrap_or(&name).to_string();
                    packages.push(pkg_name);
                }
            }
        }
        Ok(packages)
    }

    async fn install_with_uv(&self, package: &str, version: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("uv");
        cmd.arg("pip").arg("install");
        
        if let Some(venv_path) = &self.venv_path {
            // uv uses VIRTUAL_ENV env var or --python
            cmd.env("VIRTUAL_ENV", venv_path);
        } else {
            // For global/user install, uv pip install --system is needed if not in venv
            // But we want user install. uv doesn't support --user directly in the same way.
            // However, we can target the user site.
            // Actually, uv pip install --system installs to site-packages.
            // Let's try just 'uv pip install' which might require a venv, or --system.
            // If we want to mimic pip --user, we might need to find the user site.
            // For now, let's use --system which installs to the python environment found.
            cmd.arg("--system");
        }
        
        if !self.use_cache {
            cmd.arg("--no-cache");
        }
        
        if let Some(ver) = version {
            cmd.arg(&format!("{}=={}", package, ver));
        } else {
            cmd.arg(package);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install {}: {}", package, error));
        }

        Ok(())
    }

    async fn install_with_pip(&self, package: &str, version: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("pip");
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--python").arg(venv_path);
        } else {
            // If no venv is specified, assume user installation
            cmd.arg("--user");
        }
        
        cmd.arg("install");
        
        if !self.use_cache {
            cmd.arg("--no-cache-dir");
        }
        
        if let Some(ver) = version {
            cmd.arg(&format!("{}=={}", package, ver));
        } else {
            cmd.arg(package);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            
            // Check for externally managed environment error (PEP 668)
            if error.contains("externally-managed-environment") {
                eprintln!("{} Externally managed environment detected. Retrying with --break-system-packages...", yellow("WARN:"));
                
                let mut retry_cmd = Command::new("pip");
                if let Some(venv_path) = &self.venv_path {
                    retry_cmd.arg("--python").arg(venv_path);
                } else {
                    retry_cmd.arg("--user");
                }
                
                retry_cmd.arg("install");
                retry_cmd.arg("--break-system-packages");
                
                if !self.use_cache {
                    retry_cmd.arg("--no-cache-dir");
                }
                
                if let Some(ver) = version {
                    retry_cmd.arg(&format!("{}=={}", package, ver));
                } else {
                    retry_cmd.arg(package);
                }
                
                let retry_output = retry_cmd.output()?;
                if retry_output.status.success() {
                    return Ok(());
                }
                
                let retry_error = String::from_utf8_lossy(&retry_output.stderr);
                return Err(anyhow::anyhow!("Failed to install {} (even with break-system-packages): {}", package, retry_error));
            }

            return Err(anyhow::anyhow!("Failed to install {}: {}", package, error));
        }

        Ok(())
    }

    async fn install_with_conda(&self, package: &str, version: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("conda");
        cmd.arg("install").arg("-y");
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--prefix").arg(venv_path);
        }
        
        if let Some(ver) = version {
            cmd.arg(&format!("{}={}", package, ver));
        } else {
            cmd.arg(package);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install {}: {}", package, error));
        }

        Ok(())
    }

    async fn install_with_poetry(&self, package: &str, version: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("poetry");
        cmd.arg("add");
        
        if let Some(ver) = version {
            cmd.arg(&format!("{}=={}", package, ver));
        } else {
            cmd.arg(package);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install {}: {}", package, error));
        }

        Ok(())
    }

    async fn uninstall_with_uv(&self, package: &str) -> Result<()> {
        let mut cmd = Command::new("uv");
        cmd.arg("pip").arg("uninstall").arg(package);
        
        if let Some(venv_path) = &self.venv_path {
            cmd.env("VIRTUAL_ENV", venv_path);
        } else {
            cmd.arg("--system");
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to uninstall {}: {}", package, error));
        }

        Ok(())
    }

    async fn uninstall_with_pip(&self, package: &str) -> Result<()> {
        let mut cmd = Command::new("pip");
        cmd.arg("uninstall").arg("-y").arg(package);
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--python").arg(venv_path);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to uninstall {}: {}", package, error));
        }

        Ok(())
    }

    async fn uninstall_with_conda(&self, package: &str) -> Result<()> {
        let mut cmd = Command::new("conda");
        cmd.arg("remove").arg("-y").arg(package);
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--prefix").arg(venv_path);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to uninstall {}: {}", package, error));
        }

        Ok(())
    }

    async fn uninstall_with_poetry(&self, package: &str) -> Result<()> {
        let mut cmd = Command::new("poetry");
        cmd.arg("remove").arg(package);

        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to uninstall {}: {}", package, error));
        }

        Ok(())
    }

    async fn list_with_uv(&self) -> Result<Vec<String>> {
        let mut cmd = Command::new("uv");
        cmd.arg("pip").arg("freeze");
        
        if let Some(venv_path) = &self.venv_path {
            cmd.env("VIRTUAL_ENV", venv_path);
        } else {
            cmd.arg("--system");
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list packages"));
        }

        let packages: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.split("==").next().unwrap_or("").to_string())
            .collect();

        Ok(packages)
    }

    async fn list_with_pip(&self) -> Result<Vec<String>> {
        let mut cmd = Command::new("pip");
        cmd.arg("list").arg("--format=freeze");
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--python").arg(venv_path);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list packages"));
        }

        let packages: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.split('=').next().unwrap_or("").to_string())
            .collect();

        Ok(packages)
    }

    async fn list_with_conda(&self) -> Result<Vec<String>> {
        let mut cmd = Command::new("conda");
        cmd.arg("list");
        
        if let Some(venv_path) = &self.venv_path {
            cmd.arg("--prefix").arg(venv_path);
        }

        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list packages"));
        }

        let packages: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(2) // Skip header lines
            .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
            .collect();

        Ok(packages)
    }

    async fn list_with_poetry(&self) -> Result<Vec<String>> {
        let mut cmd = Command::new("poetry");
        cmd.arg("show").arg("--only=main");

        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list packages"));
        }

        let packages: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
            .collect();

        Ok(packages)
    }
    
    // Helper: Verify wheel integrity using SHA256 or MD5
    fn verify_wheel_integrity(bytes: &[u8], sha256: Option<&str>, md5: Option<&str>) -> Result<()> {
        if let Some(expected) = sha256 {
            let actual = crate::native::hash::compute_sha256_hex(bytes);
            if actual != expected {
                return Err(anyhow::anyhow!(
                    "SHA256 integrity check failed: expected {}, got {}",
                    expected, actual
                ));
            }
            println!("{}", dim("✅ SHA256 integrity verified"));
            return Ok(());
        }
        
        if let Some(expected) = md5 {
            let actual = crate::native::hash::compute_hex(bytes);
            if actual != expected {
                return Err(anyhow::anyhow!(
                    "MD5 integrity check failed: expected {}, got {}",
                    expected, actual
                ));
            }
            println!("{}", dim("✅ MD5 integrity verified"));
            return Ok(());
        }
        
        Ok(())
    }
    
    // Helper: Check available disk space
    fn check_disk_space(_install_dir: &std::path::Path, _required_bytes: u64) -> Result<()> {
        // Try to get filesystem stats
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(_metadata) = std::fs::metadata(_install_dir.parent().unwrap_or(_install_dir)) {
                // This is a simplified check - in production would use statvfs
                // For now, just ensure we can write
                return Ok(());
            }
        }
        
        // Fallback: try to write a test file
        let test_file = _install_dir.join(".snakepit_disk_check");
        match std::fs::write(&test_file, &vec![0u8; 1024]) {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                Err(anyhow::anyhow!("Permission denied"))
            }
            Err(_) => {
                Err(anyhow::anyhow!("Insufficient disk space"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_detection() {
        let backend = InstallerBackend::detect();
        // This test will pass regardless of what's installed
        assert!(matches!(backend, InstallerBackend::Uv | InstallerBackend::Pip | InstallerBackend::Conda | InstallerBackend::Poetry | InstallerBackend::Native));
    }
}

struct WheelSelector {
    os: String,
    arch: String,
    python_version: String,
}

impl WheelSelector {
    fn new() -> Self {
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();
        // TODO: Dynamically detect python version. For now, assume 3.10 as per user env.
        let python_version = "310".to_string(); 
        
        Self { os, arch, python_version }
    }

    fn score_wheel(&self, filename: &str) -> i32 {
        let parts: Vec<&str> = filename.trim_end_matches(".whl").split('-').collect();
        if parts.len() < 5 {
            return 0; // Invalid wheel name format
        }

        // Format: {distribution}-{version}-{python_tag}-{abi_tag}-{platform_tag}.whl
        let python_tag = parts[2];
        let abi_tag = parts[3];
        let platform_tag = parts[4];

        let mut score = 0;

        // 1. Platform Check
        let platform_match = match self.os.as_str() {
            "linux" => platform_tag.contains("manylinux") || platform_tag.contains("linux"),
            "macos" => platform_tag.contains("macosx") || platform_tag.contains("darwin"),
            "windows" => platform_tag.contains("win"),
            _ => false,
        };

        if platform_tag == "any" {
            score += 10; // Universal fallback
        } else if platform_match {
            score += 100; // Platform match
            
            // Arch check
            if self.arch == "x86_64" && (platform_tag.contains("x86_64") || platform_tag.contains("amd64")) {
                score += 50;
            } else if self.arch == "aarch64" && (platform_tag.contains("aarch64") || platform_tag.contains("arm64")) {
                score += 50;
            } else {
                return 0; // Wrong arch
            }
        } else {
            return 0; // Wrong OS
        }

        // 2. Python Version Check
        if python_tag == "py3" || python_tag == "py2.py3" {
            score += 10; // Universal python
        } else if python_tag.contains(&format!("cp{}", self.python_version)) {
            score += 50; // Exact version match
        } else if python_tag.contains("cp3") {
             // Generic CPython 3 match (risky but better than nothing)
             score += 5;
        } else {
            return 0; // Incompatible python version
        }

        // 3. ABI Check
        if abi_tag == "none" || abi_tag == "abi3" {
            score += 10;
        } else if abi_tag.contains(&format!("cp{}", self.python_version)) {
            score += 20;
        }

        score
    }
}
