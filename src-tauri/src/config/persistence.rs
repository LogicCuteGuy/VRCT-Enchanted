// Configuration persistence logic
// Handles loading and saving configuration to JSON files with debouncing

use crate::config::types::ConfigData;
use crate::utils::error::{Result, VrctError};
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use tracing::{info, warn, error};

/// Configuration persistence manager
/// Handles loading, saving, and debounced writes to the configuration file
pub struct ConfigPersistence {
    config_path: PathBuf,
    save_pending: Arc<Mutex<bool>>,
}

impl ConfigPersistence {
    /// Create a new ConfigPersistence instance
    /// 
    /// # Arguments
    /// * `config_path` - Path to the configuration JSON file
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            save_pending: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Get the default configuration file path
    /// Returns the path to config.json in the application's config directory
    pub fn default_config_path() -> Result<PathBuf> {
        // Get the application config directory
        let config_dir = dirs::config_dir()
            .ok_or_else(|| VrctError::Config("Failed to get config directory".to_string()))?;
        
        let app_config_dir = config_dir.join("VRCT");
        Ok(app_config_dir.join("config.json"))
    }
    
    /// Load configuration from the JSON file
    /// If the file doesn't exist, creates it with default values
    /// 
    /// # Returns
    /// * `Result<ConfigData>` - The loaded or default configuration
    pub async fn load(&self) -> Result<ConfigData> {
        info!("Loading configuration from: {:?}", self.config_path);
        
        // Check if config file exists
        if !self.config_path.exists() {
            warn!("Configuration file not found, creating with defaults");
            let default_config = ConfigData::default();
            self.save_immediate(&default_config).await?;
            return Ok(default_config);
        }
        
        // Read the file
        let contents = fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| VrctError::Config(format!("Failed to read config file: {}", e)))?;
        
        // Parse JSON
        let config: ConfigData = serde_json::from_str(&contents)
            .map_err(|e| VrctError::Config(format!("Failed to parse config JSON: {}", e)))?;
        
        info!("Configuration loaded successfully");
        Ok(config)
    }
    
    /// Save configuration immediately to the JSON file
    /// This bypasses the debouncing mechanism
    /// 
    /// # Arguments
    /// * `config` - The configuration data to save
    pub async fn save_immediate(&self, config: &ConfigData) -> Result<()> {
        info!("Saving configuration to: {:?}", self.config_path);
        
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| VrctError::Config(format!("Failed to create config directory: {}", e)))?;
        }
        
        // Serialize to JSON with pretty printing
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| VrctError::Config(format!("Failed to serialize config: {}", e)))?;
        
        // Write to file
        fs::write(&self.config_path, json)
            .await
            .map_err(|e| VrctError::Config(format!("Failed to write config file: {}", e)))?;
        
        info!("Configuration saved successfully");
        Ok(())
    }
    
    /// Schedule a debounced save operation
    /// The actual save will occur 2 seconds after the last call to this method
    /// 
    /// # Arguments
    /// * `config` - Arc reference to the configuration data to save
    pub async fn schedule_save(&self, config: Arc<ConfigData>) -> Result<()> {
        let mut pending = self.save_pending.lock().await;
        
        if *pending {
            // A save is already scheduled, just return
            return Ok(());
        }
        
        *pending = true;
        drop(pending); // Release the lock
        
        // Clone necessary data for the async task
        let config_path = self.config_path.clone();
        let save_pending = Arc::clone(&self.save_pending);
        
        // Spawn a task to handle the debounced save
        tokio::spawn(async move {
            // Wait for the debounce period
            sleep(Duration::from_secs(2)).await;
            
            // Perform the save
            let persistence = ConfigPersistence::new(config_path);
            if let Err(e) = persistence.save_immediate(&config).await {
                error!("Failed to save configuration: {}", e);
            }
            
            // Clear the pending flag
            let mut pending = save_pending.lock().await;
            *pending = false;
        });
        
        Ok(())
    }
    
    /// Load configuration with error handling
    /// If loading fails, returns default configuration and logs the error
    pub async fn load_or_default(&self) -> ConfigData {
        match self.load().await {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to load configuration: {}. Using defaults.", e);
                ConfigData::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let persistence = ConfigPersistence::new(config_path);
        
        let config = ConfigData::default();
        persistence.save_immediate(&config).await.unwrap();
        
        let loaded = persistence.load().await.unwrap();
        assert_eq!(loaded.transparency, config.transparency);
        assert_eq!(loaded.ui_language, config.ui_language);
    }
    
    #[tokio::test]
    async fn test_load_missing_file_creates_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let persistence = ConfigPersistence::new(config_path.clone());
        
        let loaded = persistence.load().await.unwrap();
        assert_eq!(loaded.transparency, 100);
        
        // Verify file was created
        assert!(config_path.exists());
    }
    
    #[tokio::test]
    async fn test_load_or_default_with_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        // Write invalid JSON
        fs::write(&config_path, "{ invalid json }").await.unwrap();
        
        let persistence = ConfigPersistence::new(config_path);
        let loaded = persistence.load_or_default().await;
        
        // Should return default config
        assert_eq!(loaded.transparency, 100);
    }
}
