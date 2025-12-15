use anyhow::{Result, anyhow};
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    // Simplified PEP 508 regex pattern
    static ref PEP508_PATTERN: Regex = Regex::new(r"(?x)
        ^
        (?P<name>[a-zA-Z0-9]([a-zA-Z0-9._-]*[a-zA-Z0-9])?)  # Package name
        (?:\[(?P<extras>[^\]]+)\])?                          # Optional extras
        \s*
        (?P<specifier>[^;]*)?                                # Version specifier
        (?:;\s*(?P<marker>.*))?                              # Environment marker
        $
    ").unwrap();
}

#[derive(Debug, Clone)]
pub struct DependencySpecifier {
    pub name: String,
    pub extras: Vec<String>,
    pub version_specs: Vec<VersionSpecifier>,
    pub marker: Option<EnvironmentMarker>,
}

#[derive(Debug, Clone)]
pub struct VersionSpecifier {
    pub operator: String,  // "==", ">=", "<=", ">", "<", "!=", "~=", "==="
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct EnvironmentMarker {
    pub raw: String,
}

impl EnvironmentMarker {
    pub fn evaluate(&self, env: &TargetEnvironment) -> bool {
        // Simplified evaluation
        // Full implementation would need a proper expression parser
        
        let marker = &self.raw;
        
        // Check for common patterns
        if marker.contains("python_version") {
            if let Some(required) = extract_version_requirement(marker, "python_version") {
                return compare_versions(&env.python_version, &required.0, &required.1);
            }
        }
        
        if marker.contains("sys_platform") {
            if let Some(required) = extract_string_requirement(marker, "sys_platform") {
                return env.sys_platform == required;
            }
        }
        
        if marker.contains("platform_system") {
            if let Some(required) = extract_string_requirement(marker, "platform_system") {
                return env.platform_system == required;
            }
        }
        
        // Default to true if we can't parse
        true
    }
}

#[derive(Debug, Clone)]
pub struct TargetEnvironment {
    pub python_version: String,
    pub sys_platform: String,
    pub platform_system: String,
    pub platform_machine: String,
}

impl Default for TargetEnvironment {
    fn default() -> Self {
        Self {
            python_version: "3.11".to_string(),
            sys_platform: std::env::consts::OS.to_string(),
            platform_system: std::env::consts::OS.to_string(),
            platform_machine: std::env::consts::ARCH.to_string(),
        }
    }
}

pub fn parse_requirement(req_str: &str) -> Result<DependencySpecifier> {
    let caps = PEP508_PATTERN.captures(req_str.trim())
        .ok_or_else(|| anyhow!("Invalid PEP 508 requirement: {}", req_str))?;

    let name = caps.name("name")
        .ok_or_else(|| anyhow!("Missing package name"))?
        .as_str()
        .to_string();

    let extras = caps.name("extras")
        .map(|m| m.as_str().split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let version_specs = if let Some(spec_str) = caps.name("specifier") {
        parse_version_specifiers(spec_str.as_str().trim())?
    } else {
        Vec::new()
    };

    let marker = caps.name("marker").map(|m| EnvironmentMarker {
        raw: m.as_str().to_string(),
    });

    Ok(DependencySpecifier {
        name,
        extras,
        version_specs,
        marker,
    })
}

fn parse_version_specifiers(spec_str: &str) -> Result<Vec<VersionSpecifier>> {
    if spec_str.is_empty() {
        return Ok(Vec::new());
    }

    let mut specs = Vec::new();
    
    // Remove parentheses
    let spec_str = spec_str.trim().trim_start_matches('(').trim_end_matches(')').trim();
    
    // Split by comma for multiple specifiers
    for part in spec_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Try to extract operator and version
        let operators = ["===", "~=", "!=", "<=", ">=", "==", "<", ">"];
        let mut found = false;
        
        for op in &operators {
            if part.starts_with(op) {
                let version = part[op.len()..].trim().to_string();
                specs.push(VersionSpecifier {
                    operator: op.to_string(),
                    version,
                });
                found = true;
                break;
            }
        }
        
        if !found && !part.is_empty() {
            // Assume it's just a version without operator (implicit ==)
            specs.push(VersionSpecifier {
                operator: "==".to_string(),
                version: part.to_string(),
            });
        }
    }
    
    Ok(specs)
}

fn extract_version_requirement(marker: &str, key: &str) -> Option<(String, String)> {
    // Very simple extraction for patterns like: python_version >= "3.8"
    let pattern = format!(r#"{}\s*(>=|<=|>|<|==|!=)\s*["']([^"']+)["']"#, key);
    if let Ok(re) = Regex::new(&pattern) {
        if let Some(caps) = re.captures(marker) {
            if let (Some(op), Some(ver)) = (caps.get(1), caps.get(2)) {
                return Some((op.as_str().to_string(), ver.as_str().to_string()));
            }
        }
    }
    None
}

fn extract_string_requirement(marker: &str, key: &str) -> Option<String> {
    // Simple extraction for patterns like: sys_platform == "linux"
    let pattern = format!(r#"{}\s*==\s*["']([^"']+)["']"#, key);
    if let Ok(re) = Regex::new(&pattern) {
        if let Some(caps) = re.captures(marker) {
            if let Some(val) = caps.get(1) {
                return Some(val.as_str().to_string());
            }
        }
    }
    None
}

fn compare_versions(actual: &str, op: &str, required: &str) -> bool {
    // Very simplified version comparison
    match op {
        ">=" => actual >= required,
        "<=" => actual <= required,
        ">" => actual > required,
        "<" => actual < required,
        "==" => actual == required,
        "!=" => actual != required,
        _ => true,
    }
}
