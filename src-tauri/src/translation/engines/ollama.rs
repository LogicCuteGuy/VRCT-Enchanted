// Ollama translation engine

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::translation::Translator;
use crate::utils::error::{Result, VrctError};

const SYSTEM_PROMPT_TEMPLATE: &str = r#"You are a helpful translation assistant.
Supported languages:
{supported_languages}

Translate the user provided text from {input_lang} to {output_lang}.
Return ONLY the translated text. Do not add quotes or extra commentary."#;

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    name: String,
}

/// Ollama translator (local LLM server)
pub struct OllamaTranslator {
    client: Client,
    base_url: String,
    model: Arc<RwLock<Option<String>>>,
    supported_languages: Vec<String>,
}

impl OllamaTranslator {
    /// Create a new Ollama translator
    pub fn new() -> Self {
        let supported_languages = vec![
            "Afrikaans", "Arabic", "Armenian", "Azerbaijani", "Belarusian", "Bosnian",
            "Bulgarian", "Catalan", "Chinese", "Croatian", "Czech", "Danish", "Dutch",
            "English", "Estonian", "Finnish", "French", "Galician", "German", "Greek",
            "Hebrew", "Hindi", "Hungarian", "Icelandic", "Indonesian", "Italian", "Japanese",
            "Kannada", "Kazakh", "Korean", "Latvian", "Lithuanian", "Macedonian", "Malay",
            "Marathi", "Maori", "Nepali", "Norwegian", "Persian", "Polish", "Portuguese",
            "Romanian", "Russian", "Serbian", "Slovak", "Slovenian", "Spanish", "Swahili",
            "Swedish", "Tagalog", "Tamil", "Thai", "Turkish", "Ukrainian", "Urdu",
            "Vietnamese", "Welsh",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            client: Client::builder()
                .timeout(Duration::from_millis(200))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: "http://localhost:11434".to_string(),
            model: Arc::new(RwLock::new(None)),
            supported_languages,
        }
    }

    /// Check if Ollama is available
    pub async fn check_connection(&self) -> Result<bool> {
        let response = self.client.get(&self.base_url).send().await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get available models
    pub async fn get_model_list(&self) -> Result<Vec<String>> {
        if !self.check_connection().await? {
            return Err(VrctError::Translation(
                "Ollama server not available".to_string(),
            ));
        }

        let models_url = format!("{}/api/tags", self.base_url);
        let response = self
            .client
            .get(&models_url)
            .send()
            .await
            .map_err(|e| VrctError::Translation(format!("Failed to fetch models: {}", e)))?;

        let models_response: ModelsResponse = response
            .json()
            .await
            .map_err(|e| VrctError::Translation(format!("Failed to parse models response: {}", e)))?;

        let mut models: Vec<String> = models_response
            .models
            .into_iter()
            .map(|model| model.name)
            .collect();

        models.sort();
        Ok(models)
    }

    /// Set the model to use
    pub async fn set_model(&self, model: String) -> Result<()> {
        let models = self.get_model_list().await?;
        if models.contains(&model) {
            let mut m = self.model.write().await;
            *m = Some(model);
            Ok(())
        } else {
            Err(VrctError::Translation(format!(
                "Model '{}' not available",
                model
            )))
        }
    }

    /// Get the current model
    pub async fn get_model(&self) -> Option<String> {
        self.model.read().await.clone()
    }

    fn format_system_prompt(&self, input_lang: &str, output_lang: &str) -> String {
        SYSTEM_PROMPT_TEMPLATE
            .replace(
                "{supported_languages}",
                &self.supported_languages.join(", "),
            )
            .replace("{input_lang}", input_lang)
            .replace("{output_lang}", output_lang)
    }
}

impl Default for OllamaTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Translator for OllamaTranslator {
    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let model = self.model.read().await;
        let model = model.as_ref().ok_or_else(|| {
            VrctError::Translation("Ollama model not set".to_string())
        })?;

        let system_prompt = self.format_system_prompt(source_lang, target_lang);

        let request = OllamaRequest {
            model: model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: text.to_string(),
                },
            ],
            stream: false,
        };

        let chat_url = format!("{}/api/chat", self.base_url);
        let response = self
            .client
            .post(&chat_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| VrctError::Translation(format!("Ollama API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(VrctError::Translation(format!(
                "Ollama API returned error {}: {}",
                status, error_text
            )));
        }

        let ollama_response: OllamaResponse = response.json().await.map_err(|e| {
            VrctError::Translation(format!("Failed to parse Ollama response: {}", e))
        })?;

        Ok(ollama_response.message.content.trim().to_string())
    }

    async fn validate_config(&self) -> Result<bool> {
        let has_model = self.model.read().await.is_some();
        let is_connected = self.check_connection().await.unwrap_or(false);
        Ok(has_model && is_connected)
    }

    fn name(&self) -> &str {
        "Ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_translator_creation() {
        let translator = OllamaTranslator::new();
        assert_eq!(translator.name(), "Ollama");
    }

    #[tokio::test]
    async fn test_ollama_translator_without_model() {
        let translator = OllamaTranslator::new();
        let result = translator.translate("Hello", "English", "Japanese").await;
        assert!(result.is_err());
    }
}
