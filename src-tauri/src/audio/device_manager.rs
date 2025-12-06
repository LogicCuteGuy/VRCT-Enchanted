// Audio device management
// Handles enumeration and management of audio input/output devices using cpal

use crate::utils::error::{Result, VrctError};
use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Represents an audio device (input or output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Human-readable device name
    pub name: String,
    /// Audio host name (e.g., "WASAPI", "MME", "ALSA")
    pub host_name: String,
    /// Device index in the enumeration
    pub index: usize,
    /// Whether this is a loopback device (for speaker capture)
    pub is_loopback: bool,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of audio channels
    pub channels: u16,
}

/// Manages audio device enumeration and selection
pub struct DeviceManager {
    host: cpal::Host,
}

impl DeviceManager {
    /// Create a new DeviceManager with the default audio host
    pub fn new() -> Self {
        let host = cpal::default_host();
        info!("Initialized audio device manager with host: {:?}", host.id());
        Self { host }
    }

    /// Enumerate all available input devices (microphones)
    pub fn enumerate_input_devices(&self) -> Result<Vec<AudioDevice>> {
        debug!("Enumerating input devices");
        
        let devices = self.host.input_devices()
            .map_err(|e| VrctError::AudioDevice(format!("Failed to enumerate input devices: {}", e)))?;

        let mut audio_devices = Vec::new();
        
        for (index, device) in devices.enumerate() {
            match self.create_audio_device(device, index, false) {
                Ok(audio_device) => {
                    debug!("Found input device: {} ({})", audio_device.name, audio_device.host_name);
                    audio_devices.push(audio_device);
                }
                Err(e) => {
                    warn!("Failed to get info for input device {}: {}", index, e);
                }
            }
        }

        info!("Found {} input devices", audio_devices.len());
        Ok(audio_devices)
    }

    /// Enumerate all available output devices (speakers/loopback)
    /// Note: On Windows, loopback devices may require special handling
    pub fn enumerate_output_devices(&self) -> Result<Vec<AudioDevice>> {
        debug!("Enumerating output devices");
        
        let devices = self.host.output_devices()
            .map_err(|e| VrctError::AudioDevice(format!("Failed to enumerate output devices: {}", e)))?;

        let mut audio_devices = Vec::new();
        
        for (index, device) in devices.enumerate() {
            match self.create_audio_device(device, index, false) {
                Ok(audio_device) => {
                    debug!("Found output device: {} ({})", audio_device.name, audio_device.host_name);
                    audio_devices.push(audio_device);
                }
                Err(e) => {
                    warn!("Failed to get info for output device {}: {}", index, e);
                }
            }
        }

        // On Windows, try to enumerate loopback devices
        // Note: cpal doesn't directly support loopback enumeration on all platforms
        // This is a placeholder for future loopback support
        #[cfg(target_os = "windows")]
        {
            debug!("Attempting to enumerate loopback devices on Windows");
            // TODO: Implement Windows-specific loopback device enumeration
            // This may require using wasapi directly or a different approach
        }

        info!("Found {} output devices", audio_devices.len());
        Ok(audio_devices)
    }

    /// Get the default input device
    pub fn get_default_input(&self) -> Result<Option<AudioDevice>> {
        debug!("Getting default input device");
        
        match self.host.default_input_device() {
            Some(device) => {
                let audio_device = self.create_audio_device(device, 0, false)?;
                info!("Default input device: {}", audio_device.name);
                Ok(Some(audio_device))
            }
            None => {
                warn!("No default input device found");
                Ok(None)
            }
        }
    }

    /// Get the default output device
    pub fn get_default_output(&self) -> Result<Option<AudioDevice>> {
        debug!("Getting default output device");
        
        match self.host.default_output_device() {
            Some(device) => {
                let audio_device = self.create_audio_device(device, 0, false)?;
                info!("Default output device: {}", audio_device.name);
                Ok(Some(audio_device))
            }
            None => {
                warn!("No default output device found");
                Ok(None)
            }
        }
    }

    /// Get a device by its name
    /// Searches through both input and output devices
    pub fn get_device_by_name(&self, name: &str) -> Result<Option<AudioDevice>> {
        debug!("Looking for device by name: {}", name);
        
        // Search input devices
        let input_devices = self.enumerate_input_devices()?;
        if let Some(device) = input_devices.iter().find(|d| d.name == name) {
            debug!("Found device in input devices: {}", name);
            return Ok(Some(device.clone()));
        }

        // Search output devices
        let output_devices = self.enumerate_output_devices()?;
        if let Some(device) = output_devices.iter().find(|d| d.name == name) {
            debug!("Found device in output devices: {}", name);
            return Ok(Some(device.clone()));
        }

        warn!("Device not found: {}", name);
        Ok(None)
    }

    /// Helper function to create an AudioDevice from a cpal::Device
    fn create_audio_device(
        &self,
        device: cpal::Device,
        index: usize,
        is_loopback: bool,
    ) -> Result<AudioDevice> {
        let name = device
            .name()
            .map_err(|e| VrctError::AudioDevice(format!("Failed to get device name: {}", e)))?;

        // Get the default config to extract sample rate and channels
        let default_config = device
            .default_input_config()
            .or_else(|_| device.default_output_config())
            .map_err(|e| {
                VrctError::AudioDevice(format!("Failed to get device config for '{}': {}", name, e))
            })?;

        let sample_rate = default_config.sample_rate().0;
        let channels = default_config.channels();

        Ok(AudioDevice {
            name,
            host_name: self.host.id().name().to_string(),
            index,
            is_loopback,
            sample_rate,
            channels,
        })
    }

    /// Get the cpal Host for direct access if needed
    pub fn host(&self) -> &cpal::Host {
        &self.host
    }

    /// Get all available device names (both input and output)
    /// Useful for configuration validation
    pub fn get_all_device_names(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        
        // Add input device names
        let input_devices = self.enumerate_input_devices()?;
        names.extend(input_devices.iter().map(|d| d.name.clone()));
        
        // Add output device names
        let output_devices = self.enumerate_output_devices()?;
        names.extend(output_devices.iter().map(|d| d.name.clone()));
        
        Ok(names)
    }

    /// Select a device by name, or use default if name is empty or not found
    /// Returns the selected device
    pub fn select_input_device(&self, device_name: &str) -> Result<AudioDevice> {
        if device_name.is_empty() {
            // Use default device
            if let Some(device) = self.get_default_input()? {
                return Ok(device);
            } else {
                return Err(VrctError::AudioDevice(
                    "No default input device available".to_string()
                ));
            }
        }

        // Try to find device by name
        if let Some(device) = self.get_device_by_name(device_name)? {
            return Ok(device);
        }

        // Device not found, fall back to default
        warn!("Device '{}' not found, using default input device", device_name);
        if let Some(device) = self.get_default_input()? {
            Ok(device)
        } else {
            Err(VrctError::AudioDevice(
                format!("Device '{}' not found and no default input device available", device_name)
            ))
        }
    }

    /// Select an output device by name, or use default if name is empty or not found
    /// Returns the selected device
    pub fn select_output_device(&self, device_name: &str) -> Result<AudioDevice> {
        if device_name.is_empty() {
            // Use default device
            if let Some(device) = self.get_default_output()? {
                return Ok(device);
            } else {
                return Err(VrctError::AudioDevice(
                    "No default output device available".to_string()
                ));
            }
        }

        // Try to find device by name
        if let Some(device) = self.get_device_by_name(device_name)? {
            return Ok(device);
        }

        // Device not found, fall back to default
        warn!("Device '{}' not found, using default output device", device_name);
        if let Some(device) = self.get_default_output()? {
            Ok(device)
        } else {
            Err(VrctError::AudioDevice(
                format!("Device '{}' not found and no default output device available", device_name)
            ))
        }
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_manager_creation() {
        let manager = DeviceManager::new();
        assert_eq!(manager.host.id().name(), cpal::default_host().id().name());
    }

    #[test]
    fn test_enumerate_input_devices() {
        let manager = DeviceManager::new();
        let result = manager.enumerate_input_devices();
        
        // Should not error even if no devices found
        assert!(result.is_ok());
        
        let devices = result.unwrap();
        println!("Found {} input devices", devices.len());
        
        for device in &devices {
            println!("  - {} ({})", device.name, device.host_name);
            assert!(!device.name.is_empty());
            assert!(!device.host_name.is_empty());
            assert!(device.sample_rate > 0);
            assert!(device.channels > 0);
        }
    }

    #[test]
    fn test_enumerate_output_devices() {
        let manager = DeviceManager::new();
        let result = manager.enumerate_output_devices();
        
        // Should not error even if no devices found
        assert!(result.is_ok());
        
        let devices = result.unwrap();
        println!("Found {} output devices", devices.len());
        
        for device in &devices {
            println!("  - {} ({})", device.name, device.host_name);
            assert!(!device.name.is_empty());
            assert!(!device.host_name.is_empty());
            assert!(device.sample_rate > 0);
            assert!(device.channels > 0);
        }
    }

    #[test]
    fn test_get_default_input() {
        let manager = DeviceManager::new();
        let result = manager.get_default_input();
        
        assert!(result.is_ok());
        
        if let Some(device) = result.unwrap() {
            println!("Default input device: {}", device.name);
            assert!(!device.name.is_empty());
        } else {
            println!("No default input device found");
        }
    }

    #[test]
    fn test_get_default_output() {
        let manager = DeviceManager::new();
        let result = manager.get_default_output();
        
        assert!(result.is_ok());
        
        if let Some(device) = result.unwrap() {
            println!("Default output device: {}", device.name);
            assert!(!device.name.is_empty());
        } else {
            println!("No default output device found");
        }
    }

    #[test]
    fn test_get_device_by_name() {
        let manager = DeviceManager::new();
        
        // Get all devices first
        let input_devices = manager.enumerate_input_devices().unwrap();
        
        if let Some(first_device) = input_devices.first() {
            let result = manager.get_device_by_name(&first_device.name);
            assert!(result.is_ok());
            
            let found_device = result.unwrap();
            assert!(found_device.is_some());
            
            let found = found_device.unwrap();
            assert_eq!(found.name, first_device.name);
        }
        
        // Test with non-existent device
        let result = manager.get_device_by_name("NonExistentDevice12345");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_get_all_device_names() {
        let manager = DeviceManager::new();
        let result = manager.get_all_device_names();
        
        assert!(result.is_ok());
        let names = result.unwrap();
        
        println!("Found {} total devices", names.len());
        assert!(!names.is_empty(), "Should find at least some devices");
        
        for name in &names {
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_select_input_device_with_empty_name() {
        let manager = DeviceManager::new();
        
        // Empty name should return default device
        let result = manager.select_input_device("");
        
        // Should succeed if there's a default device
        if let Ok(device) = result {
            println!("Selected default input device: {}", device.name);
            assert!(!device.name.is_empty());
        }
    }

    #[test]
    fn test_select_input_device_with_valid_name() {
        let manager = DeviceManager::new();
        
        // Get first available device
        let input_devices = manager.enumerate_input_devices().unwrap();
        
        if let Some(first_device) = input_devices.first() {
            let result = manager.select_input_device(&first_device.name);
            assert!(result.is_ok());
            
            let selected = result.unwrap();
            assert_eq!(selected.name, first_device.name);
        }
    }

    #[test]
    fn test_select_input_device_with_invalid_name() {
        let manager = DeviceManager::new();
        
        // Invalid name should fall back to default
        let result = manager.select_input_device("NonExistentDevice12345");
        
        // Should succeed with default device if available
        if let Ok(device) = result {
            println!("Fell back to default device: {}", device.name);
            assert!(!device.name.is_empty());
        }
    }

    #[test]
    fn test_select_output_device_with_empty_name() {
        let manager = DeviceManager::new();
        
        // Empty name should return default device
        let result = manager.select_output_device("");
        
        // Should succeed if there's a default device
        if let Ok(device) = result {
            println!("Selected default output device: {}", device.name);
            assert!(!device.name.is_empty());
        }
    }

    #[test]
    fn test_select_output_device_with_valid_name() {
        let manager = DeviceManager::new();
        
        // Get first available device
        let output_devices = manager.enumerate_output_devices().unwrap();
        
        if let Some(first_device) = output_devices.first() {
            let result = manager.select_output_device(&first_device.name);
            assert!(result.is_ok());
            
            let selected = result.unwrap();
            assert_eq!(selected.name, first_device.name);
        }
    }
}
