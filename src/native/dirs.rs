// Native alternative to `dirs` crate
// Standard directories with zero dependencies
// Savings: -20 KB, zero external deps

use std::path::PathBuf;
use std::env;

/// Get user's home directory
/// 
/// # Example
/// ```
/// let home = home_dir().unwrap();
/// println!("Home: {}", home.display());
/// ```
pub fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        env::var_os("HOME").map(PathBuf::from)
    }
    
    #[cfg(windows)]
    {
        env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .or_else(|| {
                let drive = env::var_os("HOMEDRIVE")?;
                let path = env::var_os("HOMEPATH")?;
                Some(PathBuf::from(drive).join(path))
            })
    }
}

/// Get user's cache directory
/// 
/// - Linux: `$XDG_CACHE_HOME` or `$HOME/.cache`
/// - macOS: `$HOME/Library/Caches`
/// - Windows: `%LOCALAPPDATA%`
pub fn cache_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|h| h.join(".cache")))
    }
    
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| h.join("Library/Caches"))
    }
    
    #[cfg(windows)]
    {
        env::var_os("LOCALAPPDATA").map(PathBuf::from)
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        home_dir().map(|h| h.join(".cache"))
    }
}

/// Get user's config directory
/// 
/// - Linux: `$XDG_CONFIG_HOME` or `$HOME/.config`
/// - macOS: `$HOME/Library/Application Support`
/// - Windows: `%APPDATA%`
pub fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|h| h.join(".config")))
    }
    
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| h.join("Library/Application Support"))
    }
    
    #[cfg(windows)]
    {
        env::var_os("APPDATA").map(PathBuf::from)
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        home_dir().map(|h| h.join(".config"))
    }
}

/// Get user's data directory
/// 
/// - Linux: `$XDG_DATA_HOME` or `$HOME/.local/share`
/// - macOS: `$HOME/Library/Application Support`
/// - Windows: `%APPDATA%`
pub fn data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|h| h.join(".local/share")))
    }
    
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| h.join("Library/Application Support"))
    }
    
    #[cfg(windows)]
    {
        env::var_os("APPDATA").map(PathBuf::from)
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        home_dir().map(|h| h.join(".local/share"))
    }
}

/// Get user's data local directory (Linux-specific)
/// 
/// - Linux: `$XDG_DATA_HOME` or `$HOME/.local/share`
/// - Others: Same as data_dir()
pub fn data_local_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|h| h.join(".local/share")))
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        data_dir()
    }
}

/// Get user's executable directory
/// 
/// - Linux: `$HOME/.local/bin`
/// - macOS: `$HOME/.local/bin`
/// - Windows: N/A (returns None)
pub fn executable_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        home_dir().map(|h| h.join(".local/bin"))
    }
    
    #[cfg(windows)]
    {
        None
    }
}

/// Get user's runtime directory (Linux-specific)
/// 
/// - Linux: `$XDG_RUNTIME_DIR` or `/tmp`
/// - Others: System temp directory
pub fn runtime_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .or_else(|| Some(PathBuf::from("/tmp")))
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        Some(env::temp_dir())
    }
}

/// Get system temp directory
pub fn temp_dir() -> PathBuf {
    env::temp_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_dir() {
        let home = home_dir();
        assert!(home.is_some(), "Home directory should exist");
    }

    #[test]
    fn test_cache_dir() {
        let cache = cache_dir();
        assert!(cache.is_some(), "Cache directory should be determinable");
    }

    #[test]
    fn test_config_dir() {
        let config = config_dir();
        assert!(config.is_some(), "Config directory should be determinable");
    }

    #[test]
    fn test_data_dir() {
        let data = data_dir();
        assert!(data.is_some(), "Data directory should be determinable");
    }

    #[test]
    fn test_temp_dir() {
        let temp = temp_dir();
        assert!(temp.exists(), "Temp directory should exist");
    }
}
