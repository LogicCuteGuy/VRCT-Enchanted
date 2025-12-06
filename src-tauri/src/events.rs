// Tauri event emission module
// This module provides utilities for emitting events to the frontend

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tracing::{info, error, debug};

// ============================================================================
// Event Types
// ============================================================================

/// Event names used for communication with the frontend
pub mod event_names {
    /// Initialization progress event
    pub const INIT_PROGRESS: &str = "init_progress";
    
    /// Transcription result event
    pub const TRANSCRIPTION: &str = "transcription";
    
    /// Translation result event
    pub const TRANSLATION: &str = "translation";
    
    /// Error event
    pub const ERROR: &str = "error";
    
    /// Download progress event
    pub const DOWNLOAD_PROGRESS: &str = "download_progress";
    
    /// Audio energy level event
    pub const ENERGY_LEVEL: &str = "energy_level";
    
    /// Device change event
    pub const DEVICE_CHANGE: &str = "device_change";
    
    /// Message blocked event (word filter)
    pub const MESSAGE_BLOCKED: &str = "message_blocked";
    
    /// Translation error event (for fallback notification)
    pub const TRANSLATION_ERROR: &str = "translation_error";
    
    /// WebSocket client event
    pub const WEBSOCKET_CLIENT: &str = "websocket_client";
    
    /// Overlay update event
    pub const OVERLAY_UPDATE: &str = "overlay_update";
    
    /// Configuration change event
    pub const CONFIG_CHANGE: &str = "config_change";
    
    /// VRC mic mute sync event
    pub const MIC_MUTE_SYNC: &str = "mic_mute_sync";
}

// ============================================================================
// Event Payloads
// ============================================================================

/// Initialization progress payload
#[derive(Debug, Clone, Serialize)]
pub struct InitProgressPayload {
    pub progress: u8,
    pub status: String,
    pub message: String,
}

/// Transcription event payload
#[derive(Debug, Clone, Serialize)]
pub struct TranscriptionPayload {
    pub original: String,
    pub language: String,
    pub confidence: f32,
    pub translations: Vec<TranslationItem>,
}

/// Translation item within transcription
#[derive(Debug, Clone, Serialize)]
pub struct TranslationItem {
    pub language: String,
    pub text: String,
}

/// Translation event payload
#[derive(Debug, Clone, Serialize)]
pub struct TranslationPayload {
    pub original: String,
    pub translation: String,
    pub source_language: String,
    pub target_language: String,
    pub engine: String,
}

/// Error event payload
#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    pub error_type: String,
    pub message: String,
    pub context: Option<String>,
}

/// Download progress payload
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgressPayload {
    pub model: String,
    pub progress: u8,
    pub status: String,
    pub bytes_downloaded: Option<u64>,
    pub total_bytes: Option<u64>,
}

/// Energy level payload
#[derive(Debug, Clone, Serialize)]
pub struct EnergyLevelPayload {
    pub device_type: String, // "mic" or "speaker"
    pub energy: f32,
    pub threshold: f32,
    pub is_above_threshold: bool,
}

/// Device change payload
#[derive(Debug, Clone, Serialize)]
pub struct DeviceChangePayload {
    pub event_type: String, // "added", "removed", "default_changed"
    pub device_name: String,
    pub device_type: String, // "input" or "output"
}

/// Message blocked payload
#[derive(Debug, Clone, Serialize)]
pub struct MessageBlockedPayload {
    pub reason: String,
    pub text: String,
    pub matched_word: Option<String>,
}

/// Translation error payload (for fallback notification)
#[derive(Debug, Clone, Serialize)]
pub struct TranslationErrorPayload {
    pub engine: String,
    pub error: String,
    pub fallback: Option<String>,
}

/// WebSocket client event payload
#[derive(Debug, Clone, Serialize)]
pub struct WebSocketClientPayload {
    pub event_type: String, // "connected", "disconnected"
    pub client_count: usize,
}

/// Configuration change payload
#[derive(Debug, Clone, Serialize)]
pub struct ConfigChangePayload {
    pub key: String,
    pub value: serde_json::Value,
}

/// Mic mute sync payload
#[derive(Debug, Clone, Serialize)]
pub struct MicMuteSyncPayload {
    pub is_muted: bool,
    pub transcription_paused: bool,
}

// ============================================================================
// Event Emitter
// ============================================================================

/// Event emitter for sending events to the frontend
pub struct EventEmitter {
    app_handle: AppHandle,
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
    
    /// Emit a generic event with a payload
    pub fn emit<T: Serialize + Clone>(&self, event: &str, payload: T) -> Result<(), String> {
        self.app_handle
            .emit(event, payload)
            .map_err(|e| format!("Failed to emit event '{}': {}", event, e))
    }
    
    /// Emit initialization progress
    pub fn emit_init_progress(&self, progress: u8, status: &str, message: &str) -> Result<(), String> {
        debug!("Emitting init progress: {}% - {} - {}", progress, status, message);
        self.emit(event_names::INIT_PROGRESS, InitProgressPayload {
            progress,
            status: status.to_string(),
            message: message.to_string(),
        })
    }
    
    /// Emit transcription result
    pub fn emit_transcription(&self, original: &str, language: &str, confidence: f32, translations: Vec<TranslationItem>) -> Result<(), String> {
        info!("Emitting transcription: {} ({})", original, language);
        self.emit(event_names::TRANSCRIPTION, TranscriptionPayload {
            original: original.to_string(),
            language: language.to_string(),
            confidence,
            translations,
        })
    }
    
    /// Emit translation result
    pub fn emit_translation(&self, original: &str, translation: &str, source: &str, target: &str, engine: &str) -> Result<(), String> {
        info!("Emitting translation: {} -> {}", source, target);
        self.emit(event_names::TRANSLATION, TranslationPayload {
            original: original.to_string(),
            translation: translation.to_string(),
            source_language: source.to_string(),
            target_language: target.to_string(),
            engine: engine.to_string(),
        })
    }
    
    /// Emit error event
    pub fn emit_error(&self, error_type: &str, message: &str, context: Option<&str>) -> Result<(), String> {
        error!("Emitting error: {} - {}", error_type, message);
        self.emit(event_names::ERROR, ErrorPayload {
            error_type: error_type.to_string(),
            message: message.to_string(),
            context: context.map(|s| s.to_string()),
        })
    }
    
    /// Emit download progress
    pub fn emit_download_progress(&self, model: &str, progress: u8, status: &str, bytes_downloaded: Option<u64>, total_bytes: Option<u64>) -> Result<(), String> {
        debug!("Emitting download progress: {} - {}%", model, progress);
        self.emit(event_names::DOWNLOAD_PROGRESS, DownloadProgressPayload {
            model: model.to_string(),
            progress,
            status: status.to_string(),
            bytes_downloaded,
            total_bytes,
        })
    }
    
    /// Emit energy level update
    pub fn emit_energy_level(&self, device_type: &str, energy: f32, threshold: f32) -> Result<(), String> {
        self.emit(event_names::ENERGY_LEVEL, EnergyLevelPayload {
            device_type: device_type.to_string(),
            energy,
            threshold,
            is_above_threshold: energy > threshold,
        })
    }
    
    /// Emit device change event
    pub fn emit_device_change(&self, event_type: &str, device_name: &str, device_type: &str) -> Result<(), String> {
        info!("Emitting device change: {} - {} ({})", event_type, device_name, device_type);
        self.emit(event_names::DEVICE_CHANGE, DeviceChangePayload {
            event_type: event_type.to_string(),
            device_name: device_name.to_string(),
            device_type: device_type.to_string(),
        })
    }
    
    /// Emit message blocked event
    pub fn emit_message_blocked(&self, reason: &str, text: &str, matched_word: Option<&str>) -> Result<(), String> {
        info!("Emitting message blocked: {} - {}", reason, text);
        self.emit(event_names::MESSAGE_BLOCKED, MessageBlockedPayload {
            reason: reason.to_string(),
            text: text.to_string(),
            matched_word: matched_word.map(|s| s.to_string()),
        })
    }
    
    /// Emit translation error event
    pub fn emit_translation_error(&self, engine: &str, error: &str, fallback: Option<&str>) -> Result<(), String> {
        error!("Emitting translation error: {} - {}", engine, error);
        self.emit(event_names::TRANSLATION_ERROR, TranslationErrorPayload {
            engine: engine.to_string(),
            error: error.to_string(),
            fallback: fallback.map(|s| s.to_string()),
        })
    }
    
    /// Emit WebSocket client event
    pub fn emit_websocket_client(&self, event_type: &str, client_count: usize) -> Result<(), String> {
        debug!("Emitting WebSocket client event: {} - {} clients", event_type, client_count);
        self.emit(event_names::WEBSOCKET_CLIENT, WebSocketClientPayload {
            event_type: event_type.to_string(),
            client_count,
        })
    }
    
    /// Emit configuration change event
    pub fn emit_config_change(&self, key: &str, value: serde_json::Value) -> Result<(), String> {
        debug!("Emitting config change: {}", key);
        self.emit(event_names::CONFIG_CHANGE, ConfigChangePayload {
            key: key.to_string(),
            value,
        })
    }
    
    /// Emit mic mute sync event
    pub fn emit_mic_mute_sync(&self, is_muted: bool, transcription_paused: bool) -> Result<(), String> {
        info!("Emitting mic mute sync: muted={}, paused={}", is_muted, transcription_paused);
        self.emit(event_names::MIC_MUTE_SYNC, MicMuteSyncPayload {
            is_muted,
            transcription_paused,
        })
    }
}

// ============================================================================
// Global Event Emitter (for use without AppHandle)
// ============================================================================

use std::sync::OnceLock;
use tokio::sync::RwLock;

static GLOBAL_APP_HANDLE: OnceLock<RwLock<Option<AppHandle>>> = OnceLock::new();

/// Set the global app handle for event emission
pub fn set_global_app_handle(app_handle: AppHandle) {
    let lock = GLOBAL_APP_HANDLE.get_or_init(|| RwLock::new(None));
    // Use blocking write since this is called during setup
    if let Ok(mut guard) = lock.try_write() {
        *guard = Some(app_handle);
    }
}

/// Get the global event emitter
pub async fn get_global_emitter() -> Option<EventEmitter> {
    let lock = GLOBAL_APP_HANDLE.get()?;
    let guard = lock.read().await;
    guard.as_ref().map(|h| EventEmitter::new(h.clone()))
}

/// Emit an event using the global app handle
pub async fn emit_global<T: Serialize + Clone>(event: &str, payload: T) -> Result<(), String> {
    match get_global_emitter().await {
        Some(emitter) => emitter.emit(event, payload),
        None => Err("Global app handle not set".to_string()),
    }
}
