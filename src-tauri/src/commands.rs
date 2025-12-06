// Tauri command handlers for frontend communication
// This module defines all Tauri command functions that bridge the frontend to the Rust backend

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
use tracing::{info, error, warn};

use crate::controller::Controller;
use crate::models::message::{Command, Response};

/// Thread-safe controller wrapper that implements Send + Sync
/// We use a Mutex here because cpal::Stream is not Send + Sync
pub struct ControllerWrapper {
    inner: Mutex<Option<Controller>>,
}

impl ControllerWrapper {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
    
    pub fn is_initialized(&self) -> bool {
        self.inner.lock().map(|guard| guard.is_some()).unwrap_or(false)
    }
}

// Implement Send + Sync for ControllerWrapper
// This is safe because we're using a Mutex to protect access
unsafe impl Send for ControllerWrapper {}
unsafe impl Sync for ControllerWrapper {}

/// Shared controller state type for Tauri
pub type ControllerState = Arc<ControllerWrapper>;

// ============================================================================
// Main Command Handler
// ============================================================================

/// Generic command handler that routes commands to the controller
/// This is the main entry point for all frontend commands
#[tauri::command]
pub async fn handle_command(
    command: Command,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    // We need to handle the command in a blocking context since Controller is not Send
    let controller_clone = controller.inner().clone();
    
    // Use tokio's spawn_blocking to run the command handler
    let result = tokio::task::spawn_blocking(move || {
        let guard = controller_clone.inner.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        match guard.as_ref() {
            Some(ctrl) => {
                // Create a new runtime for the blocking task
                let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Runtime error: {}", e))?;
                let response = rt.block_on(ctrl.handle_command(command));
                Ok::<Response, String>(response)
            }
            None => {
                error!("Controller not initialized");
                Ok::<Response, String>(Response::error("handle_command", "Controller not initialized"))
            }
        }
    }).await.map_err(|e| format!("Task error: {}", e))??;
    
    Ok(result)
}

// ============================================================================
// Main Window Commands
// ============================================================================

/// Enable translation feature
#[tauri::command]
pub async fn enable_translation(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableTranslation, controller).await
}

/// Disable translation feature
#[tauri::command]
pub async fn disable_translation(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::DisableTranslation, controller).await
}


/// Enable transcription send to VRChat
#[tauri::command]
pub async fn enable_transcription_send(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableTranscriptionSend, controller).await
}

/// Disable transcription send to VRChat
#[tauri::command]
pub async fn disable_transcription_send(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::DisableTranscriptionSend, controller).await
}

/// Enable transcription receive (speaker to log)
#[tauri::command]
pub async fn enable_transcription_receive(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableTranscriptionReceive, controller).await
}

/// Disable transcription receive (speaker to log)
#[tauri::command]
pub async fn disable_transcription_receive(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::DisableTranscriptionReceive, controller).await
}

/// Send a message from the message box
#[tauri::command]
pub async fn send_message_box(
    message: String,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SendMessageBox { message }, controller).await
}

// ============================================================================
// Audio Device Commands
// ============================================================================

/// Get list of available audio devices
#[tauri::command]
pub async fn get_audio_devices(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::GetAudioDevices, controller).await
}

/// Set microphone device
#[tauri::command]
pub async fn set_mic_device(
    host: String,
    device: String,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetMicDevice { host, device }, controller).await
}

/// Set speaker device
#[tauri::command]
pub async fn set_speaker_device(
    device: String,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetSpeakerDevice { device }, controller).await
}

/// Set microphone energy threshold
#[tauri::command]
pub async fn set_mic_threshold(
    threshold: u32,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetMicThreshold { threshold }, controller).await
}

/// Set speaker energy threshold
#[tauri::command]
pub async fn set_speaker_threshold(
    threshold: u32,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetSpeakerThreshold { threshold }, controller).await
}

/// Enable/disable automatic microphone threshold
#[tauri::command]
pub async fn enable_auto_mic_threshold(
    enabled: bool,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableAutoMicThreshold { enabled }, controller).await
}

/// Enable/disable automatic speaker threshold
#[tauri::command]
pub async fn enable_auto_speaker_threshold(
    enabled: bool,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableAutoSpeakerThreshold { enabled }, controller).await
}

// ============================================================================
// Transcription Commands
// ============================================================================

/// Start transcription
#[tauri::command]
pub async fn start_transcription(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::StartTranscription, controller).await
}

/// Stop transcription
#[tauri::command]
pub async fn stop_transcription(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::StopTranscription, controller).await
}

/// Set transcription engine
#[tauri::command]
pub async fn set_transcription_engine(
    engine: String,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetTranscriptionEngine { engine }, controller).await
}

/// Set Whisper model
#[tauri::command]
pub async fn set_whisper_model(
    model: String,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetWhisperModel { model }, controller).await
}

/// Download Whisper model
#[tauri::command]
pub async fn download_whisper_model(
    model: String,
    controller: State<'_, ControllerState>,
    app_handle: AppHandle,
) -> Result<Response, String> {
    // Clone model for use in response
    let model_for_response = model.clone();
    
    // Start download in background and emit progress events
    let _controller_clone = controller.inner().clone();
    
    tokio::spawn(async move {
        // Emit download started event
        let _ = app_handle.emit("download_progress", serde_json::json!({
            "model": &model,
            "progress": 0,
            "status": "started"
        }));
        
        // TODO: Implement actual download with progress reporting
        // For now, just emit completion
        let _ = app_handle.emit("download_progress", serde_json::json!({
            "model": &model,
            "progress": 100,
            "status": "completed"
        }));
    });
    
    Ok(Response::success("download_whisper_model", serde_json::json!({
        "model": model_for_response,
        "status": "download_started"
    })))
}


// ============================================================================
// Translation Commands
// ============================================================================

/// Translate text
#[tauri::command]
pub async fn translate_text(
    text: String,
    source: String,
    target: String,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::TranslateText { text, source, target }, controller).await
}

/// Set translation engine
#[tauri::command]
pub async fn set_translation_engine(
    engine: String,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetTranslationEngine { engine }, controller).await
}

/// Set source language
#[tauri::command]
pub async fn set_source_language(
    language: String,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetSourceLanguage { language }, controller).await
}

/// Set target language
#[tauri::command]
pub async fn set_target_language(
    language: String,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetTargetLanguage { language }, controller).await
}

// ============================================================================
// Configuration Commands
// ============================================================================

/// Get current configuration
#[tauri::command]
pub async fn get_config(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::GetConfig, controller).await
}

/// Set configuration
#[tauri::command]
pub async fn set_config(
    config: serde_json::Value,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetConfig { config }, controller).await
}

/// Set UI transparency
#[tauri::command]
pub async fn set_transparency(
    value: u8,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetTransparency { value }, controller).await
}

/// Set UI language
#[tauri::command]
pub async fn set_ui_language(
    language: String,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetUILanguage { language }, controller).await
}

/// Set UI scaling
#[tauri::command]
pub async fn set_ui_scaling(
    scaling: u8,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetUIScaling { scaling }, controller).await
}

// ============================================================================
// OSC Commands
// ============================================================================

/// Set OSC settings (IP and port)
#[tauri::command]
pub async fn set_osc_settings(
    ip: String,
    port: u16,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetOscSettings { ip, port }, controller).await
}

/// Enable/disable OSC message sending
#[tauri::command]
pub async fn enable_osc_send(
    enabled: bool,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableOscSend { enabled }, controller).await
}

/// Enable/disable VRC mic mute sync
#[tauri::command]
pub async fn enable_mic_mute_sync(
    enabled: bool,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableMicMuteSync { enabled }, controller).await
}

// ============================================================================
// WebSocket Commands
// ============================================================================

/// Set WebSocket server settings
#[tauri::command]
pub async fn set_websocket_settings(
    host: String,
    port: u16,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetWebSocketSettings { host, port }, controller).await
}

/// Enable/disable WebSocket server
#[tauri::command]
pub async fn enable_websocket_server(
    enabled: bool,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableWebSocketServer { enabled }, controller).await
}

// ============================================================================
// Overlay Commands
// ============================================================================

/// Enable/disable small log overlay
#[tauri::command]
pub async fn enable_small_log_overlay(
    enabled: bool,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableSmallLogOverlay { enabled }, controller).await
}

/// Enable/disable large log overlay
#[tauri::command]
pub async fn enable_large_log_overlay(
    enabled: bool,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableLargeLogOverlay { enabled }, controller).await
}

/// Set overlay settings
#[tauri::command]
pub async fn set_overlay_settings(
    settings: serde_json::Value,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetOverlaySettings { settings }, controller).await
}


// ============================================================================
// Word Filter Commands
// ============================================================================

/// Set word filter list
#[tauri::command]
pub async fn set_word_filter(
    words: Vec<String>,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SetWordFilter { words }, controller).await
}

// ============================================================================
// Logging Commands
// ============================================================================

/// Enable/disable logging
#[tauri::command]
pub async fn enable_logging(
    enabled: bool,
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableLogging { enabled }, controller).await
}

/// Open logs folder in file explorer
#[tauri::command]
pub async fn open_logs_folder(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::OpenLogsFolder, controller).await
}

// ============================================================================
// System Commands
// ============================================================================

/// Shutdown the application
#[tauri::command]
pub async fn shutdown(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::Shutdown, controller).await
}

/// Get application version
#[tauri::command]
pub async fn get_version(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::GetVersion, controller).await
}

// ============================================================================
// Update Commands
// ============================================================================

/// Check for software updates
#[tauri::command]
pub async fn check_for_updates() -> Result<Response, String> {
    use crate::utils::updater::Updater;
    
    info!("Checking for updates...");
    
    // Get current version from tauri.conf.json (hardcoded for now, should be from config)
    let current_version = env!("CARGO_PKG_VERSION");
    let updater = Updater::new(current_version);
    
    match updater.check_for_updates().await {
        Ok(update_info) => {
            info!("Update check completed: available={}", update_info.available);
            Ok(Response::success("check_for_updates", serde_json::json!(update_info)))
        }
        Err(e) => {
            error!("Update check failed: {}", e);
            Ok(Response::error("check_for_updates", format!("Failed to check for updates: {}", e)))
        }
    }
}

/// Download and install update
#[tauri::command]
pub async fn download_update(
    download_url: String,
    app_handle: AppHandle,
) -> Result<Response, String> {
    use crate::utils::updater::{Updater, DownloadProgress, DownloadStatus};
    
    info!("Starting update download from: {}", download_url);
    
    let current_version = env!("CARGO_PKG_VERSION");
    let updater = Updater::new(current_version);
    
    // Clone app_handle for use in the progress callback
    let app_handle_clone = app_handle.clone();
    
    // Download with progress reporting
    let result = updater.download_updater_with_progress(
        &download_url,
        move |progress: DownloadProgress| {
            // Emit progress event to frontend
            let _ = app_handle_clone.emit("update_download_progress", serde_json::json!({
                "downloaded_bytes": progress.downloaded_bytes,
                "total_bytes": progress.total_bytes,
                "percentage": progress.percentage,
                "status": match progress.status {
                    DownloadStatus::InProgress => "in_progress",
                    DownloadStatus::Completed => "completed",
                    DownloadStatus::Failed => "failed",
                }
            }));
        }
    ).await;
    
    match result {
        Ok(updater_path) => {
            info!("Update downloaded to: {}", updater_path.display());
            
            // Launch the updater
            match Updater::launch_updater(&updater_path) {
                Ok(()) => {
                    info!("Updater launched, application should exit");
                    
                    // Emit event to notify frontend that app will exit
                    let _ = app_handle.emit("update_launching", serde_json::json!({
                        "message": "Launching updater, application will exit..."
                    }));
                    
                    Ok(Response::success("download_update", serde_json::json!({
                        "status": "updater_launched",
                        "message": "Updater launched successfully. Application will exit."
                    })))
                }
                Err(e) => {
                    error!("Failed to launch updater: {}", e);
                    Ok(Response::error("download_update", format!("Failed to launch updater: {}", e)))
                }
            }
        }
        Err(e) => {
            error!("Update download failed: {}", e);
            
            // Emit failure event
            let _ = app_handle.emit("update_download_progress", serde_json::json!({
                "downloaded_bytes": 0,
                "total_bytes": 0,
                "percentage": 0,
                "status": "failed",
                "error": format!("{}", e)
            }));
            
            Ok(Response::error("download_update", format!("Failed to download update: {}", e)))
        }
    }
}

// ============================================================================
// Initialization Commands
// ============================================================================

/// Initialize the controller
/// This should be called once when the application starts
#[tauri::command]
pub async fn initialize_controller(
    controller: State<'_, ControllerState>,
    app_handle: AppHandle,
) -> Result<Response, String> {
    info!("Initializing controller from Tauri command");
    
    // Emit initialization started event
    let _ = app_handle.emit("init_progress", serde_json::json!({
        "progress": 0,
        "status": "starting",
        "message": "Initializing backend..."
    }));
    
    // Check if already initialized
    if controller.is_initialized() {
        warn!("Controller already initialized");
        return Ok(Response::success("initialize_controller", serde_json::json!({
            "status": "already_initialized"
        })));
    }
    
    // Emit progress
    let _ = app_handle.emit("init_progress", serde_json::json!({
        "progress": 20,
        "status": "creating_controller",
        "message": "Creating controller..."
    }));
    
    // Clone the controller state for use in spawn_blocking
    let controller_clone = controller.inner().clone();
    let app_handle_clone = app_handle.clone();
    
    // Use spawn_blocking since Controller is not Send
    let result = tokio::task::spawn_blocking(move || {
        // Create a new runtime for the blocking task
        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Runtime error: {}", e))?;
        
        rt.block_on(async {
            // Create controller
            match Controller::new().await {
                Ok(ctrl) => {
                    // Emit progress
                    let _ = app_handle_clone.emit("init_progress", serde_json::json!({
                        "progress": 50,
                        "status": "initializing_subsystems",
                        "message": "Initializing subsystems..."
                    }));
                    
                    // Initialize subsystems
                    if let Err(e) = ctrl.initialize().await {
                        error!("Failed to initialize subsystems: {}", e);
                        let _ = app_handle_clone.emit("init_progress", serde_json::json!({
                            "progress": 100,
                            "status": "error",
                            "message": format!("Initialization error: {}", e)
                        }));
                        return Ok::<Response, String>(Response::error("initialize_controller", format!("Failed to initialize: {}", e)));
                    }
                    
                    // Store the controller
                    {
                        let mut guard = controller_clone.inner.lock().map_err(|e| format!("Lock error: {}", e))?;
                        *guard = Some(ctrl);
                    }
                    
                    // Emit completion
                    let _ = app_handle_clone.emit("init_progress", serde_json::json!({
                        "progress": 100,
                        "status": "completed",
                        "message": "Initialization complete"
                    }));
                    
                    info!("Controller initialized successfully");
                    Ok(Response::success("initialize_controller", serde_json::json!({
                        "status": "initialized"
                    })))
                }
                Err(e) => {
                    error!("Failed to create controller: {}", e);
                    let _ = app_handle_clone.emit("init_progress", serde_json::json!({
                        "progress": 100,
                        "status": "error",
                        "message": format!("Failed to create controller: {}", e)
                    }));
                    Ok(Response::error("initialize_controller", format!("Failed to create controller: {}", e)))
                }
            }
        })
    }).await.map_err(|e| format!("Task error: {}", e))??;
    
    Ok(result)
}

/// Check if controller is initialized
#[tauri::command]
pub async fn is_controller_initialized(
    controller: State<'_, ControllerState>,
) -> Result<bool, String> {
    Ok(controller.is_initialized())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create the initial controller state (uninitialized)
pub fn create_controller_state() -> ControllerState {
    Arc::new(ControllerWrapper::new())
}

// ============================================================================
// Typing Indicator Commands
// ============================================================================

/// Start typing indicator
#[tauri::command]
pub async fn start_typing(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::StartTyping, controller).await
}

/// Stop typing indicator
#[tauri::command]
pub async fn stop_typing(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::StopTyping, controller).await
}

// ============================================================================
// Threshold Check Commands
// ============================================================================

/// Enable microphone threshold check mode
#[tauri::command]
pub async fn enable_mic_threshold_check(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableMicThresholdCheck, controller).await
}

/// Disable microphone threshold check mode
#[tauri::command]
pub async fn disable_mic_threshold_check(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::DisableMicThresholdCheck, controller).await
}

/// Enable speaker threshold check mode
#[tauri::command]
pub async fn enable_speaker_threshold_check(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::EnableSpeakerThresholdCheck, controller).await
}

/// Disable speaker threshold check mode
#[tauri::command]
pub async fn disable_speaker_threshold_check(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::DisableSpeakerThresholdCheck, controller).await
}

// ============================================================================
// Folder and Utility Commands
// ============================================================================

/// Open config folder in file explorer
#[tauri::command]
pub async fn open_config_folder(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::OpenConfigFolder, controller).await
}

/// Swap source and target languages
#[tauri::command]
pub async fn swap_languages(
    controller: State<'_, ControllerState>,
) -> Result<Response, String> {
    handle_command(Command::SwapLanguages, controller).await
}

/// Get ZLUDA installation information
#[tauri::command]
pub async fn get_zluda_info() -> Result<Response, String> {
    use crate::utils::zluda::{ZludaManager, detect_amd_gpus};
    
    info!("Getting ZLUDA info...");
    
    let mut manager = ZludaManager::new();
    let is_detected = manager.detect().unwrap_or(false);
    let is_initialized = manager.is_initialized();
    let zluda_path = manager.get_path().map(|p| p.to_string_lossy().to_string());
    let devices = manager.get_devices().to_vec();
    let amd_gpus = detect_amd_gpus().unwrap_or_default();
    
    Ok(Response::success("get_zluda_info", serde_json::json!({
        "detected": is_detected,
        "initialized": is_initialized,
        "path": zluda_path,
        "devices": devices,
        "amd_gpus": amd_gpus,
    })))
}

// ============================================================================
// LLM Connection Check Commands
// ============================================================================

/// Check LM Studio connection
#[tauri::command]
pub async fn check_lmstudio_connection(
    base_url: String,
) -> Result<Response, String> {
    use reqwest::Client;
    use std::time::Duration;
    
    info!("Checking LM Studio connection at: {}", base_url);
    
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let models_url = format!("{}/models", base_url);
    
    match client.get(&models_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("LM Studio connection successful");
                Ok(Response::success("check_lmstudio_connection", serde_json::json!({
                    "connected": true,
                    "base_url": base_url,
                })))
            } else {
                warn!("LM Studio connection failed with status: {}", response.status());
                Ok(Response::success("check_lmstudio_connection", serde_json::json!({
                    "connected": false,
                    "error": format!("HTTP status: {}", response.status()),
                })))
            }
        }
        Err(e) => {
            warn!("LM Studio connection failed: {}", e);
            Ok(Response::success("check_lmstudio_connection", serde_json::json!({
                "connected": false,
                "error": format!("{}", e),
            })))
        }
    }
}

/// Check Ollama connection
#[tauri::command]
pub async fn check_ollama_connection() -> Result<Response, String> {
    use reqwest::Client;
    use std::time::Duration;
    
    let base_url = "http://localhost:11434";
    info!("Checking Ollama connection at: {}", base_url);
    
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    match client.get(base_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("Ollama connection successful");
                Ok(Response::success("check_ollama_connection", serde_json::json!({
                    "connected": true,
                })))
            } else {
                warn!("Ollama connection failed with status: {}", response.status());
                Ok(Response::success("check_ollama_connection", serde_json::json!({
                    "connected": false,
                    "error": format!("HTTP status: {}", response.status()),
                })))
            }
        }
        Err(e) => {
            warn!("Ollama connection failed: {}", e);
            Ok(Response::success("check_ollama_connection", serde_json::json!({
                "connected": false,
                "error": format!("{}", e),
            })))
        }
    }
}

// ============================================================================
// Profiling Commands
// ============================================================================

/// Get profiling report with memory, CPU, and latency statistics
#[tauri::command]
pub async fn get_profiling_report() -> Result<serde_json::Value, String> {
    use crate::utils::profiling::global_profiler;
    
    let report = global_profiler().generate_report();
    
    // Convert to JSON-serializable format
    let memory = serde_json::json!({
        "heap_allocated_kb": report.memory.heap_allocated / 1024,
        "peak_heap_allocated_kb": report.memory.peak_heap_allocated / 1024,
        "active_allocations": report.memory.active_allocations,
    });
    
    let cpu: serde_json::Map<String, serde_json::Value> = report.cpu.iter().map(|(k, v)| {
        (k.clone(), serde_json::json!({
            "total_time_ms": v.total_time.as_millis(),
            "call_count": v.call_count,
            "avg_time_us": v.avg_time.as_micros(),
            "min_time_us": if v.min_time == std::time::Duration::MAX { 0 } else { v.min_time.as_micros() },
            "max_time_us": v.max_time.as_micros(),
        }))
    }).collect();
    
    let latency: serde_json::Map<String, serde_json::Value> = report.latency.iter().map(|(k, v)| {
        (k.clone(), serde_json::json!({
            "sample_count": v.sample_count,
            "avg_latency_us": v.avg_latency.as_micros(),
            "p50_latency_us": v.p50_latency.as_micros(),
            "p95_latency_us": v.p95_latency.as_micros(),
            "p99_latency_us": v.p99_latency.as_micros(),
            "min_latency_us": if v.min_latency == std::time::Duration::MAX { 0 } else { v.min_latency.as_micros() },
            "max_latency_us": v.max_latency.as_micros(),
        }))
    }).collect();
    
    Ok(serde_json::json!({
        "memory": memory,
        "cpu": cpu,
        "latency": latency,
        "formatted_report": report.format(),
    }))
}

/// Run performance benchmarks
#[tauri::command]
pub async fn run_benchmarks(iterations: Option<usize>) -> Result<serde_json::Value, String> {
    use crate::utils::benchmarks::{BenchmarkConfig, run_all_benchmarks, format_benchmark_report};
    
    let config = BenchmarkConfig {
        iterations: iterations.unwrap_or(50),
        warmup_iterations: 5,
        verbose: false,
    };
    
    // Run benchmarks in a blocking task to avoid blocking the async runtime
    let results = tokio::task::spawn_blocking(move || {
        run_all_benchmarks(&config)
    }).await.map_err(|e| format!("Benchmark task error: {}", e))?;
    
    // Convert results to JSON
    let benchmark_results: Vec<serde_json::Value> = results.iter().map(|r| {
        serde_json::json!({
            "name": r.name,
            "iterations": r.iterations,
            "total_time_ms": r.total_time.as_millis(),
            "avg_time_us": r.avg_time.as_micros(),
            "min_time_us": r.min_time.as_micros(),
            "max_time_us": r.max_time.as_micros(),
            "throughput_ops_per_sec": r.throughput,
        })
    }).collect();
    
    Ok(serde_json::json!({
        "results": benchmark_results,
        "formatted_report": format_benchmark_report(&results),
    }))
}

/// Reset profiling statistics
#[tauri::command]
pub async fn reset_profiling() -> Result<(), String> {
    use crate::utils::profiling::global_profiler;
    global_profiler().reset();
    Ok(())
}

/// Enable or disable profiling
#[tauri::command]
pub async fn set_profiling_enabled(enabled: bool) -> Result<(), String> {
    use crate::utils::profiling::global_profiler;
    global_profiler().set_enabled(enabled);
    Ok(())
}

/// Get all command handlers for registration with Tauri
/// Returns a tuple of command name and handler function
pub fn get_all_commands() -> Vec<&'static str> {
    vec![
        "handle_command",
        "enable_translation",
        "disable_translation",
        "enable_transcription_send",
        "disable_transcription_send",
        "send_message_box",
        "get_audio_devices",
        "set_mic_device",
        "set_speaker_device",
        "set_mic_threshold",
        "set_speaker_threshold",
        "enable_auto_mic_threshold",
        "enable_auto_speaker_threshold",
        "start_transcription",
        "stop_transcription",
        "set_transcription_engine",
        "set_whisper_model",
        "download_whisper_model",
        "translate_text",
        "set_translation_engine",
        "set_source_language",
        "set_target_language",
        "get_config",
        "set_config",
        "set_transparency",
        "set_ui_language",
        "set_ui_scaling",
        "set_osc_settings",
        "enable_osc_send",
        "enable_mic_mute_sync",
        "set_websocket_settings",
        "enable_websocket_server",
        "enable_small_log_overlay",
        "enable_large_log_overlay",
        "set_overlay_settings",
        "set_word_filter",
        "enable_logging",
        "open_logs_folder",
        "shutdown",
        "get_version",
        "initialize_controller",
        "is_controller_initialized",
        "check_for_updates",
        "download_update",
        "get_profiling_report",
        "run_benchmarks",
        "reset_profiling",
        "set_profiling_enabled",
    ]
}
