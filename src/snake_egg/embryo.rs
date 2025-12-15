use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use crate::snake_egg::dna::{DNA, GestationMilestone};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EggType {
    Organic,   // Python egg
    Metallic,  // Rust egg
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embryo {
    pub dna: DNA,
    pub module_path: PathBuf,
    pub current_stage: DevelopmentStage,
    pub gestation_log: Vec<GestationLogEntry>,
    pub fitness_score: f64,
    pub temperature: f64,  // Heat sharing metric
    pub egg_type: EggType,
    pub stalled_iterations: u32,  // Track if stuck
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentStage {
    pub milestone: GestationMilestone,
    pub progress_percentage: u8,
    pub criteria_met: Vec<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestationLogEntry {
    pub timestamp: u64,
    pub model: String,
    pub action: String,
    pub milestone: GestationMilestone,
}

impl Embryo {
    pub fn new(dna: DNA, module_path: PathBuf, egg_type: EggType) -> Self {
        let criteria_count = dna.self_actualization.success_criteria.len();
        Self {
            dna,
            module_path,
            current_stage: DevelopmentStage {
                milestone: GestationMilestone::Zygote,
                progress_percentage: 0,
                criteria_met: vec![false; criteria_count],
            },
            gestation_log: Vec::new(),
            fitness_score: 0.0,
            temperature: 0.0,
            egg_type,
            stalled_iterations: 0,
        }
    }

    pub async fn load(module_path: PathBuf) -> Result<Self> {
        let log_path = module_path.join("gestation_log.json");
        if log_path.exists() {
            let content = fs::read_to_string(log_path).await?;
            let embryo: Embryo = serde_json::from_str(&content)?;
            Ok(embryo)
        } else {
            let dna_name = module_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("module");
            let dna_path = module_path.join(format!("{}.dna", dna_name));
            let dna = DNA::load(&dna_path).await?;
            
            // Detect egg type from path
            let egg_type = if module_path.to_str().unwrap().contains("/organic/") {
                EggType::Organic
            } else if module_path.to_str().unwrap().contains("/metallic/") {
                EggType::Metallic
            } else {
                EggType::Organic  // Default
            };
            
            Ok(Embryo::new(dna, module_path, egg_type))
        }
    }

    pub async fn save(&self) -> Result<()> {
        let log_path = self.module_path.join("gestation_log.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(log_path, content).await?;
        Ok(())
    }

    pub fn log_action(&mut self, model: &str, action: &str) {
        self.gestation_log.push(GestationLogEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            model: model.to_string(),
            action: action.to_string(),
            milestone: self.current_stage.milestone.clone(),
        });
    }

    pub fn advance_milestone(&mut self) -> bool {
        if let Some(next) = self.current_stage.milestone.next() {
            self.current_stage.milestone = next.clone();
            self.current_stage.progress_percentage = next.progress_percentage();
            self.stalled_iterations = 0;  // Reset stall counter
            true
        } else {
            false
        }
    }

    pub fn is_hatched(&self) -> bool {
        self.current_stage.milestone == GestationMilestone::Adult &&
        self.current_stage.criteria_met.iter().all(|&met| met)
    }

    pub fn update_criteria(&mut self, index: usize, met: bool) {
        if index < self.current_stage.criteria_met.len() {
            self.current_stage.criteria_met[index] = met;
        }
    }

    pub fn calculate_fitness(&mut self) {
        let criteria_met_count = self.current_stage.criteria_met.iter().filter(|&&m| m).count();
        let total_criteria = self.current_stage.criteria_met.len() as f64;
        let criteria_score = if total_criteria > 0.0 {
            criteria_met_count as f64 / total_criteria
        } else {
            0.0
        };

        let milestone_score = self.current_stage.progress_percentage as f64 / 100.0;

        // Weighted average: 60% criteria, 40% milestone
        self.fitness_score = (criteria_score * 0.6) + (milestone_score * 0.4);
    }

    pub fn calculate_temperature(&mut self) {
        let milestone_heat = self.current_stage.milestone.progress_percentage() as f64;
        let fitness_heat = self.fitness_score * 100.0;
        let criteria_heat = (self.current_stage.criteria_met.iter()
            .filter(|&&m| m).count() as f64 / self.current_stage.criteria_met.len() as f64) * 100.0;
        
        // Weighted: 40% milestone, 40% fitness, 20% criteria
        self.temperature = (milestone_heat * 0.4) + (fitness_heat * 0.4) + (criteria_heat * 0.2);
    }

    pub fn is_failing(&self) -> bool {
        self.fitness_score < 0.2 ||
        (self.gestation_log.len() > 100 && self.current_stage.milestone == GestationMilestone::Zygote) ||
        self.temperature < 10.0 ||
        self.stalled_iterations > 20
    }

    pub fn record_stall(&mut self) {
        self.stalled_iterations += 1;
    }

    pub fn src_path(&self) -> PathBuf {
        match self.egg_type {
            EggType::Organic => self.module_path.join("src").join("__init__.py"),
            EggType::Metallic => self.module_path.join("src").join("lib.rs"),
        }
    }

    pub fn extract_intent(&self) -> String {
        format!(
            "Purpose: {}\nCriteria:\n{}\nCurrent Progress: {:?}",
            self.dna.self_actualization.purpose,
            self.dna.self_actualization.success_criteria.join("\n- "),
            self.current_stage.milestone
        )
    }
}
