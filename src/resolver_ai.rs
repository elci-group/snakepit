use crate::charmer::SnakeCharmer;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Conflict {
    pub package_name: String,
    pub constraint_a: String,
    pub source_a: String,
    pub constraint_b: String,
    pub source_b: String,
    pub available_versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub chosen_version: String,
    pub reasoning: String,
    pub confidence: f32,
    pub risks: String,
}

pub struct AIResolver {
    charmer: SnakeCharmer,
    conflict_cache: HashMap<String, Resolution>,
}

impl AIResolver {
    pub fn new() -> Result<Self> {
        Ok(Self {
            charmer: SnakeCharmer::new()?,
            conflict_cache: HashMap::new(),
        })
    }

    pub async fn resolve_conflict(&mut self, conflict: &Conflict) -> Result<Option<Resolution>> {
        // Check cache first
        let cache_key = format!("{}-{}-{}", conflict.package_name, conflict.constraint_a, conflict.constraint_b);
        if let Some(cached) = self.conflict_cache.get(&cache_key) {
            println!("💾 Using cached resolution for {}", conflict.package_name);
            return Ok(Some(cached.clone()));
        }

        // Build prompt
        let prompt = self.build_conflict_prompt(conflict);
        
        // Ask Gemini
        let response = self.charmer.ask(&prompt).await?;
        
        // Parse response
        if let Some(resolution) = self.parse_resolution(&response) {
            // Cache the result
            self.conflict_cache.insert(cache_key, resolution.clone());
            Ok(Some(resolution))
        } else {
            Ok(None)
        }
    }

    fn build_conflict_prompt(&self, conflict: &Conflict) -> String {
        format!(
            r#"You are a Python dependency expert. Analyze this version conflict:

Package: {}
Constraint A: {} (from {})
Constraint B: {} (from {})

Available versions: {}

Tasks:
1. Identify if there's a version that technically satisfies both constraints
2. If not, analyze changelogs to find the safest compromise
3. Consider: API stability, security patches, breaking changes
4. Recommend a version with confidence score (0-100)

Format your response EXACTLY as follows:
VERSION: <version>
CONFIDENCE: <0-100>
REASONING: <explanation>
RISKS: <potential issues>

Be concise and specific."#,
            conflict.package_name,
            conflict.constraint_a,
            conflict.source_a,
            conflict.constraint_b,
            conflict.source_b,
            conflict.available_versions.join(", ")
        )
    }

    fn parse_resolution(&self, response: &str) -> Option<Resolution> {
        let mut version = None;
        let mut confidence = None;
        let mut reasoning = String::new();
        let mut risks = String::new();

        for line in response.lines() {
            if line.starts_with("VERSION:") {
                version = Some(line.replace("VERSION:", "").trim().to_string());
            } else if line.starts_with("CONFIDENCE:") {
                if let Ok(conf) = line.replace("CONFIDENCE:", "").trim().parse::<f32>() {
                    confidence = Some(conf);
                }
            } else if line.starts_with("REASONING:") {
                reasoning = line.replace("REASONING:", "").trim().to_string();
            } else if line.starts_with("RISKS:") {
                risks = line.replace("RISKS:", "").trim().to_string();
            }
        }

        if let (Some(v), Some(c)) = (version, confidence) {
            Some(Resolution {
                chosen_version: v,
                reasoning,
                confidence: c,
                risks,
            })
        } else {
            None
        }
    }

    pub fn detect_conflicts(&self, dependencies: &[(String, String)]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        let mut package_constraints: HashMap<String, Vec<(String, String)>> = HashMap::new();

        // Group constraints by package
        for (package, constraint) in dependencies {
            package_constraints
                .entry(package.clone())
                .or_insert_with(Vec::new)
                .push((constraint.clone(), "user".to_string()));
        }

        // Find conflicts
        for (package, constraints) in package_constraints {
            if constraints.len() > 1 {
                // Simplified: just take first two conflicting constraints
                // In reality, we'd need to check if they actually conflict
                if let (Some((c1, s1)), Some((c2, s2))) = (constraints.get(0), constraints.get(1)) {
                    conflicts.push(Conflict {
                        package_name: package.clone(),
                        constraint_a: c1.clone(),
                        source_a: s1.clone(),
                        constraint_b: c2.clone(),
                        source_b: s2.clone(),
                        available_versions: vec![], // Would fetch from PyPI
                    });
                }
            }
        }

        conflicts
    }
}
