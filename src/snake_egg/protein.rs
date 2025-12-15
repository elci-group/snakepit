use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protein {
    pub name: String,
    pub protein_type: ProteinType,
    pub provides: Vec<String>,
    pub complexity: Complexity,
    pub code: String,
    pub metadata: ProteinMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProteinType {
    Function,
    Struct,
    Trait,
    Impl,
    Module,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProteinMetadata {
    pub author: Option<String>,
    pub created: Option<String>,
    pub tags: Vec<String>,
}

impl Protein {
    pub async fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        Self::parse(&content)
    }

    fn parse(content: &str) -> Result<Self> {
        let mut lines = content.lines();
        let mut metadata_lines = Vec::new();
        let mut code_lines = Vec::new();
        let mut in_metadata = false;

        for line in lines {
            if line.trim().starts_with("// @") {
                in_metadata = true;
                metadata_lines.push(line.trim().trim_start_matches("//").trim());
            } else if in_metadata && line.trim().starts_with("//") {
                metadata_lines.push(line.trim().trim_start_matches("//").trim());
            } else {
                in_metadata = false;
                code_lines.push(line);
            }
        }

        let name = Self::extract_metadata(&metadata_lines, "name")
            .unwrap_or_else(|| "unnamed".to_string());
        
        let protein_type_str = Self::extract_metadata(&metadata_lines, "type")
            .unwrap_or_else(|| "function".to_string());
        let protein_type = match protein_type_str.as_str() {
            "struct" => ProteinType::Struct,
            "trait" => ProteinType::Trait,
            "impl" => ProteinType::Impl,
            "module" => ProteinType::Module,
            _ => ProteinType::Function,
        };

        let provides_str = Self::extract_metadata(&metadata_lines, "provides")
            .unwrap_or_else(|| String::new());
        let provides = provides_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

        let complexity_str = Self::extract_metadata(&metadata_lines, "complexity")
            .unwrap_or_else(|| "medium".to_string());
        let complexity = match complexity_str.as_str() {
            "low" => Complexity::Low,
            "high" => Complexity::High,
            _ => Complexity::Medium,
        };

        let code = code_lines.join("\n");

        Ok(Protein {
            name,
            protein_type,
            provides,
            complexity,
            code,
            metadata: ProteinMetadata {
                author: None,
                created: None,
                tags: Vec::new(),
            },
        })
    }

    fn extract_metadata(lines: &[&str], key: &str) -> Option<String> {
        for line in lines {
            if let Some(value) = line.strip_prefix(&format!("@{}: ", key)) {
                return Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix(&format!("@{}:", key)) {
                return Some(value.trim().to_string());
            }
        }
        None
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        let mut content = String::new();
        content.push_str(&format!("// @name: {}\n", self.name));
        content.push_str(&format!("// @type: {:?}\n", self.protein_type));
        content.push_str(&format!("// @provides: {}\n", self.provides.join(", ")));
        content.push_str(&format!("// @complexity: {:?}\n", self.complexity));
        content.push_str("\n");
        content.push_str(&self.code);

        fs::write(path, content).await?;
        Ok(())
    }

    pub fn inject_into_module(&self, module_content: &str) -> String {
        format!("{}\n\n{}", module_content, self.code)
    }
}
