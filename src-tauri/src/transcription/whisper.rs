// Whisper model integration using Candle framework
// Provides transcription functionality with device selection and model downloading

use crate::utils::error::{Result, VrctError};
use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self as m, audio, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

/// Whisper model types available for download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhisperModelType {
    Tiny,
    Base,
    Small,
    Medium,
    LargeV1,
    LargeV2,
    LargeV3,
    LargeV3TurboInt8,
    LargeV3Turbo,
}

impl WhisperModelType {
    /// Get the Hugging Face repository ID for this model type
    pub fn repo_id(&self) -> &str {
        match self {
            Self::Tiny => "Systran/faster-whisper-tiny",
            Self::Base => "Systran/faster-whisper-base",
            Self::Small => "Systran/faster-whisper-small",
            Self::Medium => "Systran/faster-whisper-medium",
            Self::LargeV1 => "Systran/faster-whisper-large-v1",
            Self::LargeV2 => "Systran/faster-whisper-large-v2",
            Self::LargeV3 => "Systran/faster-whisper-large-v3",
            Self::LargeV3TurboInt8 => "Zoont/faster-whisper-large-v3-turbo-int8-ct2",
            Self::LargeV3Turbo => "deepdml/faster-whisper-large-v3-turbo-ct2",
        }
    }

    /// Get the string identifier for this model type
    pub fn as_str(&self) -> &str {
        match self {
            Self::Tiny => "tiny",
            Self::Base => "base",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::LargeV1 => "large-v1",
            Self::LargeV2 => "large-v2",
            Self::LargeV3 => "large-v3",
            Self::LargeV3TurboInt8 => "large-v3-turbo-int8",
            Self::LargeV3Turbo => "large-v3-turbo",
        }
    }

    /// Parse a model type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "tiny" => Some(Self::Tiny),
            "base" => Some(Self::Base),
            "small" => Some(Self::Small),
            "medium" => Some(Self::Medium),
            "large-v1" => Some(Self::LargeV1),
            "large-v2" => Some(Self::LargeV2),
            "large-v3" => Some(Self::LargeV3),
            "large-v3-turbo-int8" => Some(Self::LargeV3TurboInt8),
            "large-v3-turbo" => Some(Self::LargeV3Turbo),
            _ => None,
        }
    }
}

/// Compute device selection for model inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeDevice {
    Cpu,
    Cuda(usize),  // GPU index
    Zluda(usize), // AMD GPU via ZLUDA
}

impl ComputeDevice {
    /// Convert to Candle Device
    pub fn to_candle_device(&self) -> Result<Device> {
        match self {
            Self::Cpu => Ok(Device::Cpu),
            Self::Cuda(index) => Device::new_cuda(*index)
                .map_err(|e| VrctError::Transcription(format!("Failed to initialize CUDA device: {}", e))),
            Self::Zluda(index) => {
                // ZLUDA uses CUDA API, so we try CUDA device
                Device::new_cuda(*index)
                    .map_err(|e| VrctError::ZludaRuntime(format!("Failed to initialize ZLUDA device: {}", e)))
            }
        }
    }
}

/// Transcription parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionParams {
    pub language: Option<String>,
    pub task: String, // "transcribe" or "translate"
    pub temperature: f32,
    pub best_of: usize,
    pub beam_size: usize,
    pub patience: f32,
    pub length_penalty: f32,
    pub suppress_tokens: Vec<i64>,
    pub initial_prompt: Option<String>,
    pub condition_on_previous_text: bool,
    pub temperature_increment_on_fallback: f32,
    pub compression_ratio_threshold: f32,
    pub logprob_threshold: f32,
    pub no_speech_threshold: f32,
}

impl Default for TranscriptionParams {
    fn default() -> Self {
        Self {
            language: None,
            task: "transcribe".to_string(),
            temperature: 0.0,
            best_of: 5,
            beam_size: 5,
            patience: 1.0,
            length_penalty: 1.0,
            suppress_tokens: vec![-1],
            initial_prompt: None,
            condition_on_previous_text: true,
            temperature_increment_on_fallback: 0.2,
            compression_ratio_threshold: 2.4,
            logprob_threshold: -1.0,
            no_speech_threshold: 0.6,
        }
    }
}

/// Result of a transcription operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: String,
    pub confidence: f32,
}

/// Whisper transcriber using Candle framework
pub struct WhisperTranscriber {
    #[allow(dead_code)] // Placeholder for full Candle Whisper pipeline
    model: m::model::Whisper,
    #[allow(dead_code)] // Placeholder for full Candle Whisper pipeline
    config: Config,
    #[allow(dead_code)] // Placeholder for full Candle Whisper pipeline
    tokenizer: tokenizers::Tokenizer,
    device: Device,
    params: Arc<RwLock<TranscriptionParams>>,
    mel_filters: Vec<f32>,
}

impl WhisperTranscriber {
    /// Create a new WhisperTranscriber
    pub async fn new(
        model_path: &Path,
        device: ComputeDevice,
        params: TranscriptionParams,
    ) -> Result<Self> {
        info!("Loading Whisper model from {:?} on device {:?}", model_path, device);
        
        let candle_device = device.to_candle_device()?;
        
        // Load config
        let config_path = model_path.join("config.json");
        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| VrctError::Transcription(format!("Failed to read config: {}", e)))?;
        let config: Config = serde_json::from_str(&config_str)
            .map_err(|e| VrctError::Transcription(format!("Failed to parse config: {}", e)))?;
        
        // Load tokenizer
        let tokenizer_path = model_path.join("tokenizer.json");
        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| VrctError::Transcription(format!("Failed to load tokenizer: {}", e)))?;
        
        // Load model weights
        let weights_path = model_path.join("model.safetensors");
        let vb = if weights_path.exists() {
            unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &candle_device)? }
        } else {
            // Try .bin format
            let weights_path = model_path.join("model.bin");
            VarBuilder::from_pth(&weights_path, DType::F32, &candle_device)?
        };
        
        let model = m::model::Whisper::load(&vb, config.clone())
            .map_err(|e| {
                // Check for VRAM errors
                let error_msg = format!("{}", e);
                if error_msg.contains("out of memory") || error_msg.contains("CUBLAS_STATUS_ALLOC_FAILED") {
                    VrctError::VramOverflow(error_msg)
                } else {
                    VrctError::Transcription(format!("Failed to load model: {}", e))
                }
            })?;
        
        // Create mel filters - for now use a placeholder
        // The actual mel filter generation will be implemented when we have the full pipeline
        let mel_filters = vec![0.0f32; config.num_mel_bins * 201]; // 201 = n_fft/2 + 1 for n_fft=400
        
        info!("Whisper model loaded successfully");
        
        Ok(Self {
            model,
            config,
            tokenizer,
            device: candle_device,
            params: Arc::new(RwLock::new(params)),
            mel_filters,
        })
    }

    /// Transcribe audio samples
    /// Audio should be 16kHz mono f32 samples
    pub async fn transcribe(&self, audio: &[f32]) -> Result<TranscriptionResult> {
        use crate::utils::profiling::global_profiler;
        let _guard = global_profiler().start_timing("transcription_whisper");
        
        debug!("Transcribing {} audio samples", audio.len());
        
        // Convert audio to mel spectrogram
        // For now, this is a placeholder - full mel spectrogram conversion will be implemented
        let _mel_data = audio::pcm_to_mel(&self.config, audio, &self.mel_filters);
        
        // Create a placeholder mel tensor
        let _mel = Tensor::zeros((1, self.config.num_mel_bins, 3000), DType::F32, &self.device)?;
        
        // Get transcription parameters
        let params = self.params.read().unwrap().clone();
        
        // Get language
        let language = params.language.as_deref().unwrap_or("en");
        
        // For now, return a placeholder implementation
        // The full Candle Whisper decoder implementation requires more setup
        // This will be completed when we have the full model pipeline working
        let text = format!("Transcription placeholder - {} samples processed", audio.len());
        let confidence = 0.0;
        
        debug!("Transcription complete: {} chars, confidence: {:.2}", text.len(), confidence);
        
        Ok(TranscriptionResult {
            text: text.trim().to_string(),
            language: language.to_string(),
            confidence,
        })
    }
    
    /// Update transcription parameters at runtime
    pub fn update_params(&self, params: TranscriptionParams) {
        let mut current_params = self.params.write().unwrap();
        *current_params = params;
        info!("Transcription parameters updated");
    }
    
    /// Get current transcription parameters
    pub fn get_params(&self) -> TranscriptionParams {
        self.params.read().unwrap().clone()
    }
}

/// Download a Whisper model from Hugging Face
pub async fn download_model<F>(
    root: &Path,
    model_type: WhisperModelType,
    progress_callback: F,
) -> Result<PathBuf>
where
    F: Fn(f32) + Send + 'static,
{
    info!("Downloading Whisper model: {:?}", model_type);
    
    let model_path = root.join("weights").join("whisper").join(model_type.as_str());
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&model_path)
        .map_err(|e| VrctError::Transcription(format!("Failed to create model directory: {}", e)))?;
    
    // Check if model already exists
    if check_model_exists(&model_path) {
        info!("Model already exists at {:?}", model_path);
        progress_callback(1.0);
        return Ok(model_path);
    }
    
    // Download from Hugging Face
    let api = Api::new()
        .map_err(|e| VrctError::Transcription(format!("Failed to initialize HF API: {}", e)))?;
    
    let repo = api.repo(Repo::new(
        model_type.repo_id().to_string(),
        RepoType::Model,
    ));
    
    // Files to download
    let files = vec![
        "config.json",
        "preprocessor_config.json",
        "model.safetensors",
        "tokenizer.json",
        "vocabulary.txt",
        "vocabulary.json",
    ];
    
    let total_files = files.len();
    let mut downloaded = 0;
    
    for filename in files {
        debug!("Downloading {}", filename);
        
        match repo.get(filename) {
            Ok(file_path) => {
                // Copy to model directory
                let dest = model_path.join(filename);
                std::fs::copy(&file_path, &dest)
                    .map_err(|e| VrctError::Transcription(format!("Failed to copy {}: {}", filename, e)))?;
                
                downloaded += 1;
                let progress = downloaded as f32 / total_files as f32;
                progress_callback(progress);
            }
            Err(e) => {
                warn!("Failed to download {}: {}, trying alternative", filename, e);
                
                // Try alternative filename (e.g., model.bin instead of model.safetensors)
                if filename == "model.safetensors" {
                    if let Ok(file_path) = repo.get("model.bin") {
                        let dest = model_path.join("model.bin");
                        std::fs::copy(&file_path, &dest)
                            .map_err(|e| VrctError::Transcription(format!("Failed to copy model.bin: {}", e)))?;
                        downloaded += 1;
                        let progress = downloaded as f32 / total_files as f32;
                        progress_callback(progress);
                        continue;
                    }
                }
                
                // If it's not a critical file, continue
                if filename != "config.json" && filename != "tokenizer.json" {
                    downloaded += 1;
                    let progress = downloaded as f32 / total_files as f32;
                    progress_callback(progress);
                    continue;
                }
                
                return Err(VrctError::Transcription(format!("Failed to download required file {}: {}", filename, e)));
            }
        }
    }
    
    info!("Model download complete: {:?}", model_path);
    Ok(model_path)
}

/// Check if a model exists and is valid
pub fn check_model_exists(model_path: &Path) -> bool {
    let required_files = vec!["config.json", "tokenizer.json"];
    
    for file in required_files {
        if !model_path.join(file).exists() {
            return false;
        }
    }
    
    // Check for either safetensors or bin format
    let has_weights = model_path.join("model.safetensors").exists()
        || model_path.join("model.bin").exists();
    
    has_weights
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_type_conversion() {
        assert_eq!(WhisperModelType::from_str("tiny").unwrap().as_str(), "tiny");
        assert_eq!(WhisperModelType::from_str("base").unwrap().as_str(), "base");
        assert_eq!(WhisperModelType::from_str("large-v3").unwrap().as_str(), "large-v3");
        assert!(WhisperModelType::from_str("invalid").is_none());
    }
    
    #[test]
    fn test_default_params() {
        let params = TranscriptionParams::default();
        assert_eq!(params.task, "transcribe");
        assert_eq!(params.temperature, 0.0);
        assert_eq!(params.beam_size, 5);
    }
}
