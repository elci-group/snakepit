use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub struct HallucinatoryFangs {
    sandbox_dir: PathBuf,
    modifications: Vec<ModificationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModificationRule {
    pub target_module: String,
    pub target_function: String,
    pub modification_type: ModificationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModificationType {
    // Add logging to every function call
    InjectLogging,
    
    // Add retry logic
    InjectRetry { max_attempts: u32, backoff_ms: u64 },
    
    // Add caching
    InjectCache { ttl_seconds: u64 },
    
    // Mock return values
    MockReturn { return_value: String },
    
    // Custom Python code injection
    CustomCode { code: String },
}

impl HallucinatoryFangs {
    pub fn new() -> Result<Self> {
        // Create sandbox directory in cache
        let sandbox_dir = crate::native::dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
            .join("snakepit")
            .join("fangs");
        
        std::fs::create_dir_all(&sandbox_dir)?;
        
        println!("🧪 Hallucinatory Fangs initialized");
        println!("📂 Sandbox: {}", sandbox_dir.display());
        
        Ok(Self {
            sandbox_dir,
            modifications: vec![],
        })
    }
    
    pub fn add_modification(&mut self, rule: ModificationRule) {
        self.modifications.push(rule);
    }
    
    pub fn fork_module(&self, module_name: &str) -> Result<PathBuf> {
        println!("🧪 Forking module: {}", module_name);
        
        // Find module location using Python
        let module_path = self.find_module(module_name)?;
        println!("📍 Found module at: {}", module_path.display());
        
        // Create sandbox fork directory
        let fork_dir = self.sandbox_dir.join(format!("{}_fork", module_name));
        
        // Remove existing fork if present
        if fork_dir.exists() {
            std::fs::remove_dir_all(&fork_dir)?;
        }
        
        std::fs::create_dir_all(&fork_dir)?;
        
        // Copy module files to sandbox
        self.copy_module(&module_path, &fork_dir)?;
        
        println!("✅ Forked to: {}", fork_dir.display());
        println!("💡 Original module untouched");
        
        Ok(fork_dir)
    }
    
    pub fn apply_modifications(&self, fork_dir: &PathBuf) -> Result<()> {
        if self.modifications.is_empty() {
            println!("⚠️  No modifications to apply");
            return Ok(());
        }
        
        println!("🔧 Applying {} modification(s)...", self.modifications.len());
        
        for (i, rule) in self.modifications.iter().enumerate() {
            println!("   {}. {} -> {} ({})", 
                i + 1,
                rule.target_module,
                rule.target_function,
                match &rule.modification_type {
                    ModificationType::InjectLogging => "logging",
                    ModificationType::InjectRetry { .. } => "retry",
                    ModificationType::InjectCache { .. } => "cache",
                    ModificationType::MockReturn { .. } => "mock",
                    ModificationType::CustomCode { .. } => "custom",
                }
            );
            
            self.apply_modification(fork_dir, rule)?;
        }
        
        println!("✅ All modifications applied");
        
        Ok(())
    }
    
    fn apply_modification(&self, fork_dir: &PathBuf, rule: &ModificationRule) -> Result<()> {
        // Find target file
        let target_file = self.find_module_file(fork_dir, &rule.target_module)?;
        
        // Read file content
        let content = std::fs::read_to_string(&target_file)?;
        
        // Apply modification based on type
        let modified_content = match &rule.modification_type {
            ModificationType::InjectLogging => {
                self.inject_logging(&content, &rule.target_function)?
            }
            ModificationType::InjectRetry { max_attempts, backoff_ms } => {
                self.inject_retry(&content, &rule.target_function, *max_attempts, *backoff_ms)?
            }
            ModificationType::InjectCache { ttl_seconds } => {
                self.inject_cache(&content, &rule.target_function, *ttl_seconds)?
            }
            ModificationType::MockReturn { return_value } => {
                self.mock_return(&content, &rule.target_function, return_value)?
            }
            ModificationType::CustomCode { code } => {
                self.inject_custom_code(&content, &rule.target_function, code)?
            }
        };
        
        // Write modified content back
        std::fs::write(&target_file, modified_content)?;
        
        Ok(())
    }
    
    fn inject_logging(&self, content: &str, function_name: &str) -> Result<String> {
        let function_pattern = format!("def {}(", function_name);
        
        if let Some(pos) = content.find(&function_pattern) {
            // Find end of function signature (colon)
            if let Some(colon_pos) = content[pos..].find(':') {
                let insert_pos = pos + colon_pos + 1;
                
                // Create logging code
                let logging_code = format!(
                    "\n    import logging\n    logging.info('🔍 Calling {}()')\n",
                    function_name
                );
                
                let mut modified = content.to_string();
                modified.insert_str(insert_pos, &logging_code);
                
                return Ok(modified);
            }
        }
        
        Err(anyhow::anyhow!("Function '{}' not found", function_name))
    }
    
    fn inject_retry(&self, content: &str, function_name: &str, max_attempts: u32, backoff_ms: u64) -> Result<String> {
        // Add retry decorator before function
        let function_pattern = format!("def {}(", function_name);
        
        if let Some(pos) = content.find(&function_pattern) {
            let decorator = format!(
                "def _retry_wrapper(func):\n    \
                 def wrapper(*args, **kwargs):\n        \
                 for attempt in range({}):\n            \
                 try:\n                \
                 return func(*args, **kwargs)\n            \
                 except Exception as e:\n                \
                 if attempt == {} - 1:\n                    \
                 raise\n                \
                 import time\n                \
                 time.sleep({} / 1000.0)\n        \
                 return None\n    \
                 return wrapper\n\n\
                 @_retry_wrapper\n",
                max_attempts, max_attempts, backoff_ms
            );
            
            let mut modified = content.to_string();
            modified.insert_str(pos, &decorator);
            
            return Ok(modified);
        }
        
        Err(anyhow::anyhow!("Function '{}' not found", function_name))
    }
    
    fn inject_cache(&self, content: &str, function_name: &str, _ttl_seconds: u64) -> Result<String> {
        // Add functools.lru_cache decorator
        let function_pattern = format!("def {}(", function_name);
        
        if let Some(pos) = content.find(&function_pattern) {
            let decorator = "@functools.lru_cache(maxsize=128)\n";
            
            let mut modified = content.to_string();
            modified.insert_str(pos, decorator);
            
            // Add import at top if not present
            if !modified.contains("import functools") {
                modified.insert_str(0, "import functools\n");
            }
            
            return Ok(modified);
        }
        
        Err(anyhow::anyhow!("Function '{}' not found", function_name))
    }
    
    fn mock_return(&self, content: &str, function_name: &str, return_value: &str) -> Result<String> {
        let function_pattern = format!("def {}(", function_name);
        
        if let Some(pos) = content.find(&function_pattern) {
            // Find function body (after colon)
            if let Some(colon_pos) = content[pos..].find(':') {
                let insert_pos = pos + colon_pos + 1;
                
                let mock_code = format!("\n    return {}\n", return_value);
                
                let mut modified = content.to_string();
                modified.insert_str(insert_pos, &mock_code);
                
                return Ok(modified);
            }
        }
        
        Err(anyhow::anyhow!("Function '{}' not found", function_name))
    }
    
    fn inject_custom_code(&self, content: &str, function_name: &str, code: &str) -> Result<String> {
        let function_pattern = format!("def {}(", function_name);
        
        if let Some(pos) = content.find(&function_pattern) {
            if let Some(colon_pos) = content[pos..].find(':') {
                let insert_pos = pos + colon_pos + 1;
                
                let custom_code = format!("\n    {}\n", code);
                
                let mut modified = content.to_string();
                modified.insert_str(insert_pos, &custom_code);
                
                return Ok(modified);
            }
        }
        
        Err(anyhow::anyhow!("Function '{}' not found", function_name))
    }
    
    fn find_module(&self, module_name: &str) -> Result<PathBuf> {
        // Use Python to find module location
        let output = std::process::Command::new("python3")
            .args(&[
                "-c",
                &format!("import {}; import os; print(os.path.dirname({}.__file__))", module_name, module_name)
            ])
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Module '{}' not found: {}", module_name, error));
        }
        
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        Ok(PathBuf::from(path_str))
    }
    
    fn find_module_file(&self, fork_dir: &PathBuf, module_name: &str) -> Result<PathBuf> {
        // Try common patterns
        let candidates = vec![
            fork_dir.join(format!("{}.py", module_name)),
            fork_dir.join(module_name).join("__init__.py"),
            fork_dir.join("__init__.py"),
        ];
        
        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }
        
        Err(anyhow::anyhow!("Could not find module file for '{}'", module_name))
    }
    
    fn copy_module(&self, source: &PathBuf, dest: &PathBuf) -> Result<()> {
        // Recursively copy module files
        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_name = entry.file_name();
            
            // Skip __pycache__ and .pyc files
            if file_name == "__pycache__" || file_name.to_string_lossy().ends_with(".pyc") {
                continue;
            }
            
            let dest_path = dest.join(&file_name);
            
            if file_type.is_dir() {
                std::fs::create_dir_all(&dest_path)?;
                self.copy_module(&entry.path(), &dest_path)?;
            } else if file_type.is_file() {
                std::fs::copy(&entry.path(), &dest_path)?;
            }
        }
        
        Ok(())
    }
    
    pub fn rollback(&self, module_name: &str) -> Result<()> {
        println!("🔄 Rolling back modifications to {}...", module_name);
        
        let fork_dir = self.sandbox_dir.join(format!("{}_fork", module_name));
        
        if fork_dir.exists() {
            std::fs::remove_dir_all(&fork_dir)?;
            println!("✅ Rollback complete - fork removed");
        } else {
            println!("⚠️  No fork found for '{}'", module_name);
        }
        
        Ok(())
    }
    
    pub fn list_forks(&self) -> Result<Vec<String>> {
        let mut forks = vec![];
        
        if !self.sandbox_dir.exists() {
            return Ok(forks);
        }
        
        for entry in std::fs::read_dir(&self.sandbox_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with("_fork") {
                    forks.push(name.trim_end_matches("_fork").to_string());
                }
            }
        }
        
        Ok(forks)
    }
}
