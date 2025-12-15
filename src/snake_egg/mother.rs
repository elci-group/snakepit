use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::Mutex;
use crate::charmer::SnakeCharmer;
use crate::snake_egg::embryo::{Embryo, EggType};
use crate::snake_egg::protein::Protein;
use crate::snake_egg::nest::Nest;
use crate::snake_egg::clutch::Clutch;
use crate::snake_egg::chrono_capacitus::{ChronoCapacitus, GeminiModel};

pub struct Mother {
    charmer: Arc<Mutex<SnakeCharmer>>,
    nest: Arc<Mutex<Nest>>,
    protein_harvest: Vec<Protein>,
    last_api_calls: HashMap<String, SystemTime>,  // Track API call times
}

impl Mother {
    pub fn new(charmer: Arc<Mutex<SnakeCharmer>>, nest: Arc<Mutex<Nest>>) -> Self {
        Self {
            charmer,
            nest,
            protein_harvest: Vec::new(),
            last_api_calls: HashMap::new(),
        }
    }

    /// Darwinian Diet: Cannibalize failing eggs
    pub async fn darwinian_cycle(&mut self, clutch: &mut Clutch, embryos: &mut Vec<Embryo>) -> Result<Vec<Protein>> {
        let mut harvested = Vec::new();
        let mut to_remove = Vec::new();
        
        for (idx, embryo) in embryos.iter().enumerate() {
            if embryo.is_failing() {
                println!("🦖 Cannibalizing failing egg: {} (temp: {:.1}°C, fitness: {:.2})",
                    embryo.dna.identity.name, embryo.temperature, embryo.fitness_score);
                
                let proteins = self.cannibalize(embryo).await?;
                harvested.extend(proteins.clone());
                self.protein_harvest.extend(proteins);
                
                to_remove.push(idx);
                clutch.remove_egg(&embryo.dna.identity.name);
            }
        }
        
        // Remove from end to start to preserve indices
        for idx in to_remove.iter().rev() {
            embryos.remove(*idx);
        }
        
        Ok(harvested)
    }

    async fn cannibalize(&self, egg: &Embryo) -> Result<Vec<Protein>> {
        let mut proteins = Vec::new();
        
        if !egg.src_path().exists() {
            return Ok(proteins);
        }
        
        let src = tokio::fs::read_to_string(egg.src_path()).await?;
        
        // Extract working code snippets (simplified pattern matching)
        let lines: Vec<&str> = src.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains("fn ") || line.contains("def ") {
                // Found a function, extract it
                let snippet = self.extract_function(&lines, i);
                if !snippet.is_empty() {
                    proteins.push(Protein {
                        name: format!("harvested_{}", egg.dna.identity.name),
                        protein_type: crate::snake_egg::protein::ProteinType::Function,
                        provides: vec![egg.dna.self_actualization.purpose.clone()],
                        complexity: crate::snake_egg::protein::Complexity::Medium,
                        code: snippet,
                        metadata: crate::snake_egg::protein::ProteinMetadata {
                            author: Some("Mother (cannibalized)".to_string()),
                            created: Some(chrono::Utc::now().to_rfc3339()),
                            tags: vec!["harvested".to_string(), format!("{:?}", egg.dna.identity.species)],
                        },
                    });
                }
            }
        }
        
        Ok(proteins)
    }

    fn extract_function(&self, lines: &[&str], start: usize) -> String {
        let mut snippet = String::new();
        let mut brace_count = 0;
        let mut started = false;
        
        for line in &lines[start..] {
            if line.contains('{') || line.contains(':') {
                started = true;
            }
            if started {
                snippet.push_str(line);
                snippet.push('\n');
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;
                
                if brace_count == 0 && started {
                    break;
                }
            }
        }
        
        snippet
    }

    /// Dual egg orchestration: Organic leads, Metallic oxidizes
    pub async fn orchestrate_dual_eggs(&mut self, organic: &mut Embryo, metallic: &mut Embryo) -> Result<()> {
        // 1. Evolve organic egg first (Python, fast iteration)
        self.evolve_code(organic).await?;
        
        // 2. Extract intent from organic
        let intent = organic.extract_intent();
        
        // 3. Send intent to metallic egg to "oxidize" (translate to Rust)
        self.oxidize_intent(metallic, &intent).await?;
        
        Ok(())
    }

    async fn oxidize_intent(&self, metallic: &mut Embryo, intent: &str) -> Result<()> {
        let charmer = self.charmer.lock().await;
        
        let prompt = format!(
            "Convert this Python module intent into Rust with minimal dependencies:\n\n{}\n\n\
             Generate idiomatic Rust code using only std lib where possible.",
            intent
        );
        
        let response = charmer.ask(&prompt).await?;
        metallic.log_action("oxidation", "Received Rust implementation from Python intent");
        
        Ok(())
    }

    pub async fn nourish_embryo(&self, embryo: &mut Embryo, proteins: &[Protein]) -> Result<()> {
        let mut src_content = String::new();

        if embryo.src_path().exists() {
            src_content = tokio::fs::read_to_string(embryo.src_path()).await?;
        }

        for protein in proteins {
            if embryo.dna.dependencies.proteins.contains(&protein.name) {
                src_content = protein.inject_into_module(&src_content);
                embryo.log_action("mother", &format!("Injected protein: {}", protein.name));
            }
        }

        // Also inject harvested proteins
        for protein in &self.protein_harvest {
            src_content = protein.inject_into_module(&src_content);
        }

        tokio::fs::write(embryo.src_path(), src_content).await?;

        Ok(())
    }

    pub async fn evolve_code(&mut self, embryo: &mut Embryo) -> Result<()> {
        // Allocate resources based on maturity
        let allocation = ChronoCapacitus::allocate(embryo);
        
        // Check if eligible for API call
        let last_call = self.last_api_calls.get(&embryo.dna.identity.name)
            .copied()
            .unwrap_or(SystemTime::UNIX_EPOCH);
        
        if !allocation.can_make_call(last_call) {
            embryo.record_stall();
            return Ok(()); // Too soon, wait for next cycle
        }
        
        println!("🧬 Evolving {} using {} (priority: {}, interval: {}s)",
            embryo.dna.identity.name,
            allocation.model.api_name(),
            allocation.priority_level,
            allocation.api_call_interval_seconds
        );
        
        let charmer = self.charmer.lock().await;
        let prompt = self.build_evolution_prompt(embryo, &allocation);
        
        // Make API call with allocated model
        let response = charmer.ask(&prompt).await?;
        
        // Record call time
        self.last_api_calls.insert(
            embryo.dna.identity.name.clone(),
            SystemTime::now()
        );
        
        embryo.log_action(
            allocation.model.api_name(),
            &format!("Generated code evolution ({}T)", allocation.max_tokens_per_call)
        );

        Ok(())
    }

    fn build_evolution_prompt(&self, embryo: &Embryo, allocation: &ChronoCapacitus) -> String {
        let lang = match embryo.egg_type {
            EggType::Organic => "Python",
            EggType::Metallic => "Rust",
            EggType::Dual => "Rust/Python Hybrid",
        };
        
        let focus = match embryo.current_stage.milestone {
            crate::snake_egg::dna::GestationMilestone::Zygote => "Basic struct/trait definitions. Keep it simple.",
            crate::snake_egg::dna::GestationMilestone::Embryo => "Method signatures and stubs. Rapid scaffolding.",
            crate::snake_egg::dna::GestationMilestone::Fetus => "Core logic implementation. This is critical.",
            crate::snake_egg::dna::GestationMilestone::Hatchling => "Error handling and edge cases. Polish.",
            crate::snake_egg::dna::GestationMilestone::Juvenile => "Optimization and documentation. Refine.",
            crate::snake_egg::dna::GestationMilestone::Adult => "Final polish. Production-ready.",
        };
        
        format!(
            "Module: {} ({} | Model: {})\n\
             Purpose: {}\n\
             Current Stage: {:?}\n\
             Success Criteria:\n{}\n\
             \n\
             Generate {} code (max {} tokens) to advance this module.\n\
             Focus: {}\n\
             \n\
             Priority Level: {}/10",
            embryo.dna.identity.name,
            lang,
            allocation.model.api_name(),
            embryo.dna.self_actualization.purpose,
            embryo.current_stage.milestone,
            embryo.dna.self_actualization.success_criteria.join("\n- "),
            lang,
            allocation.max_tokens_per_call,
            focus,
            allocation.priority_level
        )
    }

    pub async fn evaluate_fitness(&self, embryo: &mut Embryo) -> Result<()> {
        let test_output = self.run_tests(&embryo.module_path).await?;
        
        let criteria = embryo.dna.self_actualization.success_criteria.clone();
        
        for (i, criterion) in criteria.iter().enumerate() {
            let met = test_output.contains(&format!("✓ {}", criterion));
            embryo.update_criteria(i, met);
        }

        embryo.calculate_fitness();
        embryo.log_action("mother", &format!("Fitness evaluated: {:.2}", embryo.fitness_score));

        Ok(())
    }

    async fn run_tests(&self, module_path: &std::path::Path) -> Result<String> {
        Ok(String::from("Test output"))
    }

    pub async fn check_hatch_readiness(&self, embryo: &Embryo) -> bool {
        embryo.is_hatched()
    }
}
