use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::snake_egg::embryo::Embryo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clutch {
    pub name: String,
    pub eggs: Vec<String>,
    pub sync_interval_minutes: u64,
    pub heat_map: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatExchange {
    pub from_egg: String,
    pub to_egg: String,
    pub knowledge: KnowledgePacket,
    pub thermal_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePacket {
    pub successful_pattern: String,
    pub approach: String,
    pub pitfalls_avoided: Vec<String>,
}

impl Clutch {
    pub fn new(name: String) -> Self {
        Self {
            name,
            eggs: Vec::new(),
            sync_interval_minutes: 30,
            heat_map: std::collections::HashMap::new(),
        }
    }

    pub fn add_egg(&mut self, module_name: String) {
        if !self.eggs.contains(&module_name) {
            self.eggs.push(module_name.clone());
            self.heat_map.insert(module_name, 0.0);
        }
    }

    pub fn remove_egg(&mut self, module_name: &str) {
        self.eggs.retain(|name| name != module_name);
        self.heat_map.remove(module_name);
    }

    pub async fn thermal_cycle(&mut self, embryos: &mut [Embryo]) -> Result<Vec<HeatExchange>> {
        let mut exchanges = Vec::new();
        
        // 1. Update temperature for all eggs
        for embryo in embryos.iter_mut() {
            embryo.calculate_temperature();
            self.heat_map.insert(embryo.dna.identity.name.clone(), embryo.temperature);
        }
        
        // 2. Identify hot and cool eggs
        let avg_temp: f64 = self.heat_map.values().sum::<f64>() / self.heat_map.len() as f64;
        
        // 3. Heat transfer: Hot → Cool
        for i in 0..embryos.len() {
            for j in 0..embryos.len() {
                if i == j { continue; }
                
                let thermal_delta = embryos[i].temperature - embryos[j].temperature;
                
                // Only transfer if delta > 20°C
                if thermal_delta > 20.0 {
                    let knowledge = self.extract_knowledge(&embryos[i]);
                    embryos[j].absorb_heat(&knowledge).await;
                    
                    exchanges.push(HeatExchange {
                        from_egg: embryos[i].dna.identity.name.clone(),
                        to_egg: embryos[j].dna.identity.name.clone(),
                        knowledge,
                        thermal_delta,
                    });
                }
            }
        }
        
        Ok(exchanges)
    }

    fn extract_knowledge(&self, embryo: &Embryo) -> KnowledgePacket {
        KnowledgePacket {
            successful_pattern: format!("Reached {:?} stage", embryo.current_stage.milestone),
            approach: embryo.dna.self_actualization.purpose.clone(),
            pitfalls_avoided: embryo.gestation_log.iter()
                .filter(|log| log.action.contains("error") || log.action.contains("failed"))
                .map(|log| log.action.clone())
                .collect(),
        }
    }

    pub fn sibling_similarity(&self, embryos: &[Embryo]) -> f64 {
        if embryos.len() < 2 {
            return 1.0;
        }

        let mut total_diff = 0.0;
        let mut comparison_count = 0;

        for i in 0..embryos.len() {
            for j in (i+1)..embryos.len() {
                total_diff += (embryos[i].temperature - embryos[j].temperature).abs();
                comparison_count += 1;
            }
        }

        if comparison_count > 0 {
            1.0 - (total_diff / (comparison_count as f64 * 100.0))  // Normalize to 0-1
        } else {
            1.0
        }
    }
}

impl Embryo {
    pub async fn absorb_heat(&mut self, knowledge: &KnowledgePacket) {
        self.log_action("heat_sharing", &format!("Absorbed knowledge: {}", knowledge.approach));
        
        // Slight temperature boost from learning
        self.temperature += 5.0;
    }
}
