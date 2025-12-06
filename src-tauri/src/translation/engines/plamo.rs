// Plamo translation engine

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::translation::Translator;
use crate::utils::error::{Result, VrctError};

const BASE_URL: &str = "https://api.platform.preferredai.jp/v1";
const SYSTEM_PROMPT_TEMPLATE: &str = r#"You are a helpful translation assistant.
Supported languages:
{supported_languages}

Translate the user provided text from {input_lang} to {output_lang}.
Return ONLY the translated text. Do not add quotes or extra commentary."#;

#[derive(Debug, Serialize)]
struct PlamoRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct PlamoResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
}

/// Plamo API translator
pub struct PlamoTranslator {
    client: Client,
    api_key: Arc<RwLock<Option<String>>>,
    model: Arc<RwLock<Option<String>>>,
    supported_languages: Vec<String>,
}

impl PlamoTranslator {
    /// Create a new Plamo translator
    pub fn new() -> Self {
        let supported_languages = vec![
            "English", "Japanese", "Korean", "French", "German", "Spanish", "Portuguese",
            "Russian", "Italian", "Dutch", "Polish", "Turkish", "Arabic", "Hindi", "Thai",
            "Vietnamese", "Indonesian", "Malay", "Filipino", "Swedish", "Finnish", "Danish",
            "Norwegian", "Romanian", "Czech", "Hungarian", "Greek", "Hebrew",
            "Simplified Chinese", "Traditional Chinese",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            client: Client::new(),
            api_key: Arc::new(RwLock::new(None)),
            model: Arc::new(RwLock::new(None)),
            supported_languages,
        }
    }

    /// Set the API key
    pub async fn set_api_key(&self, api_key: String) -> Result<()> {
        // Validate the API key by listing models
        let models_url = format!("{}/models", BASE_URL);
        let response = self
            .client
            .get(&models_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| {
                VrctError::Translation(format!("Failed to validate Plamo API key: {}", e))
            })?;

        if response.status().is_success() {
            let mut key = self.api_key.write().await;
            *key = Some(api_key);
            Ok(())
        } else {
            Err(VrctError::Translation(format!(
                "Invalid Plamo API key: {}",
                response.status()
            )))
        }
    }

    /// Get the current API key
    pub async fn get_api_key(&self) -> Option<String> {
        self.api_key.read().await.clone()
    }

    /// Get available models
    pub async fn get_model_list(&self) -> Result<Vec<String>> {
        let api_key = self.api_key.read().await;
        let api_key = api_key.as_ref().ok_or_else(|| {
            VrctError::Translation("Plamo API key not set".to_string())
        })?;

        let models_url = format!("{}/models", BASE_URL);
        let response = self
            .client
            .get(&models_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| VrctError::Translation(format!("Failed to fetch models: {}", e)))?;

        let models_response: ModelsResponse = response
            .json()
            .await
            .map_err(|e| VrctError::Translation(format!("Failed to parse models response: {}", e)))?;

        let mut models: Vec<String> = models_response
            .data
            .into_iter()
            .map(|model| model.id)
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

impl Default for PlamoTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Translator for PlamoTranslator {
    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let api_key = self.api_key.read().await;
        let api_key = api_key.as_ref().ok_or_else(|| {
            VrctError::Translation("Plamo API key not set".to_string())
        })?;

        let model = self.model.read().await;
        let model = model.as_ref().ok_or_else(|| {
            VrctError::Translation("Plamo model not set".to_string())
        })?;

        let system_prompt = self.format_system_prompt(source_lang, target_lang);

        let request = PlamoRequest {
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
            temperature: 0.3,
        };

        let chat_url = format!("{}/chat/completions", BASE_URL);
        let response = self
            .client
            .post(&chat_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| VrctError::Translation(format!("Plamo API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(VrctError::Translation(format!(
                "Plamo API returned error {}: {}",
                status, error_text
            )));
        }

        let plamo_response: PlamoResponse = response.json().await.map_err(|e| {
            VrctError::Translation(format!("Failed to parse Plamo response: {}", e))
        })?;

        plamo_response
            .choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .ok_or_else(|| {
                VrctError::Translation("No translation returned from Plamo".to_string())
            })
    }

    async fn validate_config(&self) -> Result<bool> {
        let has_key = self.api_key.read().await.is_some();
        let has_model = self.model.read().await.is_some();
        Ok(has_key && has_model)
    }

    fn name(&self) -> &str {
        "Plamo_API"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plamo_translator_creation() {
        let translator = PlamoTranslator::new();
        assert_eq!(translator.name(), "Plamo_API");
        assert!(!translator.validate_config().await.unwrap());
    }

    #[tokio::test]
    async fn test_plamo_translator_without_key() {
        let translator = PlamoTranslator::new();
        let result = translator.translate("Hello", "English", "Japanese").await;
        assert!(result.is_err());
    }
}
