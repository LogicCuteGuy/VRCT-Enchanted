// Configuration management module
// This module handles loading, saving, and managing application configuration

pub mod types;
pub mod persistence;

pub use types::*;
pub use persistence::*;

use crate::utils::error::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Thread-safe configuration manager singleton
/// Provides get/set methods with type safety and debounced persistence
pub struct Config {
    data: Arc<RwLock<ConfigData>>,
    persistence: Arc<ConfigPersistence>,
}

impl Config {
    /// Create a new Config instance
    /// 
    /// # Arguments
    /// * `config_path` - Optional path to the configuration file. If None, uses default path.
    pub async fn new(config_path: Option<std::path::PathBuf>) -> Result<Self> {
        let path = match config_path {
            Some(p) => p,
            None => ConfigPersistence::default_config_path()?,
        };
        
        let persistence = Arc::new(ConfigPersistence::new(path));
        let config_data = persistence.load_or_default().await;
        
        Ok(Self {
            data: Arc::new(RwLock::new(config_data)),
            persistence,
        })
    }
    
    /// Load configuration from disk
    pub async fn load(&self) -> Result<()> {
        info!("Loading configuration");
        let config_data = self.persistence.load().await?;
        let mut data = self.data.write().await;
        *data = config_data;
        Ok(())
    }
    
    /// Save configuration immediately to disk
    pub async fn save(&self) -> Result<()> {
        info!("Saving configuration immediately");
        let data = self.data.read().await;
        self.persistence.save_immediate(&data).await
    }
    
    /// Schedule a debounced save operation
    /// The actual save will occur 2 seconds after the last call to this method
    pub async fn schedule_save(&self) -> Result<()> {
        let data = self.data.read().await;
        let config_arc = Arc::new(data.clone());
        self.persistence.schedule_save(config_arc).await
    }
    
    /// Get a read-only reference to the configuration data
    /// Use this for reading multiple fields efficiently
    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, ConfigData> {
        self.data.read().await
    }
    
    /// Get a mutable reference to the configuration data
    /// Use this for modifying multiple fields efficiently
    /// Remember to call schedule_save() after modifications
    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, ConfigData> {
        self.data.write().await
    }
    
    /// Get a specific field value by providing a closure
    /// 
    /// # Example
    /// ```ignore
    /// let transparency = config.get(|data| data.transparency).await;
    /// ```
    pub async fn get<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&ConfigData) -> T,
    {
        let data = self.data.read().await;
        f(&data)
    }
    
    /// Set a specific field value by providing a closure
    /// Automatically schedules a debounced save
    /// 
    /// # Example
    /// ```ignore
    /// config.set(|data| data.transparency = 80).await;
    /// ```
    pub async fn set<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut ConfigData),
    {
        {
            let mut data = self.data.write().await;
            f(&mut data);
        }
        self.schedule_save().await
    }
    
    /// Validate the configuration and apply defaults for invalid values
    /// 
    /// # Arguments
    /// * `available_devices` - Optional list of available audio device names
    /// 
    /// # Returns
    /// * `Vec<String>` - List of validation warnings
    pub async fn validate(&self, available_devices: Option<&[String]>) -> Vec<String> {
        let mut data = self.data.write().await;
        let warnings = data.validate(available_devices);
        
        if !warnings.is_empty() {
            warn!("Configuration validation found {} issues", warnings.len());
            for warning in &warnings {
                warn!("  - {}", warning);
            }
            
            // Save the corrected configuration
            drop(data); // Release write lock before saving
            if let Err(e) = self.save().await {
                warn!("Failed to save corrected configuration: {}", e);
            }
        }
        
        warnings
    }
    
    /// Get a clone of the entire configuration data
    /// Use sparingly as this clones the entire structure
    pub async fn get_all(&self) -> ConfigData {
        let data = self.data.read().await;
        data.clone()
    }
    
    /// Replace the entire configuration data
    /// Automatically schedules a debounced save
    pub async fn set_all(&self, new_config: ConfigData) -> Result<()> {
        {
            let mut data = self.data.write().await;
            *data = new_config;
        }
        self.schedule_save().await
    }
}

// Convenience methods for common operations
impl Config {
    /// Get transparency value
    pub async fn get_transparency(&self) -> u8 {
        self.get(|data| data.transparency).await
    }
    
    /// Set transparency value
    pub async fn set_transparency(&self, value: u8) -> Result<()> {
        self.set(|data| data.transparency = value).await
    }
    
    /// Get UI language
    pub async fn get_ui_language(&self) -> String {
        self.get(|data| data.ui_language.clone()).await
    }
    
    /// Set UI language
    pub async fn set_ui_language(&self, language: String) -> Result<()> {
        self.set(|data| data.ui_language = language).await
    }
    
    /// Get selected microphone device
    pub async fn get_mic_device(&self) -> (String, String) {
        self.get(|data| (data.selected_mic_host.clone(), data.selected_mic_device.clone())).await
    }
    
    /// Set selected microphone device
    pub async fn set_mic_device(&self, host: String, device: String) -> Result<()> {
        self.set(|data| {
            data.selected_mic_host = host;
            data.selected_mic_device = device;
            data.auto_mic_select = false;
        }).await
    }
    
    /// Enable automatic microphone selection
    pub async fn set_auto_mic_select(&self, auto: bool) -> Result<()> {
        self.set(|data| data.auto_mic_select = auto).await
    }
    
    /// Get selected speaker device
    pub async fn get_speaker_device(&self) -> String {
        self.get(|data| data.selected_speaker_device.clone()).await
    }
    
    /// Set selected speaker device
    pub async fn set_speaker_device(&self, device: String) -> Result<()> {
        self.set(|data| {
            data.selected_speaker_device = device;
            data.auto_speaker_select = false;
        }).await
    }
    
    /// Enable automatic speaker selection
    pub async fn set_auto_speaker_select(&self, auto: bool) -> Result<()> {
        self.set(|data| data.auto_speaker_select = auto).await
    }
    
    /// Check if automatic microphone selection is enabled
    pub async fn is_auto_mic_select(&self) -> bool {
        self.get(|data| data.auto_mic_select).await
    }
    
    /// Check if automatic speaker selection is enabled
    pub async fn is_auto_speaker_select(&self) -> bool {
        self.get(|data| data.auto_speaker_select).await
    }
    
    /// Get OSC settings
    pub async fn get_osc_settings(&self) -> (String, u16) {
        self.get(|data| (data.osc_ip_address.clone(), data.osc_port)).await
    }
    
    /// Set OSC settings
    pub async fn set_osc_settings(&self, ip: String, port: u16) -> Result<()> {
        self.set(|data| {
            data.osc_ip_address = ip;
            data.osc_port = port;
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_config_new_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let config = Config::new(Some(config_path.clone())).await.unwrap();
        
        // Should have default values
        let transparency = config.get_transparency().await;
        assert_eq!(transparency, 100);
    }
    
    #[tokio::test]
    async fn test_config_get_set() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let config = Config::new(Some(config_path)).await.unwrap();
        
        // Set transparency
        config.set_transparency(80).await.unwrap();
        
        // Get transparency
        let transparency = config.get_transparency().await;
        assert_eq!(transparency, 80);
    }
    
    #[tokio::test]
    async fn test_config_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        // Create config and set value
        {
            let config = Config::new(Some(config_path.clone())).await.unwrap();
            config.set_transparency(75).await.unwrap();
            // Use immediate save instead of debounced save for testing
            config.save().await.unwrap();
        }
        
        // Load config again
        {
            let config = Config::new(Some(config_path)).await.unwrap();
            let transparency = config.get_transparency().await;
            assert_eq!(transparency, 75);
        }
    }
    
    #[tokio::test]
    async fn test_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let config = Config::new(Some(config_path)).await.unwrap();
        
        // Set invalid value
        config.set(|data| data.transparency = 150).await.unwrap();
        
        // Validate
        let warnings = config.validate(None).await;
        assert!(!warnings.is_empty());
        
        // Value should be corrected
        let transparency = config.get_transparency().await;
        assert_eq!(transparency, 100);
    }
    
    #[tokio::test]
    async fn test_config_convenience_methods() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let config = Config::new(Some(config_path)).await.unwrap();
        
        // Test UI language
        config.set_ui_language("ja".to_string()).await.unwrap();
        let language = config.get_ui_language().await;
        assert_eq!(language, "ja");
        
        // Test mic device
        config.set_mic_device("WASAPI".to_string(), "TestMic".to_string()).await.unwrap();
        let (host, device) = config.get_mic_device().await;
        assert_eq!(host, "WASAPI");
        assert_eq!(device, "TestMic");
        
        // Auto mic select should be disabled after setting device
        let auto_mic = config.is_auto_mic_select().await;
        assert!(!auto_mic);
        
        // Test speaker device
        config.set_speaker_device("TestSpeaker".to_string()).await.unwrap();
        let speaker = config.get_speaker_device().await;
        assert_eq!(speaker, "TestSpeaker");
        
        // Auto speaker select should be disabled after setting device
        let auto_speaker = config.is_auto_speaker_select().await;
        assert!(!auto_speaker);
        
        // Test enabling auto select
        config.set_auto_mic_select(true).await.unwrap();
        let auto_mic = config.is_auto_mic_select().await;
        assert!(auto_mic);
        
        config.set_auto_speaker_select(true).await.unwrap();
        let auto_speaker = config.is_auto_speaker_select().await;
        assert!(auto_speaker);
        
        // Test OSC settings
        config.set_osc_settings("192.168.1.1".to_string(), 9001).await.unwrap();
        let (ip, port) = config.get_osc_settings().await;
        assert_eq!(ip, "192.168.1.1");
        assert_eq!(port, 9001);
    }
}
