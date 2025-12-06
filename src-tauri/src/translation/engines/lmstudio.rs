// LM Studio translation engine

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
struct LMStudioRequest {
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
struct LMStudioResponse {
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

/// LM Studio translator (OpenAI-compatible local server)
pub struct LMStudioTranslator {
    client: Client,
    base_url: Arc<RwLock<Option<String>>>,
    model: Arc<RwLock<Option<String>>>,
    supported_languages: Vec<String>,
}

impl LMStudioTranslator {
    /// Create a new LM Studio translator
    pub fn new(base_url: Option<String>) -> Self {
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
            base_url: Arc::new(RwLock::new(base_url)),
            model: Arc::new(RwLock::new(None)),
            supported_languages,
        }
    }

    /// Set the base URL and validate connection
    pub async fn set_base_url(&self, base_url: String) -> Result<()> {
        // Validate by checking if we can list models
        let models_url = format!("{}/models", base_url);
        let response = self
            .client
            .get(&models_url)
            .send()
            .await
            .map_err(|e| {
                VrctError::Translation(format!("Failed to connect to LM Studio: {}", e))
            })?;

        if response.status().is_success() {
            let mut url = self.base_url.write().await;
            *url = Some(base_url);
            Ok(())
        } else {
            Err(VrctError::Translation(format!(
                "LM Studio connection failed: {}",
                response.status()
            )))
        }
    }

    /// Get the current base URL
    pub async fn get_base_url(&self) -> Option<String> {
        self.base_url.read().await.clone()
    }

    /// Get available models
    pub async fn get_model_list(&self) -> Result<Vec<String>> {
        let base_url = self.base_url.read().await;
        let base_url = base_url.as_ref().ok_or_else(|| {
            VrctError::Translation("LM Studio base URL not set".to_string())
        })?;

        let models_url = format!("{}/models", base_url);
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

impl Default for LMStudioTranslator {
    fn default() -> Self {
        Self::new(Some("http://127.0.0.1:1234/v1".to_string()))
    }
}

#[async_trait]
impl Translator for LMStudioTranslator {
    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let base_url = self.base_url.read().await;
        let base_url = base_url.as_ref().ok_or_else(|| {
            VrctError::Translation("LM Studio base URL not set".to_string())
        })?;

        let model = self.model.read().await;
        let model = model.as_ref().ok_or_else(|| {
            VrctError::Translation("LM Studio model not set".to_string())
        })?;

        let system_prompt = self.format_system_prompt(source_lang, target_lang);

        let request = LMStudioRequest {
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

        let chat_url = format!("{}/chat/completions", base_url);
        let response = self
            .client
            .post(&chat_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| VrctError::Translation(format!("LM Studio API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(VrctError::Translation(format!(
                "LM Studio API returned error {}: {}",
                status, error_text
            )));
        }

        let lmstudio_response: LMStudioResponse = response.json().await.map_err(|e| {
            VrctError::Translation(format!("Failed to parse LM Studio response: {}", e))
        })?;

        lmstudio_response
            .choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .ok_or_else(|| {
                VrctError::Translation("No translation returned from LM Studio".to_string())
            })
    }

    async fn validate_config(&self) -> Result<bool> {
        let has_url = self.base_url.read().await.is_some();
        let has_model = self.model.read().await.is_some();
        Ok(has_url && has_model)
    }

    fn name(&self) -> &str {
        "LMStudio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lmstudio_translator_creation() {
        let translator = LMStudioTranslator::new(None);
        assert_eq!(translator.name(), "LMStudio");
        assert!(!translator.validate_config().await.unwrap());
    }

    #[tokio::test]
    async fn test_lmstudio_translator_without_url() {
        let translator = LMStudioTranslator::new(None);
        let result = translator.translate("Hello", "English", "Japanese").await;
        assert!(result.is_err());
    }
}
