// Message data structures for commands and responses

use serde::{Deserialize, Serialize};

/// Command enum representing all possible commands from the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Command {
    // Main Window Commands
    EnableTranslation,
    DisableTranslation,
    EnableTranscriptionSend,
    DisableTranscriptionSend,
    EnableTranscriptionReceive,
    DisableTranscriptionReceive,
    SendMessageBox { message: String },
    StartTyping,
    StopTyping,
    
    // Audio Device Commands
    GetAudioDevices,
    SetMicDevice { host: String, device: String },
    SetSpeakerDevice { device: String },
    SetMicThreshold { threshold: u32 },
    SetSpeakerThreshold { threshold: u32 },
    EnableAutoMicThreshold { enabled: bool },
    EnableAutoSpeakerThreshold { enabled: bool },
    EnableMicThresholdCheck,
    DisableMicThresholdCheck,
    EnableSpeakerThresholdCheck,
    DisableSpeakerThresholdCheck,
    
    // Transcription Commands
    StartTranscription,
    StopTranscription,
    SetTranscriptionEngine { engine: String },
    SetWhisperModel { model: String },
    DownloadWhisperModel { model: String },
    
    // Translation Commands
    TranslateText { text: String, source: String, target: String },
    SetTranslationEngine { engine: String },
    SetSourceLanguage { language: String },
    SetTargetLanguage { language: String },
    
    // Configuration Commands
    GetConfig,
    SetConfig { config: serde_json::Value },
    SetTransparency { value: u8 },
    SetUILanguage { language: String },
    SetUIScaling { scaling: u8 },
    
    // OSC Commands
    SetOscSettings { ip: String, port: u16 },
    EnableOscSend { enabled: bool },
    EnableMicMuteSync { enabled: bool },
    
    // WebSocket Commands
    SetWebSocketSettings { host: String, port: u16 },
    EnableWebSocketServer { enabled: bool },
    
    // Overlay Commands
    EnableSmallLogOverlay { enabled: bool },
    EnableLargeLogOverlay { enabled: bool },
    SetOverlaySettings { settings: serde_json::Value },
    
    // Word Filter Commands
    SetWordFilter { words: Vec<String> },
    
    // Logging Commands
    EnableLogging { enabled: bool },
    OpenLogsFolder,
    OpenConfigFolder,
    
    // System Commands
    Shutdown,
    GetVersion,
    GetZludaInfo,
    SwapLanguages,
    
    // LLM Connection Commands
    CheckLMStudioConnection { base_url: String },
    CheckOllamaConnection,
    
    // Update Commands
    CheckForUpdates,
    DownloadUpdate { download_url: String },
}

/// Response structure for all command responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub endpoint: String,
    pub result: serde_json::Value,
}

impl Response {
    /// Create a success response
    pub fn success(endpoint: impl Into<String>, result: serde_json::Value) -> Self {
        Self {
            status: 200,
            endpoint: endpoint.into(),
            result,
        }
    }
    
    /// Create an error response
    pub fn error(endpoint: impl Into<String>, error_message: impl Into<String>) -> Self {
        Self {
            status: 500,
            endpoint: endpoint.into(),
            result: serde_json::json!({
                "error": error_message.into(),
            }),
        }
    }
    
    /// Create a not found response
    pub fn not_found(endpoint: impl Into<String>) -> Self {
        Self {
            status: 404,
            endpoint: endpoint.into(),
            result: serde_json::json!({
                "error": "Endpoint not found",
            }),
        }
    }
}

/// Transcription message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionMessage {
    pub original: MessageContent,
    pub translations: Vec<MessageContent>,
}

/// Message content with text and transliteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    pub message: String,
    pub transliteration: Vec<TransliterationSegment>,
}

/// Transliteration segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransliterationSegment {
    pub original: String,
    pub hiragana: Option<String>,
    pub romaji: Option<String>,
}
