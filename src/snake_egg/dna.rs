use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DNA {
    pub identity: Identity,
    pub self_actualization: SelfActualization,
    pub gestation_milestones: GestationMilestones,
    pub dependencies: Dependencies,
    pub evolution_parameters: EvolutionParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub species: Species,
    pub generation: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Species {
    Service,
    Utility,
    Model,
    Handler,
    Controller,
    Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfActualization {
    pub purpose: String,
    pub success_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestationMilestones {
    pub zygote: String,
    pub embryo: String,
    pub fetus: String,
    pub hatchling: String,
    pub juvenile: String,
    pub adult: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dependencies {
    pub proteins: Vec<String>,
    pub maternal_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionParameters {
    pub mutation_rate: f64,
    pub learning_rate: f64,
    pub fitness_function: String,
}

impl DNA {
    pub async fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let dna: DNA = toml::from_str(&content)?;
        dna.validate()?;
        Ok(dna)
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string).await?;
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        if self.identity.name.is_empty() {
            return Err(anyhow!("DNA identity name cannot be empty"));
        }

        if self.self_actualization.purpose.is_empty() {
            return Err(anyhow!("DNA purpose cannot be empty"));
        }

        if self.self_actualization.success_criteria.is_empty() {
            return Err(anyhow!("DNA must have at least one success criterion"));
        }

        if self.evolution_parameters.mutation_rate < 0.0 || self.evolution_parameters.mutation_rate > 1.0 {
            return Err(anyhow!("Mutation rate must be between 0.0 and 1.0"));
        }

        if self.evolution_parameters.learning_rate < 0.0 || self.evolution_parameters.learning_rate > 1.0 {
            return Err(anyhow!("Learning rate must be between 0.0 and 1.0"));
        }

        Ok(())
    }

    pub fn module_path(&self, nest_root: &Path, clutch_name: &str) -> PathBuf {
        nest_root
            .join(clutch_name)
            .join(&self.identity.name)
    }

    pub fn dna_path(&self, nest_root: &Path, clutch_name: &str) -> PathBuf {
        self.module_path(nest_root, clutch_name)
            .join(format!("{}.dna", self.identity.name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GestationMilestone {
    Zygote,
    Embryo,
    Fetus,
    Hatchling,
    Juvenile,
    Adult,
}

impl GestationMilestone {
    pub fn progress_percentage(&self) -> u8 {
        match self {
            GestationMilestone::Zygote => 0,
            GestationMilestone::Embryo => 16,
            GestationMilestone::Fetus => 33,
            GestationMilestone::Hatchling => 50,
            GestationMilestone::Juvenile => 75,
            GestationMilestone::Adult => 100,
        }
    }

    pub fn next(&self) -> Option<GestationMilestone> {
        match self {
            GestationMilestone::Zygote => Some(GestationMilestone::Embryo),
            GestationMilestone::Embryo => Some(GestationMilestone::Fetus),
            GestationMilestone::Fetus => Some(GestationMilestone::Hatchling),
            GestationMilestone::Hatchling => Some(GestationMilestone::Juvenile),
            GestationMilestone::Juvenile => Some(GestationMilestone::Adult),
            GestationMilestone::Adult => None,
        }
    }
}

impl std::fmt::Display for Species {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Species::Service => write!(f, "Service"),
            Species::Utility => write!(f, "Utility"),
            Species::Model => write!(f, "Model"),
            Species::Handler => write!(f, "Handler"),
            Species::Controller => write!(f, "Controller"),
            Species::Repository => write!(f, "Repository"),
        }
    }
}

impl std::str::FromStr for Species {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "service" => Ok(Species::Service),
            "utility" => Ok(Species::Utility),
            "model" => Ok(Species::Model),
            "handler" => Ok(Species::Handler),
            "controller" => Ok(Species::Controller),
            "repository" => Ok(Species::Repository),
            _ => Err(anyhow!("Unknown species: {}", s)),
        }
    }
}

impl std::fmt::Display for GestationMilestone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GestationMilestone::Zygote => write!(f, "Zygote"),
            GestationMilestone::Embryo => write!(f, "Embryo"),
            GestationMilestone::Fetus => write!(f, "Fetus"),
            GestationMilestone::Hatchling => write!(f, "Hatchling"),
            GestationMilestone::Juvenile => write!(f, "Juvenile"),
            GestationMilestone::Adult => write!(f, "Adult"),
        }
    }
}

impl Default for GestationMilestones {
    fn default() -> Self {
        Self {
            zygote: "Initial structure created".to_string(),
            embryo: "Basic methods implemented".to_string(),
            fetus: "Core logic functional".to_string(),
            hatchling: "Tests passing".to_string(),
            juvenile: "Optimized and documented".to_string(),
            adult: "Production ready".to_string(),
        }
    }
}

impl Default for EvolutionParameters {
    fn default() -> Self {
        Self {
            mutation_rate: 0.1,
            learning_rate: 0.5,
            fitness_function: "standard".to_string(),
        }
    }
}
