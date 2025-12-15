use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub options: Option<OllamaOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaOptions {
    pub temperature: f32,
    pub num_predict: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub done: bool,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            base_url: OLLAMA_BASE_URL.to_string(),
        }
    }

    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).timeout(Duration::from_secs(2)).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);
        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: 0.7,
                num_predict: None,
            }),
        };

        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama API returned error: {}", response.status());
        }

        let ollama_resp: OllamaResponse = response.json().await?;
        Ok(ollama_resp.response)
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        #[derive(Deserialize)]
        struct ModelList {
            models: Vec<ModelInfo>,
        }
        
        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let list: ModelList = response.json().await?;
        Ok(list.models.into_iter().map(|m| m.name).collect())
    }
}
