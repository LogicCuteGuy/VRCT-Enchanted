// Integration tests for audio device management with configuration persistence
// These tests demonstrate the complete workflow of device selection and persistence

#[cfg(test)]
mod integration_tests {
    use crate::audio::DeviceManager;
    use crate::config::Config;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_device_selection_and_persistence_workflow() {
        // Create a temporary config file
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        // Initialize device manager and config
        let device_manager = DeviceManager::new();
        let config = Config::new(Some(config_path.clone())).await.unwrap();
        
        // Get available devices
        let input_devices = device_manager.enumerate_input_devices().unwrap();
        
        if let Some(first_device) = input_devices.first() {
            println!("Selecting device: {}", first_device.name);
            
            // Set the device in config
            config.set_mic_device(
                first_device.host_name.clone(),
                first_device.name.clone()
            ).await.unwrap();
            
            // Save immediately for testing
            config.save().await.unwrap();
            
            // Verify the device was saved
            let (saved_host, saved_device) = config.get_mic_device().await;
            assert_eq!(saved_host, first_device.host_name);
            assert_eq!(saved_device, first_device.name);
            
            // Verify auto_mic_select was disabled
            assert!(!config.is_auto_mic_select().await);
        }
        
        // Test loading config in a new instance
        let config2 = Config::new(Some(config_path)).await.unwrap();
        
        if let Some(first_device) = input_devices.first() {
            let (loaded_host, loaded_device) = config2.get_mic_device().await;
            assert_eq!(loaded_host, first_device.host_name);
            assert_eq!(loaded_device, first_device.name);
            assert!(!config2.is_auto_mic_select().await);
        }
    }

    #[tokio::test]
    async fn test_automatic_device_selection() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let device_manager = DeviceManager::new();
        let config = Config::new(Some(config_path)).await.unwrap();
        
        // Enable automatic selection
        config.set_auto_mic_select(true).await.unwrap();
        assert!(config.is_auto_mic_select().await);
        
        // Get default device
        if let Some(default_device) = device_manager.get_default_input().unwrap() {
            println!("Default device: {}", default_device.name);
            
            // When auto select is enabled, we should use the default device
            let selected = device_manager.select_input_device("").unwrap();
            assert_eq!(selected.name, default_device.name);
        }
    }

    #[tokio::test]
    async fn test_device_validation_on_config_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let device_manager = DeviceManager::new();
        let config = Config::new(Some(config_path)).await.unwrap();
        
        // Set an invalid device name
        config.set_mic_device("WASAPI".to_string(), "NonExistentDevice".to_string()).await.unwrap();
        
        // Get all available device names
        let available_devices = device_manager.get_all_device_names().unwrap();
        
        // Validate configuration
        let warnings = config.validate(Some(&available_devices)).await;
        
        // Should have a warning about the invalid device
        assert!(!warnings.is_empty());
        println!("Validation warnings: {:?}", warnings);
        
        // Auto select should be re-enabled
        assert!(config.is_auto_mic_select().await);
    }

    #[tokio::test]
    async fn test_speaker_device_selection() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let device_manager = DeviceManager::new();
        let config = Config::new(Some(config_path)).await.unwrap();
        
        // Get available output devices
        let output_devices = device_manager.enumerate_output_devices().unwrap();
        
        if let Some(first_device) = output_devices.first() {
            println!("Selecting speaker device: {}", first_device.name);
            
            // Set the speaker device
            config.set_speaker_device(first_device.name.clone()).await.unwrap();
            
            // Verify it was saved
            let saved_device = config.get_speaker_device().await;
            assert_eq!(saved_device, first_device.name);
            
            // Verify auto_speaker_select was disabled
            assert!(!config.is_auto_speaker_select().await);
        }
    }

    #[tokio::test]
    async fn test_device_fallback_on_not_found() {
        let device_manager = DeviceManager::new();
        
        // Try to select a non-existent device
        let result = device_manager.select_input_device("ThisDeviceDoesNotExist12345");
        
        // Should fall back to default device
        if let Ok(device) = result {
            println!("Fell back to device: {}", device.name);
            assert!(!device.name.is_empty());
            
            // Verify it's actually a valid device
            let all_devices = device_manager.enumerate_input_devices().unwrap();
            assert!(all_devices.iter().any(|d| d.name == device.name));
        }
    }

    #[tokio::test]
    async fn test_energy_level_reporting() {
        use crate::audio::{AudioRecorder, RecorderConfig};
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use std::time::Duration;
        
        let device_manager = DeviceManager::new();
        
        // Get default input device
        if let Ok(Some(device)) = device_manager.get_default_input() {
            println!("Testing energy reporting with device: {}", device.name);
            
            // Create recorder with energy reporting enabled
            let config = RecorderConfig {
                enable_energy_reporting: true,
                energy_report_interval_ms: 50, // Report every 50ms (20 times per second)
                ..Default::default()
            };
            
            let mut recorder = AudioRecorder::new(device, config).unwrap();
            
            // Set up a callback to track energy reports
            let report_count = Arc::new(AtomicU32::new(0));
            let last_energy = Arc::new(AtomicU32::new(0.0_f32.to_bits()));
            
            let report_count_clone = Arc::clone(&report_count);
            let last_energy_clone = Arc::clone(&last_energy);
            
            recorder.set_energy_callback(move |energy| {
                report_count_clone.fetch_add(1, Ordering::Relaxed);
                last_energy_clone.store(energy.to_bits(), Ordering::Relaxed);
                println!("Energy report: {:.6}", energy);
            });
            
            // Start recording
            if recorder.start().is_ok() {
                println!("Recording started, waiting for energy reports...");
                
                // Wait for at least 500ms to collect energy reports
                tokio::time::sleep(Duration::from_millis(500)).await;
                
                // Stop recording
                let _ = recorder.stop();
                
                // Verify we received energy reports
                let count = report_count.load(Ordering::Relaxed);
                println!("Received {} energy reports in 500ms", count);
                
                // With 50ms interval, we should get at least 8-10 reports in 500ms
                // (allowing for some timing variance)
                assert!(count >= 8, "Expected at least 8 energy reports, got {}", count);
                
                // Verify we can read the last reported energy
                let energy = f32::from_bits(last_energy.load(Ordering::Relaxed));
                println!("Last reported energy: {:.6}", energy);
                
                // Energy should be non-negative
                assert!(energy >= 0.0);
                
                // Verify get_current_energy also works
                let current = recorder.get_current_energy();
                println!("Current energy from getter: {:.6}", current);
                assert!(current >= 0.0);
            } else {
                println!("Could not start recording (device may be in use)");
            }
        } else {
            println!("No default input device available, skipping test");
        }
    }

    #[tokio::test]
    async fn test_energy_reporting_interval_configuration() {
        use crate::audio::{AudioRecorder, RecorderConfig};
        
        let device_manager = DeviceManager::new();
        
        if let Ok(Some(device)) = device_manager.get_default_input() {
            // Test with different intervals
            let intervals = vec![50, 100, 200];
            
            for interval_ms in intervals {
                println!("Testing with interval: {}ms", interval_ms);
                
                let config = RecorderConfig {
                    enable_energy_reporting: true,
                    energy_report_interval_ms: interval_ms,
                    ..Default::default()
                };
                
                let recorder = AudioRecorder::new(device.clone(), config).unwrap();
                
                // Verify configuration
                assert!(recorder.is_energy_reporting_enabled());
                assert_eq!(recorder.get_energy_report_interval(), interval_ms);
            }
        }
    }

    #[tokio::test]
    async fn test_energy_reporting_can_be_disabled() {
        use crate::audio::{AudioRecorder, RecorderConfig};
        
        let device_manager = DeviceManager::new();
        
        if let Ok(Some(device)) = device_manager.get_default_input() {
            // Create recorder with energy reporting disabled
            let config = RecorderConfig {
                enable_energy_reporting: false,
                ..Default::default()
            };
            
            let recorder = AudioRecorder::new(device, config).unwrap();
            
            // Verify reporting is disabled
            assert!(!recorder.is_energy_reporting_enabled());
            
            // get_current_energy should still work (it reads the atomic value)
            let energy = recorder.get_current_energy();
            assert!(energy >= 0.0);
        }
    }

    #[tokio::test]
    async fn test_energy_callback_can_be_cleared() {
        use crate::audio::{AudioRecorder, RecorderConfig};
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        
        let device_manager = DeviceManager::new();
        
        if let Ok(Some(device)) = device_manager.get_default_input() {
            let config = RecorderConfig::default();
            let mut recorder = AudioRecorder::new(device, config).unwrap();
            
            // Set a callback
            let called = Arc::new(AtomicBool::new(false));
            let called_clone = Arc::clone(&called);
            
            recorder.set_energy_callback(move |_| {
                called_clone.store(true, Ordering::Relaxed);
            });
            
            // Clear the callback
            recorder.clear_energy_callback();
            
            // The callback should not be called after clearing
            // (We can't easily test this without starting recording, but we verify the API works)
            println!("Energy callback cleared successfully");
        }
    }
}
