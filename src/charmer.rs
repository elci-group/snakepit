use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use crate::native::ollama::OllamaClient;
use crate::native::hardware::HardwareCapabilities;
use crate::native::style::{dim, yellow, green};

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent";
const PYPI_API_URL: &str = "https://pypi.org/pypi";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStrategy {
    #[serde(rename = "command")]
    SimpleCommand(String),
    #[serde(rename = "script")]
    PythonScript(String),
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    generationConfig: GenerationConfig,
}

#[derive(Serialize)]
struct GenerationConfig {
    response_mime_type: String,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<ResponseContent>,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Option<Vec<ResponsePart>>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

#[derive(Deserialize)]
struct PyPiResponse {
    info: PyPiInfo,
}

#[derive(Deserialize)]
struct PyPiInfo {
    summary: Option<String>,
    description: Option<String>,
}

pub struct SnakeCharmer {
    client: Client,
    api_key: Option<String>,
    ollama: OllamaClient,
    hardware: HardwareCapabilities,
}

impl SnakeCharmer {
    pub fn new() -> Result<Self> {
        let api_key = env::var("GEMINI_API_KEY").ok();
        
        if api_key.is_none() {
            println!("{}", yellow("⚠️  GEMINI_API_KEY not set. Cloud features disabled."));
        }

        Ok(Self {
            client: Client::new(),
            api_key,
            ollama: OllamaClient::new(),
            hardware: HardwareCapabilities::detect(),
        })
    }

    async fn select_backend(&self) -> Option<Backend> {
        // 1. Check if Ollama is available and hardware is good (Prefer Local for privacy/offline)
        // Or should we prefer Cloud if available?
        // User said: "enhances security and offline functionality" -> implies Local is preferred or at least a strong option.
        // Let's try Local first if hardware allows, or if Cloud is missing.
        
        let ollama_available = self.ollama.is_available().await;
        
        if ollama_available && self.hardware.can_run_local_llm() {
            // Check for specific models? For now assume 'llama3' or 'mistral' or 'gemma'
            // We'll just use a default list
            if let Ok(models) = self.ollama.list_models().await {
                if !models.is_empty() {
                    // Pick the first one or a preferred one
                    let model = models.iter()
                        .find(|m| m.contains("gemma") || m.contains("llama3") || m.contains("mistral"))
                        .unwrap_or(&models[0])
                        .clone();
                    return Some(Backend::Ollama(model));
                }
            }
        }

        // 2. Fallback to Gemini if API key is present
        if self.api_key.is_some() {
            return Some(Backend::Gemini);
        }

        // 3. Fallback to Ollama even if hardware is weak (if it's the only option)
        if ollama_available {
             if let Ok(models) = self.ollama.list_models().await {
                if let Some(m) = models.first() {
                    return Some(Backend::Ollama(m.clone()));
                }
             }
        }

        None
    }

    pub async fn charm_package(&self, package_name: &str) -> Result<TestStrategy> {
        // 1. Fetch info from PyPI
        let pypi_info = self.fetch_pypi_info(package_name).await
            .unwrap_or_else(|_| PyPiInfo { 
                summary: Some("No summary available".to_string()), 
                description: None 
            });

        let description = pypi_info.description.unwrap_or_default();
        let summary = pypi_info.summary.unwrap_or_default();
        
        // Truncate description to avoid token limits (approx 2000 chars)
        let truncated_desc = if description.len() > 2000 {
            &description[..2000]
        } else {
            &description
        };

        // 2. Construct Prompt
        let prompt = format!(
            "I am installing the Python package '{}'. \
            Summary: {}\n\
            Description snippet: {}\n\n\
            Determine the best way to verify this package is installed and working.\n\
            - If it is a CLI tool or application, prefer a simple shell command (e.g., 'package --version', 'package --help', or 'which package').\n\
            - If it is a library, provide a simple Python script to import it and run a basic check.\n\
            \
            Return a JSON object with the following schema:\n\
            {{ \"type\": \"command\" | \"script\", \"content\": \"the command or script code\" }}\n\
            For 'command', the content is the shell command string.\n\
            For 'script', the content is the python code string (no markdown).\n",
            package_name, summary, truncated_desc
        );

        let backend = self.select_backend().await
            .context("No AI backend available. Set GEMINI_API_KEY or install Ollama.")?;

        let response_text = match backend {
            Backend::Gemini => {
                // 3. Call Gemini
                let request_body = GeminiRequest {
                    contents: vec![Content {
                        parts: vec![Part { text: prompt.clone() }],
                    }],
                    generationConfig: GenerationConfig {
                        response_mime_type: "application/json".to_string(),
                    },
                };

                let url = format!("{}?key={}", GEMINI_API_URL, self.api_key.as_ref().unwrap());

                let response = self.client
                    .post(&url)
                    .json(&request_body)
                    .send()
                    .await
                    .context("Failed to send request to Gemini API")?;

                if !response.status().is_success() {
                    let error_text = response.text().await?;
                    anyhow::bail!("Gemini API returned error: {}", error_text);
                }

                let gemini_response: GeminiResponse = response.json().await
                    .context("Failed to parse Gemini API response")?;

                gemini_response.candidates
                    .and_then(|c| c.into_iter().next())
                    .and_then(|c| c.content)
                    .and_then(|c| c.parts)
                    .and_then(|p| p.into_iter().next())
                    .map(|p| p.text)
                    .context("Gemini API returned no text content")?
            },
            Backend::Ollama(model) => {
                println!("{}", dim(format!("🤖 Using local model: {}", model)));
                // Append JSON instruction for Ollama since it might not support response_mime_type natively in all models
                let ollama_prompt = format!("{}\n\nRespond with valid JSON only.", prompt);
                self.ollama.generate(&model, &ollama_prompt).await?
            }
        };

        // 4. Parse JSON Response
        #[derive(Deserialize)]
        struct StrategyResponse {
            #[serde(rename = "type")]
            type_: String,
            content: String,
        }
        
        // Clean markdown code blocks if present (common in LLM output)
        let clean_text = response_text.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let strategy: StrategyResponse = serde_json::from_str(clean_text)
            .context(format!("Failed to parse JSON response: {}", clean_text))?;

        match strategy.type_.as_str() {
            "command" => Ok(TestStrategy::SimpleCommand(strategy.content)),
            "script" => Ok(TestStrategy::PythonScript(strategy.content)),
            _ => Ok(TestStrategy::PythonScript(strategy.content)), // Fallback
        }
    }

    async fn fetch_pypi_info(&self, package_name: &str) -> Result<PyPiInfo> {
        let url = format!("{}/{}/json", PYPI_API_URL, package_name);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch PyPI info");
        }

        let pypi_resp: PyPiResponse = response.json().await?;
        Ok(pypi_resp.info)
    }

    pub async fn diagnose_error(&self, command: &str, error_output: &str) -> Result<Option<String>> {
        let prompt = format!(
            "I ran the command `{}` and it failed with the following error:\n\n{}\n\n\
            Analyze this error. If it is caused by a missing Python package/module, identify the package name that needs to be installed.\n\
            Return ONLY the package name as a plain string. \
            If it is not a missing package issue or you are unsure, return 'UNKNOWN'.\n\
            Do not include any other text, markdown, or explanations.",
            command, error_output
        );

        let backend = self.select_backend().await
            .context("No AI backend available")?;

        match backend {
            Backend::Gemini => {
                let request_body = GeminiRequest {
                    contents: vec![Content {
                        parts: vec![Part { text: prompt }],
                    }],
                    generationConfig: GenerationConfig {
                        response_mime_type: "text/plain".to_string(),
                    },
                };

                let url = format!("{}?key={}", GEMINI_API_URL, self.api_key.as_ref().unwrap());

                let response = self.client.post(&url).json(&request_body).send().await?;
                if !response.status().is_success() { return Ok(None); }
                
                let gemini_response: GeminiResponse = response.json().await?;
                let text = gemini_response.candidates
                    .and_then(|c| c.into_iter().next())
                    .and_then(|c| c.content)
                    .and_then(|c| c.parts)
                    .and_then(|p| p.into_iter().next())
                    .map(|p| p.text)
                    .unwrap_or_default();
                    
                let result = text.trim().to_string();
                if result == "UNKNOWN" { Ok(None) } else { Ok(Some(result)) }
            },
            Backend::Ollama(model) => {
                let response = self.ollama.generate(&model, &prompt).await?;
                let result = response.trim().to_string();
                if result == "UNKNOWN" { Ok(None) } else { Ok(Some(result)) }
            }
        }
    }

    pub async fn ask(&self, prompt: &str) -> Result<String> {
        let backend = self.select_backend().await
            .context("No AI backend available")?;

        match backend {
            Backend::Gemini => {
                let request_body = GeminiRequest {
                    contents: vec![Content {
                        parts: vec![Part { text: prompt.to_string() }],
                    }],
                    generationConfig: GenerationConfig {
                        response_mime_type: "text/plain".to_string(),
                    },
                };

                let url = format!("{}?key={}", GEMINI_API_URL, self.api_key.as_ref().unwrap());
                let response = self.client.post(&url).json(&request_body).send().await?;
                let gemini_response: GeminiResponse = response.json().await?;
                
                gemini_response.candidates
                    .and_then(|c| c.into_iter().next())
                    .and_then(|c| c.content)
                    .and_then(|c| c.parts)
                    .and_then(|p| p.into_iter().next())
                    .map(|p| p.text)
                    .context("Gemini API returned no text content")
            },
            Backend::Ollama(model) => {
                self.ollama.generate(&model, prompt).await
            }
        }
    }
    pub async fn consult_oracle(&self, prompt: &str) -> Result<String> {
        self.ask(prompt).await
    }

    pub async fn analyze_uninstall_risk(&self, package: &str, dependents: &[String]) -> Result<String> {
        let prompt = format!(
            "I am about to uninstall the Python package '{}'. \
            The following installed packages depend on it: {:?}. \
            \
            Please analyze the risk of this operation. \
            Explain what functionality might break in the dependent packages. \
            Also mention if this package is commonly used by other tools or scripts (e.g. numpy, requests). \
            \
            Keep the response concise (under 3 sentences).",
            package, dependents
        );

        self.ask(&prompt).await
    }
}

enum Backend {
    Gemini,
    Ollama(String),
}
