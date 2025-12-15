use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use crate::snake_egg::dna::GestationMilestone;
use crate::snake_egg::embryo::Embryo;

/// Chrono-Capacitus: Time-based resource capacity allocation
/// Mirrors natural life where maturity breeds resource access and efficiency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronoCapacitus {
    pub model: GeminiModel,
    pub api_call_interval_seconds: u64,
    pub max_tokens_per_call: usize,
    pub priority_level: u8,  // 1-10, higher = more access
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GeminiModel {
    Flash2_0,      // Rapid unconscious growth (zero-quota)
    Pro2_0,        // Developing proficiency
    Flash2_5,      // Cutting-edge speed for juveniles
    Pro2_5,        // Maximum capability for adults
}

impl GeminiModel {
    pub fn api_name(&self) -> &'static str {
        match self {
            GeminiModel::Flash2_0 => "gemini-2.0-flash-exp",
            GeminiModel::Pro2_0 => "gemini-2.0-pro",
            GeminiModel::Flash2_5 => "gemini-2.5-flash",
            GeminiModel::Pro2_5 => "gemini-2.5-pro",
        }
    }

    pub fn cost_factor(&self) -> f64 {
        match self {
            GeminiModel::Flash2_0 => 0.0,   // Zero-quota
            GeminiModel::Pro2_0 => 1.0,
            GeminiModel::Flash2_5 => 0.5,
            GeminiModel::Pro2_5 => 2.0,
        }
    }
}

impl ChronoCapacitus {
    /// Allocate resources based on egg's maturity, size, and protein value
    pub fn allocate(embryo: &Embryo) -> Self {
        let milestone = &embryo.current_stage.milestone;
        let code_size = embryo.estimate_code_size();
        let protein_bonus = embryo.dna.dependencies.proteins.len() as f64 * 0.1;
        
        // Base allocation from milestone
        let (base_model, base_interval, base_tokens, base_priority) = match milestone {
            GestationMilestone::Zygote => {
                // Rapid unconscious growth - max frequency, minimal model
                (GeminiModel::Flash2_0, 5, 1024, 1)
            },
            GestationMilestone::Embryo => {
                // Still rapid, starting to use better models for key decisions
                (GeminiModel::Flash2_0, 10, 2048, 2)
            },
            GestationMilestone::Fetus => {
                // Transitioning to Pro for complex logic
                (GeminiModel::Pro2_0, 30, 4096, 4)
            },
            GestationMilestone::Hatchling => {
                // Using cutting-edge flash for speed + quality
                (GeminiModel::Flash2_5, 60, 4096, 6)
            },
            GestationMilestone::Juvenile => {
                // Optimization phase - strategic calls
                (GeminiModel::Flash2_5, 120, 8192, 7)
            },
            GestationMilestone::Adult => {
                // Final polish - maximum capability, infrequent
                (GeminiModel::Pro2_5, 300, 8192, 10)
            },
        };

        // Adjust for code size
        let size_multiplier = if code_size > 5000 {
            1.5  // Larger eggs need more powerful models
        } else if code_size > 1000 {
            1.2
        } else {
            1.0
        };

        // Upgrade model if egg is large and has high fitness
        let model = if size_multiplier > 1.2 && embryo.fitness_score > 0.7 {
            match base_model {
                GeminiModel::Flash2_0 => GeminiModel::Pro2_0,
                GeminiModel::Pro2_0 => GeminiModel::Flash2_5,
                GeminiModel::Flash2_5 => GeminiModel::Pro2_5,
                m => m,
            }
        } else {
            base_model
        };

        // Priority boost from proteins (harvested knowledge)
        let priority = (base_priority as f64 + protein_bonus).min(10.0) as u8;

        ChronoCapacitus {
            model,
            api_call_interval_seconds: base_interval,
            max_tokens_per_call: (base_tokens as f64 * size_multiplier) as usize,
            priority_level: priority,
        }
    }

    /// Check if egg is eligible for API call based on last call time
    pub fn can_make_call(&self, last_call: SystemTime) -> bool {
        let elapsed = SystemTime::now()
            .duration_since(last_call)
            .unwrap_or(Duration::from_secs(u64::MAX));
        
        elapsed.as_secs() >= self.api_call_interval_seconds
    }

    /// Calculate resource budget for a clutch
    pub fn clutch_budget(embryos: &[Embryo]) -> ClutchResourceBudget {
        let mut budget = ClutchResourceBudget {
            total_priority: 0,
            allocations: Vec::new(),
        };

        for embryo in embryos {
            let allocation = ChronoCapacitus::allocate(embryo);
            budget.total_priority += allocation.priority_level as u32;
            budget.allocations.push((embryo.dna.identity.name.clone(), allocation));
        }

        budget
    }
}

#[derive(Debug, Clone)]
pub struct ClutchResourceBudget {
    pub total_priority: u32,
    pub allocations: Vec<(String, ChronoCapacitus)>,
}

impl ClutchResourceBudget {
    /// Get next egg to evolve based on priority and availability
    pub fn next_eligible(&self, last_calls: &std::collections::HashMap<String, SystemTime>) -> Option<String> {
        // Filter to eligible eggs
        let mut eligible: Vec<_> = self.allocations.iter()
            .filter(|(name, allocation)| {
                let last_call = last_calls.get(name)
                    .copied()
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                allocation.can_make_call(last_call)
            })
            .collect();

        // Sort by priority (descending)
        eligible.sort_by(|a, b| b.1.priority_level.cmp(&a.1.priority_level));

        eligible.first().map(|(name, _)| name.clone())
    }
}

impl Embryo {
    fn estimate_code_size(&self) -> usize {
        // Estimate based on gestation log length and milestone
        let base_size = match self.current_stage.milestone {
            GestationMilestone::Zygote => 50,
            GestationMilestone::Embryo => 200,
            GestationMilestone::Fetus => 800,
            GestationMilestone::Hatchling => 2000,
            GestationMilestone::Juvenile => 4000,
            GestationMilestone::Adult => 6000,
        };

        base_size + (self.gestation_log.len() * 10)
    }
}
