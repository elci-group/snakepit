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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
