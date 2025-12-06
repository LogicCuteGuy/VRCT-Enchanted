// OpenAI translation engine

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::translation::Translator;
use crate::utils::error::{Result, VrctError};

const SYSTEM_PROMPT_TEMPLATE: &str = r#"You are a helpful translation assistant.
Supported languages:
{supported_languages}

Translate the user provided text from {input_lang} to {output_lang}.
Return ONLY the translated text. Do not add quotes or extra commentary."#;

#[derive(Debug, Serialize)]
struct OpenAIRequest {
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
struct OpenAIResponse {
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

/// OpenAI API translator
pub struct OpenAITranslator {
    client: Client,
    api_key: Arc<RwLock<Option<String>>>,
    model: Arc<RwLock<Option<String>>>,
    base_url: String,
    supported_languages: Vec<String>,
}

impl OpenAITranslator {
    /// Create a new OpenAI translator
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
            client: Client::new(),
            api_key: Arc::new(RwLock::new(None)),
            model: Arc::new(RwLock::new(None)),
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            supported_languages,
        }
    }

    /// Set the API key
    pub async fn set_api_key(&self, api_key: String) -> Result<()> {
        // Validate the API key by listing models
        let models_url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&models_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| {
                VrctError::Translation(format!("Failed to validate OpenAI API key: {}", e))
            })?;

        if response.status().is_success() {
            let mut key = self.api_key.write().await;
            *key = Some(api_key);
            Ok(())
        } else {
            Err(VrctError::Translation(format!(
                "Invalid OpenAI API key: {}",
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
            VrctError::Translation("OpenAI API key not set".to_string())
        })?;

        let models_url = format!("{}/models", self.base_url);
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

        // Filter for GPT models suitable for translation
        let exclude_keywords = [
            "whisper", "embedding", "image", "tts", "audio", "search", "transcribe", "diarize",
            "vision",
        ];

        let mut allowed_models: Vec<String> = models_response
            .data
            .into_iter()
            .filter(|model| {
                let id = model.id.to_lowercase();
                // Include GPT models, exclude unwanted types
                (id.starts_with("gpt-") || (id.starts_with("ft:") && id.contains("gpt-")))
                    && !exclude_keywords.iter().any(|kw| id.contains(kw))
            })
            .map(|model| model.id)
            .collect();

        allowed_models.sort();
        Ok(allowed_models)
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

impl Default for OpenAITranslator {
    fn default() -> Self {
        Self::new(None)
    }
}

#[async_trait]
impl Translator for OpenAITranslator {
    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let api_key = self.api_key.read().await;
        let api_key = api_key.as_ref().ok_or_else(|| {
            VrctError::Translation("OpenAI API key not set".to_string())
        })?;

        let model = self.model.read().await;
        let model = model.as_ref().ok_or_else(|| {
            VrctError::Translation("OpenAI model not set".to_string())
        })?;

        let system_prompt = self.format_system_prompt(source_lang, target_lang);

        let request = OpenAIRequest {
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

        let chat_url = format!("{}/chat/completions", self.base_url);
        let response = self
            .client
            .post(&chat_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| VrctError::Translation(format!("OpenAI API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(VrctError::Translation(format!(
                "OpenAI API returned error {}: {}",
                status, error_text
            )));
        }

        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
            VrctError::Translation(format!("Failed to parse OpenAI response: {}", e))
        })?;

        openai_response
            .choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .ok_or_else(|| {
                VrctError::Translation("No translation returned from OpenAI".to_string())
            })
    }

    async fn validate_config(&self) -> Result<bool> {
        let has_key = self.api_key.read().await.is_some();
        let has_model = self.model.read().await.is_some();
        Ok(has_key && has_model)
    }

    fn name(&self) -> &str {
        "OpenAI_API"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_openai_translator_creation() {
        let translator = OpenAITranslator::new(None);
        assert_eq!(translator.name(), "OpenAI_API");
        assert!(!translator.validate_config().await.unwrap());
    }

    #[tokio::test]
    async fn test_openai_translator_without_key() {
        let translator = OpenAITranslator::new(None);
        let result = translator.translate("Hello", "English", "Japanese").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_system_prompt_formatting() {
        let translator = OpenAITranslator::new(None);
        let prompt = translator.format_system_prompt("English", "Japanese");
        assert!(prompt.contains("English"));
        assert!(prompt.contains("Japanese"));
        assert!(prompt.contains("Supported languages"));
    }
}
