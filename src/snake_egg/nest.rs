use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use crate::snake_egg::dna::DNA;
use crate::snake_egg::embryo::Embryo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nest {
    pub root: PathBuf,
    pub clutches: Vec<String>,
}

impl Nest {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            clutches: Vec::new(),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.root).await?;
        fs::create_dir_all(self.proteins_dir()).await?;
        
        let registry_path = self.registry_path();
        if !registry_path.exists() {
            let empty_nest = Nest::new(self.root.clone());
            let content = serde_json::to_string_pretty(&empty_nest)?;
            fs::write(registry_path, content).await?;
        }

        Ok(())
    }

    pub fn proteins_dir(&self) -> PathBuf {
        self.root.join("proteins")
    }

    pub fn clutch_dir(&self, clutch_name: &str) -> PathBuf {
        self.root.join(clutch_name)
    }

    pub fn registry_path(&self) -> PathBuf {
        self.root.join("registry.json")
    }

    pub async fn create_clutch(&mut self, clutch_name: &str) -> Result<()> {
        if self.clutches.contains(&clutch_name.to_string()) {
            return Err(anyhow!("Clutch {} already exists", clutch_name));
        }

        let clutch_path = self.clutch_dir(clutch_name);
        fs::create_dir_all(&clutch_path).await?;

        self.clutches.push(clutch_name.to_string());
        self.save_registry().await?;

        Ok(())
    }

    pub async fn lay_egg(&self, dna: DNA, clutch_name: &str) -> Result<(Embryo, Embryo)> {
        // Create both organic and metallic eggs
        let module_base = self.root.join(clutch_name).join(&dna.identity.name);
        
        // Organic (Python) egg
        let organic_path = module_base.join("organic");
        fs::create_dir_all(&organic_path).await?;
        fs::create_dir_all(organic_path.join("src")).await?;
        
        // Metallic (Rust) egg  
        let metallic_path = module_base.join("metallic");
        fs::create_dir_all(&metallic_path).await?;
        fs::create_dir_all(metallic_path.join("src")).await?;
        
        // Save DNA in both locations
        let dna_organic_path = organic_path.join(format!("{}.dna", dna.identity.name));
        let dna_metallic_path = metallic_path.join(format!("{}.dna", dna.identity.name));
        dna.save(&dna_organic_path).await?;
        dna.save(&dna_metallic_path).await?;

        let organic = Embryo::new(dna.clone(), organic_path, crate::snake_egg::embryo::EggType::Organic);
        let metallic = Embryo::new(dna, metallic_path, crate::snake_egg::embryo::EggType::Metallic);

        Ok((organic, metallic))
    }

    async fn save_registry(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(self.registry_path(), content).await?;
        Ok(())
    }

    pub async fn load(root: PathBuf) -> Result<Self> {
        let registry_path = root.join("registry.json");
        if !registry_path.exists() {
            return Ok(Nest::new(root));
        }

        let content = fs::read_to_string(registry_path).await?;
        let nest: Nest = serde_json::from_str(&content)?;
        Ok(nest)
    }

    pub async fn list_eggs(&self, clutch_name: &str) -> Result<Vec<String>> {
        let clutch_path = self.clutch_dir(clutch_name);
        if !clutch_path.exists() {
            return Ok(Vec::new());
        }

        let mut eggs = Vec::new();
        let mut entries = fs::read_dir(clutch_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    eggs.push(name.to_string());
                }
            }
        }

        Ok(eggs)
    }
}
