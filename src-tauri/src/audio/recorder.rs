// Audio recording functionality
// Captures audio streams with lock-free ring buffer

use crate::audio::device_manager::AudioDevice;
use crate::utils::error::{Result, VrctError};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use ringbuf::traits::{Consumer, Observer, RingBuffer};
use ringbuf::HeapRb;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Configuration for audio recording
#[derive(Debug, Clone)]
pub struct RecorderConfig {
    /// Energy threshold for voice activity detection (0.0 - 1.0)
    pub energy_threshold: f32,
    /// Whether voice activity detection is enabled
    pub vad_enabled: bool,
    /// Buffer size in samples (default: 48000 = 1 second at 48kHz)
    pub buffer_size: usize,
    /// Whether to enable energy level reporting
    pub enable_energy_reporting: bool,
    /// Minimum interval between energy reports (in milliseconds)
    pub energy_report_interval_ms: u64,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            energy_threshold: 0.01,
            vad_enabled: true,
            buffer_size: 48000,
            enable_energy_reporting: true,
            energy_report_interval_ms: 100, // 10 times per second
        }
    }
}

/// Callback type for energy level updates
pub type EnergyCallback = Arc<dyn Fn(f32) + Send + Sync>;

/// Audio recorder that captures audio from a device
pub struct AudioRecorder {
    /// The audio device being recorded from
    device_info: AudioDevice,
    /// The cpal device handle
    device: Device,
    /// The active audio stream (None when stopped)
    stream: Option<Stream>,
    /// Lock-free ring buffer for audio samples
    buffer: Arc<Mutex<HeapRb<f32>>>,
    /// Current energy threshold (stored as u32 bits for atomic access)
    energy_threshold: Arc<AtomicU32>,
    /// Whether VAD is enabled
    vad_enabled: Arc<AtomicBool>,
    /// Whether the recorder is currently recording
    is_recording: Arc<AtomicBool>,
    /// Current energy level (stored as u32 bits for atomic access)
    current_energy: Arc<AtomicU32>,
    /// Configuration
    config: RecorderConfig,
    /// Optional callback for energy level updates
    energy_callback: Option<EnergyCallback>,
    /// Last time energy was reported
    last_energy_report: Arc<Mutex<Instant>>,
}

impl AudioRecorder {
    /// Create a new AudioRecorder for the specified device
    pub fn new(device_info: AudioDevice, config: RecorderConfig) -> Result<Self> {
        info!("Creating audio recorder for device: {}", device_info.name);
        
        // Get the cpal host and device
        let host = cpal::default_host();
        let device = Self::find_device(&host, &device_info)?;
        
        // Create ring buffer with configured size
        let buffer = Arc::new(Mutex::new(HeapRb::<f32>::new(config.buffer_size)));
        
        Ok(Self {
            device_info,
            device,
            stream: None,
            buffer,
            energy_threshold: Arc::new(AtomicU32::new(config.energy_threshold.to_bits())),
            vad_enabled: Arc::new(AtomicBool::new(config.vad_enabled)),
            is_recording: Arc::new(AtomicBool::new(false)),
            current_energy: Arc::new(AtomicU32::new(0.0_f32.to_bits())),
            config,
            energy_callback: None,
            last_energy_report: Arc::new(Mutex::new(Instant::now())),
        })
    }

    /// Find a cpal device matching the AudioDevice info
    fn find_device(host: &cpal::Host, device_info: &AudioDevice) -> Result<Device> {
        // Try input devices first
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    if name == device_info.name {
                        debug!("Found matching input device: {}", name);
                        return Ok(device);
                    }
                }
            }
        }
        
        // Try output devices (for loopback)
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    if name == device_info.name {
                        debug!("Found matching output device: {}", name);
                        return Ok(device);
                    }
                }
            }
        }
        
        Err(VrctError::AudioDevice(format!(
            "Device '{}' not found",
            device_info.name
        )))
    }

    /// Start recording audio
    pub fn start(&mut self) -> Result<()> {
        if self.is_recording.load(Ordering::Relaxed) {
            warn!("Recorder is already running");
            return Ok(());
        }

        info!("Starting audio recording from: {}", self.device_info.name);

        // Get the default input config
        let config = self
            .device
            .default_input_config()
            .map_err(|e| VrctError::AudioDevice(format!("Failed to get input config: {}", e)))?;

        debug!(
            "Audio config: {} Hz, {} channels, {:?}",
            config.sample_rate().0,
            config.channels(),
            config.sample_format()
        );

        // Build the stream based on sample format
        let stream = match config.sample_format() {
            SampleFormat::F32 => self.build_stream::<f32>(&config.into())?,
            SampleFormat::I16 => self.build_stream::<i16>(&config.into())?,
            SampleFormat::U16 => self.build_stream::<u16>(&config.into())?,
            _ => {
                return Err(VrctError::AudioDevice(format!(
                    "Unsupported sample format: {:?}",
                    config.sample_format()
                )))
            }
        };

        // Start the stream
        stream
            .play()
            .map_err(|e| VrctError::AudioDevice(format!("Failed to start stream: {}", e)))?;

        self.stream = Some(stream);
        self.is_recording.store(true, Ordering::Relaxed);

        info!("Audio recording started successfully");
        Ok(())
    }

    /// Stop recording audio
    pub fn stop(&mut self) -> Result<()> {
        if !self.is_recording.load(Ordering::Relaxed) {
            warn!("Recorder is not running");
            return Ok(());
        }

        info!("Stopping audio recording");

        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        self.is_recording.store(false, Ordering::Relaxed);

        // Clear the buffer
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.clear();
        }

        info!("Audio recording stopped");
        Ok(())
    }

    /// Build an audio stream for a specific sample type
    fn build_stream<T>(&self, config: &StreamConfig) -> Result<Stream>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let buffer = Arc::clone(&self.buffer);
        let current_energy = Arc::clone(&self.current_energy);
        let channels = config.channels as usize;
        let energy_callback = self.energy_callback.clone();
        let enable_reporting = self.config.enable_energy_reporting;
        let report_interval = Duration::from_millis(self.config.energy_report_interval_ms);
        let last_report = Arc::clone(&self.last_energy_report);

        let stream = self
            .device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    // Convert samples to f32 and calculate energy
                    let mut sum_squares = 0.0_f32;
                    let mut sample_count = 0;

                    // Process audio data
                    let mut samples = Vec::with_capacity(data.len() / channels);
                    
                    for frame in data.chunks(channels) {
                        // Average all channels to mono
                        let mut frame_sum = 0.0_f32;
                        for &sample in frame {
                            let sample_f32: f32 = cpal::Sample::from_sample(sample);
                            frame_sum += sample_f32;
                            sum_squares += sample_f32 * sample_f32;
                            sample_count += 1;
                        }
                        let mono_sample = frame_sum / channels as f32;
                        samples.push(mono_sample);
                    }

                    // Calculate RMS energy
                    let energy = if sample_count > 0 {
                        (sum_squares / sample_count as f32).sqrt()
                    } else {
                        0.0
                    };

                    // Update current energy
                    current_energy.store(energy.to_bits(), Ordering::Relaxed);

                    // Report energy if enabled and interval has passed
                    if enable_reporting {
                        if let Ok(mut last) = last_report.lock() {
                            let now = Instant::now();
                            if now.duration_since(*last) >= report_interval {
                                if let Some(ref callback) = energy_callback {
                                    callback(energy);
                                }
                                *last = now;
                            }
                        }
                    }

                    // Write samples to ring buffer
                    if let Ok(mut buffer) = buffer.lock() {
                        for sample in samples {
                            // If buffer is full, oldest samples will be overwritten
                            let _ = buffer.push_overwrite(sample);
                        }
                    }
                },
                move |err| {
                    error!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| VrctError::AudioDevice(format!("Failed to build stream: {}", e)))?;

        Ok(stream)
    }

    /// Set the energy threshold for voice activity detection
    pub fn set_energy_threshold(&self, threshold: f32) {
        let clamped = threshold.clamp(0.0, 1.0);
        self.energy_threshold.store(clamped.to_bits(), Ordering::Relaxed);
        debug!("Energy threshold set to: {}", clamped);
    }

    /// Get the current energy threshold
    pub fn get_energy_threshold(&self) -> f32 {
        f32::from_bits(self.energy_threshold.load(Ordering::Relaxed))
    }

    /// Get the current energy level
    pub fn get_current_energy(&self) -> f32 {
        f32::from_bits(self.current_energy.load(Ordering::Relaxed))
    }

    /// Enable or disable voice activity detection
    pub fn set_vad_enabled(&self, enabled: bool) {
        self.vad_enabled.store(enabled, Ordering::Relaxed);
        debug!("VAD enabled: {}", enabled);
    }

    /// Check if VAD is enabled
    pub fn is_vad_enabled(&self) -> bool {
        self.vad_enabled.load(Ordering::Relaxed)
    }

    /// Check if the recorder is currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    /// Read audio samples from the buffer
    /// Returns None if no audio is available or if energy is below threshold (when VAD is enabled)
    pub fn read_audio(&self) -> Option<Vec<f32>> {
        if !self.is_recording() {
            return None;
        }

        // Check energy threshold if VAD is enabled
        if self.is_vad_enabled() {
            let current_energy = self.get_current_energy();
            let threshold = self.get_energy_threshold();
            
            if current_energy < threshold {
                debug!("Energy {} below threshold {}, skipping", current_energy, threshold);
                return None;
            }
        }

        // Read samples from buffer
        if let Ok(mut buffer) = self.buffer.lock() {
            if buffer.occupied_len() == 0 {
                return None;
            }

            let mut samples = Vec::with_capacity(buffer.occupied_len());
            while let Some(sample) = buffer.try_pop() {
                samples.push(sample);
            }

            if samples.is_empty() {
                None
            } else {
                debug!("Read {} audio samples", samples.len());
                Some(samples)
            }
        } else {
            warn!("Failed to lock audio buffer");
            None
        }
    }

    /// Get the device info
    pub fn device_info(&self) -> &AudioDevice {
        &self.device_info
    }

    /// Get the number of samples currently in the buffer
    pub fn buffer_len(&self) -> usize {
        if let Ok(buffer) = self.buffer.lock() {
            buffer.occupied_len()
        } else {
            0
        }
    }

    /// Set a callback for energy level updates
    /// The callback will be called at the configured interval when recording
    pub fn set_energy_callback<F>(&mut self, callback: F)
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.energy_callback = Some(Arc::new(callback));
        debug!("Energy callback registered");
    }

    /// Clear the energy callback
    pub fn clear_energy_callback(&mut self) {
        self.energy_callback = None;
        debug!("Energy callback cleared");
    }

    /// Enable or disable energy reporting
    /// Note: This requires restarting the stream to take effect
    pub fn set_energy_reporting_enabled(&mut self, enabled: bool) {
        self.config.enable_energy_reporting = enabled;
        debug!("Energy reporting enabled: {}", enabled);
    }

    /// Check if energy reporting is enabled
    pub fn is_energy_reporting_enabled(&self) -> bool {
        self.config.enable_energy_reporting
    }

    /// Set the energy reporting interval in milliseconds
    /// Note: This requires restarting the stream to take effect
    pub fn set_energy_report_interval(&mut self, interval_ms: u64) {
        self.config.energy_report_interval_ms = interval_ms;
        debug!("Energy report interval set to: {} ms", interval_ms);
    }

    /// Get the energy reporting interval in milliseconds
    pub fn get_energy_report_interval(&self) -> u64 {
        self.config.energy_report_interval_ms
    }
}

impl Drop for AudioRecorder {
    fn drop(&mut self) {
        if self.is_recording() {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::device_manager::DeviceManager;

    #[test]
    fn test_recorder_config_default() {
        let config = RecorderConfig::default();
        assert_eq!(config.energy_threshold, 0.01);
        assert!(config.vad_enabled);
        assert_eq!(config.buffer_size, 48000);
    }

    #[test]
    fn test_create_recorder() {
        let manager = DeviceManager::new();
        
        if let Ok(Some(device)) = manager.get_default_input() {
            let config = RecorderConfig::default();
            let result = AudioRecorder::new(device, config);
            
            assert!(result.is_ok());
            let recorder = result.unwrap();
            assert!(!recorder.is_recording());
            assert_eq!(recorder.get_energy_threshold(), 0.01);
            assert!(recorder.is_vad_enabled());
        }
    }

    #[test]
    fn test_set_energy_threshold() {
        let manager = DeviceManager::new();
        
        if let Ok(Some(device)) = manager.get_default_input() {
            let config = RecorderConfig::default();
            let recorder = AudioRecorder::new(device, config).unwrap();
            
            recorder.set_energy_threshold(0.5);
            assert_eq!(recorder.get_energy_threshold(), 0.5);
            
            // Test clamping
            recorder.set_energy_threshold(1.5);
            assert_eq!(recorder.get_energy_threshold(), 1.0);
            
            recorder.set_energy_threshold(-0.5);
            assert_eq!(recorder.get_energy_threshold(), 0.0);
        }
    }

    #[test]
    fn test_vad_enable_disable() {
        let manager = DeviceManager::new();
        
        if let Ok(Some(device)) = manager.get_default_input() {
            let config = RecorderConfig::default();
            let recorder = AudioRecorder::new(device, config).unwrap();
            
            assert!(recorder.is_vad_enabled());
            
            recorder.set_vad_enabled(false);
            assert!(!recorder.is_vad_enabled());
            
            recorder.set_vad_enabled(true);
            assert!(recorder.is_vad_enabled());
        }
    }

    #[test]
    fn test_start_stop_recording() {
        let manager = DeviceManager::new();
        
        if let Ok(Some(device)) = manager.get_default_input() {
            let config = RecorderConfig::default();
            let mut recorder = AudioRecorder::new(device, config).unwrap();
            
            assert!(!recorder.is_recording());
            
            // Start recording
            let result = recorder.start();
            if result.is_ok() {
                assert!(recorder.is_recording());
                
                // Stop recording
                let result = recorder.stop();
                assert!(result.is_ok());
                assert!(!recorder.is_recording());
            }
        }
    }

    #[test]
    fn test_read_audio_when_not_recording() {
        let manager = DeviceManager::new();
        
        if let Ok(Some(device)) = manager.get_default_input() {
            let config = RecorderConfig::default();
            let recorder = AudioRecorder::new(device, config).unwrap();
            
            // Should return None when not recording
            assert!(recorder.read_audio().is_none());
        }
    }

    #[test]
    fn test_energy_reporting_config() {
        let manager = DeviceManager::new();
        
        if let Ok(Some(device)) = manager.get_default_input() {
            let config = RecorderConfig {
                enable_energy_reporting: true,
                energy_report_interval_ms: 50,
                ..Default::default()
            };
            let mut recorder = AudioRecorder::new(device, config).unwrap();
            
            assert!(recorder.is_energy_reporting_enabled());
            assert_eq!(recorder.get_energy_report_interval(), 50);
            
            recorder.set_energy_reporting_enabled(false);
            assert!(!recorder.is_energy_reporting_enabled());
            
            recorder.set_energy_report_interval(200);
            assert_eq!(recorder.get_energy_report_interval(), 200);
        }
    }

    #[test]
    fn test_energy_callback_registration() {
        let manager = DeviceManager::new();
        
        if let Ok(Some(device)) = manager.get_default_input() {
            let config = RecorderConfig::default();
            let mut recorder = AudioRecorder::new(device, config).unwrap();
            
            // Register a callback
            let called = Arc::new(AtomicBool::new(false));
            let called_clone = Arc::clone(&called);
            
            recorder.set_energy_callback(move |_energy| {
                called_clone.store(true, Ordering::Relaxed);
            });
            
            // Clear callback
            recorder.clear_energy_callback();
        }
    }
}
