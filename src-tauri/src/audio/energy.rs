// Energy detection and voice activity detection
// Implements RMS energy calculation and automatic threshold adjustment

use std::collections::VecDeque;
use tracing::{debug, trace};

/// Configuration for energy detection
#[derive(Debug, Clone)]
pub struct EnergyDetectorConfig {
    /// Initial energy threshold (0.0 - 1.0)
    pub initial_threshold: f32,
    /// Whether automatic threshold adjustment is enabled
    pub auto_adjust: bool,
    /// Number of samples to use for ambient noise estimation
    pub ambient_window_size: usize,
    /// Multiplier for ambient noise to set threshold (e.g., 2.0 = 2x ambient)
    pub threshold_multiplier: f32,
    /// Minimum threshold value to prevent over-sensitivity
    pub min_threshold: f32,
    /// Maximum threshold value to prevent under-sensitivity
    pub max_threshold: f32,
}

impl Default for EnergyDetectorConfig {
    fn default() -> Self {
        Self {
            initial_threshold: 0.01,
            auto_adjust: true,
            ambient_window_size: 100, // ~2 seconds at 50 updates/sec
            threshold_multiplier: 2.0,
            min_threshold: 0.001,
            max_threshold: 0.5,
        }
    }
}

/// Energy detector for voice activity detection
pub struct EnergyDetector {
    /// Current energy threshold
    threshold: f32,
    /// Configuration
    config: EnergyDetectorConfig,
    /// Rolling window of recent energy values for ambient noise estimation
    ambient_window: VecDeque<f32>,
    /// Current ambient noise level
    ambient_level: f32,
}

impl EnergyDetector {
    /// Create a new energy detector with the given configuration
    pub fn new(config: EnergyDetectorConfig) -> Self {
        Self {
            threshold: config.initial_threshold,
            config,
            ambient_window: VecDeque::new(),
            ambient_level: 0.0,
        }
    }

    /// Calculate RMS (Root Mean Square) energy of audio samples
    pub fn calculate_rms_energy(samples: &[f32]) -> f32 {
        use crate::utils::profiling::global_profiler;
        let _guard = global_profiler().start_timing("audio_energy_calculation");
        
        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Update the detector with a new energy value
    /// Returns the current threshold after any adjustments
    pub fn update(&mut self, energy: f32) -> f32 {
        if self.config.auto_adjust {
            self.update_ambient_level(energy);
            self.adjust_threshold();
        }
        
        self.threshold
    }

    /// Update the ambient noise level estimate
    fn update_ambient_level(&mut self, energy: f32) {
        // Add new energy value to window
        self.ambient_window.push_back(energy);
        
        // Remove old values if window is full
        while self.ambient_window.len() > self.config.ambient_window_size {
            self.ambient_window.pop_front();
        }
        
        // Calculate ambient level as the average of the window
        if !self.ambient_window.is_empty() {
            let sum: f32 = self.ambient_window.iter().sum();
            self.ambient_level = sum / self.ambient_window.len() as f32;
            trace!("Ambient level updated to: {}", self.ambient_level);
        }
    }

    /// Adjust the threshold based on ambient noise level
    fn adjust_threshold(&mut self) {
        if self.ambient_level > 0.0 {
            // Set threshold as a multiple of ambient noise
            let new_threshold = self.ambient_level * self.config.threshold_multiplier;
            
            // Clamp to min/max values
            self.threshold = new_threshold.clamp(
                self.config.min_threshold,
                self.config.max_threshold,
            );
            
            trace!("Threshold adjusted to: {}", self.threshold);
        }
    }

    /// Check if the given energy exceeds the threshold
    pub fn is_above_threshold(&self, energy: f32) -> bool {
        energy >= self.threshold
    }

    /// Filter audio samples based on energy threshold
    /// Returns Some(samples) if energy is above threshold, None otherwise
    pub fn filter_by_threshold(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
        let energy = Self::calculate_rms_energy(samples);
        
        // Update threshold if auto-adjust is enabled
        if self.config.auto_adjust {
            self.update(energy);
        }
        
        if self.is_above_threshold(energy) {
            debug!("Energy {} above threshold {}, passing samples", energy, self.threshold);
            Some(samples.to_vec())
        } else {
            trace!("Energy {} below threshold {}, filtering out", energy, self.threshold);
            None
        }
    }

    /// Get the current energy threshold
    pub fn get_threshold(&self) -> f32 {
        self.threshold
    }

    /// Set the energy threshold manually
    /// This will be overridden if auto-adjust is enabled
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(
            self.config.min_threshold,
            self.config.max_threshold,
        );
        debug!("Threshold manually set to: {}", self.threshold);
    }

    /// Get the current ambient noise level
    pub fn get_ambient_level(&self) -> f32 {
        self.ambient_level
    }

    /// Enable or disable automatic threshold adjustment
    pub fn set_auto_adjust(&mut self, enabled: bool) {
        self.config.auto_adjust = enabled;
        debug!("Auto-adjust set to: {}", enabled);
    }

    /// Check if automatic threshold adjustment is enabled
    pub fn is_auto_adjust_enabled(&self) -> bool {
        self.config.auto_adjust
    }

    /// Reset the ambient noise estimation
    pub fn reset_ambient(&mut self) {
        self.ambient_window.clear();
        self.ambient_level = 0.0;
        self.threshold = self.config.initial_threshold;
        debug!("Ambient noise estimation reset");
    }

    /// Get the configuration
    pub fn config(&self) -> &EnergyDetectorConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: EnergyDetectorConfig) {
        self.config = config;
        self.threshold = self.config.initial_threshold;
        self.reset_ambient();
        debug!("Configuration updated");
    }
}

impl Default for EnergyDetector {
    fn default() -> Self {
        Self::new(EnergyDetectorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_rms_energy_empty() {
        let samples: Vec<f32> = vec![];
        let energy = EnergyDetector::calculate_rms_energy(&samples);
        assert_eq!(energy, 0.0);
    }

    #[test]
    fn test_calculate_rms_energy_silence() {
        let samples = vec![0.0, 0.0, 0.0, 0.0];
        let energy = EnergyDetector::calculate_rms_energy(&samples);
        assert_eq!(energy, 0.0);
    }

    #[test]
    fn test_calculate_rms_energy_constant() {
        let samples = vec![0.5, 0.5, 0.5, 0.5];
        let energy = EnergyDetector::calculate_rms_energy(&samples);
        assert_eq!(energy, 0.5);
    }

    #[test]
    fn test_calculate_rms_energy_varying() {
        let samples = vec![0.0, 0.5, 1.0, 0.5];
        let energy = EnergyDetector::calculate_rms_energy(&samples);
        // RMS of [0, 0.5, 1.0, 0.5] = sqrt((0 + 0.25 + 1.0 + 0.25) / 4) = sqrt(0.375) ≈ 0.612
        assert!((energy - 0.612).abs() < 0.001);
    }

    #[test]
    fn test_energy_detector_creation() {
        let config = EnergyDetectorConfig::default();
        let detector = EnergyDetector::new(config.clone());
        
        assert_eq!(detector.get_threshold(), config.initial_threshold);
        assert_eq!(detector.get_ambient_level(), 0.0);
        assert!(detector.is_auto_adjust_enabled());
    }

    #[test]
    fn test_manual_threshold_setting() {
        let mut detector = EnergyDetector::default();
        
        detector.set_threshold(0.05);
        assert_eq!(detector.get_threshold(), 0.05);
        
        // Test clamping to max
        detector.set_threshold(1.0);
        assert_eq!(detector.get_threshold(), 0.5); // max_threshold is 0.5
        
        // Test clamping to min
        detector.set_threshold(0.0);
        assert_eq!(detector.get_threshold(), 0.001); // min_threshold is 0.001
    }

    #[test]
    fn test_is_above_threshold() {
        let mut detector = EnergyDetector::default();
        detector.set_threshold(0.1);
        
        assert!(detector.is_above_threshold(0.15));
        assert!(detector.is_above_threshold(0.1));
        assert!(!detector.is_above_threshold(0.05));
    }

    #[test]
    fn test_filter_by_threshold_without_auto_adjust() {
        let config = EnergyDetectorConfig {
            auto_adjust: false,
            initial_threshold: 0.1,
            ..Default::default()
        };
        let mut detector = EnergyDetector::new(config);
        
        // High energy samples should pass
        let high_energy = vec![0.5, 0.5, 0.5, 0.5]; // RMS = 0.5
        let result = detector.filter_by_threshold(&high_energy);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), high_energy);
        
        // Low energy samples should be filtered
        let low_energy = vec![0.01, 0.01, 0.01, 0.01]; // RMS = 0.01
        let result = detector.filter_by_threshold(&low_energy);
        assert!(result.is_none());
    }

    #[test]
    fn test_ambient_level_update() {
        let config = EnergyDetectorConfig {
            auto_adjust: true,
            ambient_window_size: 5,
            ..Default::default()
        };
        let mut detector = EnergyDetector::new(config);
        
        // Feed some energy values
        detector.update(0.1);
        detector.update(0.2);
        detector.update(0.1);
        detector.update(0.2);
        detector.update(0.1);
        
        // Ambient level should be average: (0.1 + 0.2 + 0.1 + 0.2 + 0.1) / 5 = 0.14
        let ambient = detector.get_ambient_level();
        assert!((ambient - 0.14).abs() < 0.001);
    }

    #[test]
    fn test_automatic_threshold_adjustment() {
        let config = EnergyDetectorConfig {
            auto_adjust: true,
            ambient_window_size: 5,
            threshold_multiplier: 2.0,
            min_threshold: 0.001,
            max_threshold: 0.5,
            ..Default::default()
        };
        let mut detector = EnergyDetector::new(config);
        
        // Feed consistent low energy (simulating quiet environment)
        for _ in 0..5 {
            detector.update(0.05);
        }
        
        // Threshold should adjust to 2x ambient (0.05 * 2.0 = 0.1)
        let threshold = detector.get_threshold();
        assert!((threshold - 0.1).abs() < 0.001);
        
        // Feed higher energy (simulating louder environment)
        for _ in 0..5 {
            detector.update(0.2);
        }
        
        // Threshold should adjust to 2x new ambient (0.2 * 2.0 = 0.4)
        let threshold = detector.get_threshold();
        assert!((threshold - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_threshold_clamping_in_auto_adjust() {
        let config = EnergyDetectorConfig {
            auto_adjust: true,
            ambient_window_size: 5,
            threshold_multiplier: 10.0, // Very high multiplier
            min_threshold: 0.001,
            max_threshold: 0.5,
            ..Default::default()
        };
        let mut detector = EnergyDetector::new(config);
        
        // Feed high energy values
        for _ in 0..5 {
            detector.update(0.1);
        }
        
        // Threshold should be clamped to max (0.1 * 10.0 = 1.0, but max is 0.5)
        let threshold = detector.get_threshold();
        assert_eq!(threshold, 0.5);
    }

    #[test]
    fn test_reset_ambient() {
        let mut detector = EnergyDetector::default();
        
        // Build up some ambient history
        for _ in 0..10 {
            detector.update(0.1);
        }
        
        assert!(detector.get_ambient_level() > 0.0);
        
        // Reset
        detector.reset_ambient();
        
        assert_eq!(detector.get_ambient_level(), 0.0);
        assert_eq!(detector.get_threshold(), detector.config().initial_threshold);
    }

    #[test]
    fn test_auto_adjust_enable_disable() {
        let mut detector = EnergyDetector::default();
        
        assert!(detector.is_auto_adjust_enabled());
        
        detector.set_auto_adjust(false);
        assert!(!detector.is_auto_adjust_enabled());
        
        // Set a manual threshold
        detector.set_threshold(0.2);
        
        // Feed energy values - threshold should not change
        detector.update(0.5);
        detector.update(0.5);
        detector.update(0.5);
        
        assert_eq!(detector.get_threshold(), 0.2);
        
        // Re-enable auto-adjust
        detector.set_auto_adjust(true);
        
        // Feed energy values - threshold should now adjust
        for _ in 0..10 {
            detector.update(0.1);
        }
        
        // Threshold should have adjusted (0.1 * 2.0 = 0.2)
        assert!((detector.get_threshold() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_filter_with_auto_adjust() {
        let config = EnergyDetectorConfig {
            auto_adjust: true,
            ambient_window_size: 3,
            threshold_multiplier: 2.0,
            ..Default::default()
        };
        let mut detector = EnergyDetector::new(config);
        
        // Start with low ambient noise
        let quiet_samples = vec![0.05; 100]; // RMS = 0.05
        for _ in 0..3 {
            detector.filter_by_threshold(&quiet_samples);
        }
        
        // Threshold should be around 0.1 (0.05 * 2.0)
        let threshold = detector.get_threshold();
        assert!((threshold - 0.1).abs() < 0.01);
        
        // Now samples with energy 0.05 should be filtered out
        let result = detector.filter_by_threshold(&quiet_samples);
        assert!(result.is_none());
        
        // But louder samples should pass
        let loud_samples = vec![0.2; 100]; // RMS = 0.2
        let result = detector.filter_by_threshold(&loud_samples);
        assert!(result.is_some());
    }

    #[test]
    fn test_config_update() {
        let mut detector = EnergyDetector::default();
        
        // Build up some state
        for _ in 0..10 {
            detector.update(0.1);
        }
        
        let old_ambient = detector.get_ambient_level();
        assert!(old_ambient > 0.0);
        
        // Update config
        let new_config = EnergyDetectorConfig {
            initial_threshold: 0.2,
            auto_adjust: false,
            ..Default::default()
        };
        detector.set_config(new_config);
        
        // State should be reset
        assert_eq!(detector.get_ambient_level(), 0.0);
        assert_eq!(detector.get_threshold(), 0.2);
        assert!(!detector.is_auto_adjust_enabled());
    }
}
