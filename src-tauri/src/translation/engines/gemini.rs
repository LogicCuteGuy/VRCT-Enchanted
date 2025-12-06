// Gemini translation engine

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
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: String,
}

/// Gemini API translator
pub struct GeminiTranslator {
    client: Client,
    api_key: Arc<RwLock<Option<String>>>,
    model: Arc<RwLock<String>>,
    supported_languages: Vec<String>,
}

impl GeminiTranslator {
    /// Create a new Gemini translator
    pub fn new() -> Self {
        let supported_languages = vec![
            "Arabic", "Bengali", "Bulgarian", "Simplified Chinese", "Traditional Chinese",
            "Croatian", "Czech", "Danish", "Dutch", "English", "Estonian", "Finnish", "French",
            "German", "Greek", "Hebrew", "Hindi", "Hungarian", "Indonesian", "Italian",
            "Japanese", "Korean", "Latvian", "Lithuanian", "Norwegian", "Polish", "Portuguese",
            "Romanian", "Russian", "Serbian", "Slovak", "Slovenian", "Spanish", "Swedish",
            "Thai", "Turkish", "Ukrainian", "Vietnamese",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            client: Client::new(),
            api_key: Arc::new(RwLock::new(None)),
            model: Arc::new(RwLock::new("gemini-pro".to_string())),
            supported_languages,
        }
    }

    /// Set the API key
    pub async fn set_api_key(&self, api_key: String) -> Result<()> {
        // Validate by making a simple request
        let model = self.model.read().await.clone();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, api_key
        );

        let test_request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: "test".to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                temperature: 0.3,
                max_output_tokens: 10,
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&test_request)
            .send()
            .await
            .map_err(|e| {
                VrctError::Translation(format!("Failed to validate Gemini API key: {}", e))
            })?;

        if response.status().is_success() {
            let mut key = self.api_key.write().await;
            *key = Some(api_key);
            Ok(())
        } else {
            Err(VrctError::Translation(format!(
                "Invalid Gemini API key: {}",
                response.status()
            )))
        }
    }

    /// Get the current API key
    pub async fn get_api_key(&self) -> Option<String> {
        self.api_key.read().await.clone()
    }

    /// Get available models
    pub fn get_model_list(&self) -> Vec<String> {
        vec![
            "gemini-pro".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ]
    }

    /// Set the model to use
    pub async fn set_model(&self, model: String) -> Result<()> {
        let models = self.get_model_list();
        if models.contains(&model) {
            let mut m = self.model.write().await;
            *m = model;
            Ok(())
        } else {
            Err(VrctError::Translation(format!(
                "Model '{}' not available",
                model
            )))
        }
    }

    /// Get the current model
    pub async fn get_model(&self) -> String {
        self.model.read().await.clone()
    }

    fn format_prompt(&self, text: &str, input_lang: &str, output_lang: &str) -> String {
        let system_prompt = SYSTEM_PROMPT_TEMPLATE
            .replace(
                "{supported_languages}",
                &self.supported_languages.join(", "),
            )
            .replace("{input_lang}", input_lang)
            .replace("{output_lang}", output_lang);

        format!("{}\n\nText to translate:\n{}", system_prompt, text)
    }
}

impl Default for GeminiTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Translator for GeminiTranslator {
    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let api_key = self.api_key.read().await;
        let api_key = api_key.as_ref().ok_or_else(|| {
            VrctError::Translation("Gemini API key not set".to_string())
        })?;

        let model = self.model.read().await.clone();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, api_key
        );

        let prompt = self.format_prompt(text, source_lang, target_lang);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
            generation_config: GenerationConfig {
                temperature: 0.3,
                max_output_tokens: 2048,
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| VrctError::Translation(format!("Gemini API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(VrctError::Translation(format!(
                "Gemini API returned error {}: {}",
                status, error_text
            )));
        }

        let gemini_response: GeminiResponse = response.json().await.map_err(|e| {
            VrctError::Translation(format!("Failed to parse Gemini response: {}", e))
        })?;

        gemini_response
            .candidates
            .first()
            .and_then(|candidate| candidate.content.parts.first())
            .map(|part| part.text.trim().to_string())
            .ok_or_else(|| {
                VrctError::Translation("No translation returned from Gemini".to_string())
            })
    }

    async fn validate_config(&self) -> Result<bool> {
        Ok(self.api_key.read().await.is_some())
    }

    fn name(&self) -> &str {
        "Gemini_API"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gemini_translator_creation() {
        let translator = GeminiTranslator::new();
        assert_eq!(translator.name(), "Gemini_API");
        assert!(!translator.validate_config().await.unwrap());
    }

    #[tokio::test]
    async fn test_gemini_translator_without_key() {
        let translator = GeminiTranslator::new();
        let result = translator
            .translate("Hello", "English", "Japanese")
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_model_list() {
        let translator = GeminiTranslator::new();
        let models = translator.get_model_list();
        assert!(!models.is_empty());
        assert!(models.contains(&"gemini-pro".to_string()));
    }
}
