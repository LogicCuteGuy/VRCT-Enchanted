use thiserror::Error;

/// Main error type for VRCT application
#[derive(Debug, Error)]
pub enum VrctError {
    #[error("Audio device error: {0}")]
    AudioDevice(String),
    
    #[error("Audio recording error: {0}")]
    AudioRecording(String),
    
    #[error("Transcription error: {0}")]
    Transcription(String),
    
    #[error("Translation error: {0}")]
    Translation(String),
    
    #[error("OSC communication error: {0}")]
    Osc(String),
    
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    
    #[error("Overlay error: {0}")]
    Overlay(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("VRAM overflow: {0}")]
    VramOverflow(String),
    
    #[error("ZLUDA runtime error: {0}")]
    ZludaRuntime(String),
    
    #[error("Model loading error: {0}")]
    ModelLoading(String),
    
    #[error("Model download error: {0}")]
    ModelDownload(String),
    
    #[error("Initialization error: {0}")]
    Initialization(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Candle ML error: {0}")]
    Candle(String),
    
    #[error("Update error: {0}")]
    Update(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

// Manual From implementation for candle_core::Error
impl From<candle_core::Error> for VrctError {
    fn from(err: candle_core::Error) -> Self {
        let error_msg = format!("{}", err);
        // Check for VRAM errors
        if error_msg.contains("out of memory") || error_msg.contains("CUBLAS_STATUS_ALLOC_FAILED") {
            VrctError::VramOverflow(error_msg)
        } else {
            VrctError::Candle(error_msg)
        }
    }
}

/// Result type alias for VRCT operations
pub type Result<T> = std::result::Result<T, VrctError>;

impl VrctError {
    /// Create a new error with a custom message
    pub fn custom(msg: impl Into<String>) -> Self {
        VrctError::Unknown(msg.into())
    }
    
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            VrctError::AudioDevice(_)
                | VrctError::Translation(_)
                | VrctError::Osc(_)
                | VrctError::WebSocket(_)
                | VrctError::VramOverflow(_)
                | VrctError::ZludaRuntime(_)
        )
    }
    
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            VrctError::AudioDevice(msg) => format!("Audio device error: {}", msg),
            VrctError::AudioRecording(msg) => format!("Audio recording error: {}", msg),
            VrctError::Transcription(msg) => format!("Transcription failed: {}", msg),
            VrctError::Translation(msg) => format!("Translation failed: {}", msg),
            VrctError::Osc(msg) => format!("OSC communication error: {}", msg),
            VrctError::WebSocket(msg) => format!("WebSocket error: {}", msg),
            VrctError::Overlay(msg) => format!("VR overlay error: {}", msg),
            VrctError::Config(msg) => format!("Configuration error: {}", msg),
            VrctError::VramOverflow(msg) => format!("GPU memory overflow: {}", msg),
            VrctError::ZludaRuntime(msg) => format!("ZLUDA runtime error: {}", msg),
            VrctError::ModelLoading(msg) => format!("Model loading error: {}", msg),
            VrctError::ModelDownload(msg) => format!("Model download error: {}", msg),
            VrctError::Initialization(msg) => format!("Initialization error: {}", msg),
            VrctError::Io(err) => format!("IO error: {}", err),
            VrctError::Serialization(err) => format!("Data serialization error: {}", err),
            VrctError::Http(err) => format!("Network error: {}", err),
            VrctError::Candle(msg) => format!("ML framework error: {}", msg),
            VrctError::Update(msg) => format!("Update error: {}", msg),
            VrctError::Unknown(msg) => format!("Error: {}", msg),
        }
    }
}
