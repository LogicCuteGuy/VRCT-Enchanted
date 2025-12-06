// Translation subsystem module
// This module handles text translation using multiple engines

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

pub mod ctranslate2;
pub mod engines;
pub mod languages;

#[cfg(test)]
mod fallback_tests;

use crate::utils::error::{Result, VrctError};

pub use ctranslate2::CTranslate2Translator;

/// Translation error notification that can be sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationErrorNotification {
    pub failed_engine: String,
    pub error_message: String,
    pub fallback_engine: String,
    pub fallback_succeeded: bool,
}

/// Trait for translation engines
/// All translation engines must implement this trait
#[async_trait]
pub trait Translator: Send + Sync {
    /// Translate text from source language to target language
    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String>;

    /// Validate the translator configuration (API keys, connection, etc.)
    async fn validate_config(&self) -> Result<bool>;

    /// Get the name of this translator
    fn name(&self) -> &str;
}

/// Callback type for translation error notifications
pub type ErrorNotificationCallback = Arc<dyn Fn(TranslationErrorNotification) + Send + Sync>;

/// Translation manager that handles multiple translation engines
pub struct TranslationManager {
    engines: HashMap<String, Arc<dyn Translator>>,
    active_engine: Arc<RwLock<Option<String>>>,
    fallback_engine: Arc<RwLock<Option<String>>>,
    error_callback: Arc<RwLock<Option<ErrorNotificationCallback>>>,
}

impl TranslationManager {
    /// Create a new translation manager
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
            active_engine: Arc::new(RwLock::new(None)),
            fallback_engine: Arc::new(RwLock::new(None)),
            error_callback: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the error notification callback
    /// This callback will be invoked when translation fails and fallback is attempted
    pub async fn set_error_callback(&self, callback: ErrorNotificationCallback) {
        let mut cb = self.error_callback.write().await;
        *cb = Some(callback);
    }

    /// Register a translation engine
    pub fn register_engine(&mut self, engine: Arc<dyn Translator>) {
        let name = engine.name().to_string();
        self.engines.insert(name, engine);
    }

    /// Set the active translation engine
    pub async fn set_active_engine(&self, name: &str) -> Result<()> {
        if !self.engines.contains_key(name) {
            return Err(VrctError::Translation(format!(
                "Translation engine '{}' not found",
                name
            )));
        }
        
        let mut active = self.active_engine.write().await;
        *active = Some(name.to_string());
        Ok(())
    }

    /// Set the fallback translation engine (typically CTranslate2)
    pub async fn set_fallback_engine(&self, name: &str) -> Result<()> {
        if !self.engines.contains_key(name) {
            return Err(VrctError::Translation(format!(
                "Translation engine '{}' not found",
                name
            )));
        }
        
        let mut fallback = self.fallback_engine.write().await;
        *fallback = Some(name.to_string());
        Ok(())
    }

    /// Get the active engine name
    pub async fn get_active_engine(&self) -> Option<String> {
        self.active_engine.read().await.clone()
    }

    /// Get the fallback engine name
    pub async fn get_fallback_engine(&self) -> Option<String> {
        self.fallback_engine.read().await.clone()
    }

    /// Translate text using the active engine, with fallback on failure
    /// 
    /// When a non-CTranslate2 engine fails, this method will:
    /// 1. Log the failure
    /// 2. Attempt to use the fallback engine (typically CTranslate2)
    /// 3. Send an error notification to the frontend via the callback
    /// 
    /// Requirements: 4.4
    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        use crate::utils::profiling::global_profiler;
        let _guard = global_profiler().start_timing("translation_total");
        
        // If source and target are the same, return original text
        if source_lang == target_lang {
            return Ok(text.to_string());
        }

        let active_name = self.active_engine.read().await.clone();
        
        if let Some(engine_name) = active_name {
            if let Some(engine) = self.engines.get(&engine_name) {
                match engine.translate(text, source_lang, target_lang).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        let error_message = e.user_message();
                        tracing::warn!(
                            "Translation failed with engine '{}': {}. Attempting fallback.",
                            engine_name,
                            error_message
                        );
                        
                        // Try fallback engine if available and different from active engine
                        let fallback_name = self.fallback_engine.read().await.clone();
                        if let Some(fallback_engine_name) = fallback_name {
                            // Only attempt fallback if it's a different engine
                            if fallback_engine_name != engine_name {
                                if let Some(fallback_engine) = self.engines.get(&fallback_engine_name) {
                                    tracing::info!(
                                        "Attempting fallback translation with engine '{}'",
                                        fallback_engine_name
                                    );
                                    
                                    match fallback_engine.translate(text, source_lang, target_lang).await {
                                        Ok(result) => {
                                            // Fallback succeeded - notify frontend
                                            let notification = TranslationErrorNotification {
                                                failed_engine: engine_name.clone(),
                                                error_message: error_message.clone(),
                                                fallback_engine: fallback_engine_name.clone(),
                                                fallback_succeeded: true,
                                            };
                                            
                                            if let Some(callback) = self.error_callback.read().await.as_ref() {
                                                callback(notification);
                                            }
                                            
                                            tracing::info!(
                                                "Fallback translation succeeded with engine '{}'",
                                                fallback_engine_name
                                            );
                                            
                                            return Ok(result);
                                        }
                                        Err(fallback_error) => {
                                            // Fallback also failed - notify frontend
                                            let notification = TranslationErrorNotification {
                                                failed_engine: engine_name.clone(),
                                                error_message: error_message.clone(),
                                                fallback_engine: fallback_engine_name.clone(),
                                                fallback_succeeded: false,
                                            };
                                            
                                            if let Some(callback) = self.error_callback.read().await.as_ref() {
                                                callback(notification);
                                            }
                                            
                                            tracing::error!(
                                                "Fallback translation also failed with engine '{}': {}",
                                                fallback_engine_name,
                                                fallback_error.user_message()
                                            );
                                            
                                            return Err(VrctError::Translation(format!(
                                                "Translation failed with both {} and fallback {}: {}",
                                                engine_name, fallback_engine_name, error_message
                                            )));
                                        }
                                    }
                                }
                            } else {
                                tracing::warn!(
                                    "Fallback engine '{}' is the same as active engine, skipping fallback",
                                    fallback_engine_name
                                );
                            }
                        } else {
                            tracing::warn!("No fallback engine configured");
                        }
                        
                        // No fallback available or fallback is same as active - return original error
                        return Err(e);
                    }
                }
            }
        }

        Err(VrctError::Translation(
            "No active translation engine configured".to_string(),
        ))
    }

    /// List all registered engine names
    pub fn list_engines(&self) -> Vec<String> {
        self.engines.keys().cloned().collect()
    }

    /// Get a specific engine by name
    pub fn get_engine(&self, name: &str) -> Option<Arc<dyn Translator>> {
        self.engines.get(name).cloned()
    }
}

impl Default for TranslationManager {
    fn default() -> Self {
        Self::new()
    }
}
