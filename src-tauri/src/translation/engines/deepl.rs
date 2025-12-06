// DeepL translation engine

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::sleep;

use crate::translation::Translator;
use crate::utils::error::{Result, VrctError};

#[derive(Debug, Deserialize)]
struct DeepLResponse {
    translations: Vec<Translation>,
}

#[derive(Debug, Deserialize)]
struct Translation {
    text: String,
}

/// Rate limiter for DeepL API calls
/// Implements a simple token bucket algorithm
struct RateLimiter {
    semaphore: Arc<Semaphore>,
    min_interval: Duration,
    last_request: Arc<RwLock<Option<tokio::time::Instant>>>,
}

impl RateLimiter {
    fn new(max_concurrent: usize, min_interval_ms: u64) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            min_interval: Duration::from_millis(min_interval_ms),
            last_request: Arc::new(RwLock::new(None)),
        }
    }

    async fn acquire(&self) -> Result<()> {
        // Acquire semaphore permit for concurrent request limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| VrctError::Translation(format!("Rate limiter error: {}", e)))?;

        // Enforce minimum interval between requests
        let mut last = self.last_request.write().await;
        if let Some(last_time) = *last {
            let elapsed = last_time.elapsed();
            if elapsed < self.min_interval {
                let wait_time = self.min_interval - elapsed;
                drop(last); // Release lock before sleeping
                sleep(wait_time).await;
                let mut last = self.last_request.write().await;
                *last = Some(tokio::time::Instant::now());
            } else {
                *last = Some(tokio::time::Instant::now());
            }
        } else {
            *last = Some(tokio::time::Instant::now());
        }

        // Permit is automatically released when dropped
        Ok(())
    }
}

/// DeepL API translator
pub struct DeepLTranslator {
    client: Client,
    api_key: Arc<RwLock<Option<String>>>,
    api_url: String,
    rate_limiter: RateLimiter,
}

impl DeepLTranslator {
    /// Create a new DeepL translator
    /// 
    /// Rate limiting configuration:
    /// - Max 5 concurrent requests
    /// - Minimum 200ms between requests (5 requests per second)
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: Arc::new(RwLock::new(None)),
            api_url: "https://api-free.deepl.com/v2/translate".to_string(),
            rate_limiter: RateLimiter::new(5, 200),
        }
    }

    /// Create a new DeepL translator with custom rate limiting
    pub fn with_rate_limit(max_concurrent: usize, min_interval_ms: u64) -> Self {
        Self {
            client: Client::new(),
            api_key: Arc::new(RwLock::new(None)),
            api_url: "https://api-free.deepl.com/v2/translate".to_string(),
            rate_limiter: RateLimiter::new(max_concurrent, min_interval_ms),
        }
    }

    /// Set the API key
    /// 
    /// Validates the API key by making a test request to the DeepL API.
    /// This validation request is subject to rate limiting.
    pub async fn set_api_key(&self, api_key: String) -> Result<()> {
        // Apply rate limiting before validation request
        self.rate_limiter.acquire().await?;

        // Validate the API key by making a test request
        let test_result = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
            .form(&[
                ("text", "test"),
                ("source_lang", "EN"),
                ("target_lang", "DE"),
            ])
            .send()
            .await;

        match test_result {
            Ok(response) => {
                if response.status().is_success() {
                    let mut key = self.api_key.write().await;
                    *key = Some(api_key);
                    Ok(())
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    Err(VrctError::Translation(format!(
                        "Invalid DeepL API key: {} - {}",
                        status, error_text
                    )))
                }
            }
            Err(e) => Err(VrctError::Translation(format!(
                "Failed to validate DeepL API key: {}",
                e
            ))),
        }
    }

    /// Get the current API key
    pub async fn get_api_key(&self) -> Option<String> {
        self.api_key.read().await.clone()
    }
}

impl Default for DeepLTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Translator for DeepLTranslator {
    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        // Apply rate limiting before making the request
        self.rate_limiter.acquire().await?;

        let api_key = self.api_key.read().await;
        let api_key = api_key.as_ref().ok_or_else(|| {
            VrctError::Translation("DeepL API key not set".to_string())
        })?;

        // Convert language codes to DeepL format (uppercase)
        let source_lang_upper = source_lang.to_uppercase();
        let target_lang_upper = target_lang.to_uppercase();

        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
            .form(&[
                ("text", text),
                ("source_lang", source_lang_upper.as_str()),
                ("target_lang", target_lang_upper.as_str()),
            ])
            .send()
            .await
            .map_err(|e| VrctError::Translation(format!("DeepL API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(VrctError::Translation(format!(
                "DeepL API returned error {}: {}",
                status, error_text
            )));
        }

        let deepl_response: DeepLResponse = response
            .json()
            .await
            .map_err(|e| VrctError::Translation(format!("Failed to parse DeepL response: {}", e)))?;

        deepl_response
            .translations
            .first()
            .map(|t| t.text.clone())
            .ok_or_else(|| VrctError::Translation("No translation returned from DeepL".to_string()))
    }

    async fn validate_config(&self) -> Result<bool> {
        Ok(self.api_key.read().await.is_some())
    }

    fn name(&self) -> &str {
        "DeepL_API"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deepl_translator_creation() {
        let translator = DeepLTranslator::new();
        assert_eq!(translator.name(), "DeepL_API");
        assert!(!translator.validate_config().await.unwrap());
    }

    #[tokio::test]
    async fn test_deepl_translator_with_custom_rate_limit() {
        let translator = DeepLTranslator::with_rate_limit(10, 100);
        assert_eq!(translator.name(), "DeepL_API");
        assert!(!translator.validate_config().await.unwrap());
    }

    #[tokio::test]
    async fn test_deepl_translator_without_key() {
        let translator = DeepLTranslator::new();
        let result = translator.translate("Hello", "EN", "DE").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key not set"));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        use tokio::time::Instant;

        let translator = DeepLTranslator::with_rate_limit(1, 500);
        
        // First request should be immediate
        let start = Instant::now();
        let _ = translator.rate_limiter.acquire().await;
        let first_elapsed = start.elapsed();
        
        // Second request should be delayed by at least 500ms
        let start = Instant::now();
        let _ = translator.rate_limiter.acquire().await;
        let second_elapsed = start.elapsed();
        
        // First request should be fast (< 100ms)
        assert!(first_elapsed.as_millis() < 100);
        // Second request should be delayed (>= 400ms, allowing some margin)
        assert!(second_elapsed.as_millis() >= 400);
    }

    #[tokio::test]
    async fn test_api_key_getter() {
        let translator = DeepLTranslator::new();
        assert!(translator.get_api_key().await.is_none());
    }
}
