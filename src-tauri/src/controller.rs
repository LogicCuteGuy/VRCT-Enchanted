// Controller layer for orchestrating all subsystems
// This is the main coordination layer that handles UI requests and manages application state

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};

use crate::config::Config;
use crate::audio::device_manager::DeviceManager;
use crate::transcription::whisper::WhisperTranscriber;
use crate::translation::TranslationManager;
use crate::communication::osc::OscHandler;
use crate::communication::websocket::WebSocketServer;
use crate::overlay::OverlayManager;
use crate::utils::error::Result;
use crate::utils::keyword_filter::KeywordFilter;
use crate::utils::message_formatter::MessageFormatter;
use crate::audio::recorder::AudioRecorder;

/// Application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub transcription_enabled: bool,
    pub translation_enabled: bool,
    pub transcription_send_enabled: bool,
    pub transcription_receive_enabled: bool,
    pub is_typing: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            transcription_enabled: false,
            translation_enabled: false,
            transcription_send_enabled: false,
            transcription_receive_enabled: false,
            is_typing: false,
        }
    }
}

/// Main controller that orchestrates all subsystems
pub struct Controller {
    config: Arc<Config>,
    device_manager: Arc<DeviceManager>,
    #[allow(dead_code)] // Placeholder for full transcription pipeline
    transcriber: Arc<RwLock<Option<WhisperTranscriber>>>,
    translator: Arc<TranslationManager>,
    osc_handler: Arc<RwLock<Option<OscHandler>>>,
    websocket_server: Arc<RwLock<WebSocketServer>>,
    overlay_manager: Arc<RwLock<Option<OverlayManager>>>,
    keyword_filter: Arc<RwLock<KeywordFilter>>,
    #[allow(dead_code)] // Placeholder for full transcription pipeline
    message_formatter: Arc<RwLock<MessageFormatter>>,
    audio_recorder: Arc<RwLock<Option<AudioRecorder>>>,
    state: Arc<RwLock<AppState>>,
}

impl Controller {
    /// Create a new controller instance
    pub async fn new() -> Result<Self> {
        info!("Initializing controller");

        // Load configuration
        let config = Arc::new(Config::new(None).await?);
        
        // Initialize device manager
        let device_manager = Arc::new(DeviceManager::new());
        
        // Initialize translation manager
        let translator = Arc::new(TranslationManager::new());
        
        // Initialize WebSocket server (but don't start it yet)
        let websocket_server = Arc::new(RwLock::new(WebSocketServer::new()));
        
        // Initialize keyword filter
        let keyword_filter = Arc::new(RwLock::new(KeywordFilter::new()));
        
        // Initialize message formatter with default values (will be updated from config)
        let default_format = crate::config::types::MessageFormatParts {
            message: crate::config::types::MessagePart {
                prefix: String::new(),
                suffix: String::new(),
            },
            separator: " ".to_string(),
            translation: crate::config::types::TranslationPart {
                prefix: String::new(),
                separator: " ".to_string(),
                suffix: String::new(),
            },
            translation_first: false,
        };
        let message_formatter = Arc::new(RwLock::new(MessageFormatter::new(
            default_format.clone(),
            default_format,
            false,
        )));
        
        // Other subsystems will be initialized lazily or on demand
        let transcriber = Arc::new(RwLock::new(None));
        let osc_handler = Arc::new(RwLock::new(None));
        let overlay_manager = Arc::new(RwLock::new(None));
        let audio_recorder = Arc::new(RwLock::new(None));
        
        let state = Arc::new(RwLock::new(AppState::default()));

        Ok(Self {
            config,
            device_manager,
            transcriber,
            translator,
            osc_handler,
            websocket_server,
            overlay_manager,
            keyword_filter,
            message_formatter,
            audio_recorder,
            state,
        })
    }

    /// Initialize all subsystems based on configuration
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing subsystems");

        // Load keyword filter from config
        let filter_words = self.config.get(|data| data.mic_word_filter.clone()).await;
        if !filter_words.is_empty() {
            let filter = self.keyword_filter.read().await;
            filter.load_filters(filter_words);
        }

        // Initialize OSC handler if enabled
        let send_to_vrc = self.config.get(|data| data.send_message_to_vrc).await;
        if send_to_vrc {
            let (ip, port) = self.config.get_osc_settings().await;
            match OscHandler::new(&ip, port) {
                Ok(handler) => {
                    *self.osc_handler.write().await = Some(handler);
                    info!("OSC handler initialized");
                }
                Err(e) => {
                    error!("Failed to initialize OSC handler: {}", e);
                }
            }
        }

        // Initialize WebSocket server if enabled
        let ws_enabled = self.config.get(|data| data.websocket_server).await;
        if ws_enabled {
            let (host, port) = self.config.get(|data| {
                (data.websocket_host.clone(), data.websocket_port)
            }).await;
            let addr_str = format!("{}:{}", host, port);
            if let Ok(addr) = addr_str.parse() {
                let server = self.websocket_server.write().await;
                if let Err(e) = server.start(addr).await {
                    error!("Failed to start WebSocket server: {}", e);
                } else {
                    info!("WebSocket server started on {}", addr_str);
                }
            } else {
                error!("Invalid WebSocket address: {}", addr_str);
            }
        }

        // Initialize overlay manager if overlays are enabled
        let small_log = self.config.get(|data| data.overlay_small_log).await;
        if small_log {
            match OverlayManager::new() {
                Ok(manager) => {
                    *self.overlay_manager.write().await = Some(manager);
                    info!("Overlay manager initialized");
                }
                Err(e) => {
                    warn!("Failed to initialize overlay manager: {}", e);
                }
            }
        }

        info!("Subsystems initialized");
        Ok(())
    }

    /// Gracefully shutdown all subsystems
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down controller");

        // Stop transcription if running
        if let Some(mut recorder) = self.audio_recorder.write().await.take() {
            if let Err(e) = recorder.stop() {
                error!("Error stopping audio recorder: {}", e);
            }
        }

        // Stop WebSocket server
        let server = self.websocket_server.write().await;
        server.stop().await;

        // Save configuration
        if let Err(e) = self.config.save().await {
            error!("Error saving configuration: {}", e);
        }

        info!("Controller shutdown complete");
        Ok(())
    }

    /// Get a reference to the config
    pub fn config(&self) -> &Arc<Config> {
        &self.config
    }

    /// Get a reference to the device manager
    pub fn device_manager(&self) -> &Arc<DeviceManager> {
        &self.device_manager
    }

    /// Get a reference to the translation manager
    pub fn translator(&self) -> &Arc<TranslationManager> {
        &self.translator
    }

    /// Get a reference to the application state
    pub fn state(&self) -> &Arc<RwLock<AppState>> {
        &self.state
    }
}

// Command handling implementation
use crate::models::message::{Command, Response};

impl Controller {
    /// Handle a command from the frontend
    /// Routes commands to appropriate subsystems and returns a response
    pub async fn handle_command(&self, cmd: Command) -> Response {
        match cmd {
            // Main Window Commands
            Command::EnableTranslation => self.handle_enable_translation().await,
            Command::DisableTranslation => self.handle_disable_translation().await,
            Command::EnableTranscriptionSend => self.handle_enable_transcription_send().await,
            Command::DisableTranscriptionSend => self.handle_disable_transcription_send().await,
            Command::EnableTranscriptionReceive => self.handle_enable_transcription_receive().await,
            Command::DisableTranscriptionReceive => self.handle_disable_transcription_receive().await,
            Command::SendMessageBox { message } => self.handle_send_message_box(message).await,
            Command::StartTyping => self.handle_start_typing().await,
            Command::StopTyping => self.handle_stop_typing().await,
            
            // Audio Device Commands
            Command::GetAudioDevices => self.handle_get_audio_devices().await,
            Command::SetMicDevice { host, device } => self.handle_set_mic_device(host, device).await,
            Command::SetSpeakerDevice { device } => self.handle_set_speaker_device(device).await,
            Command::SetMicThreshold { threshold } => self.handle_set_mic_threshold(threshold).await,
            Command::SetSpeakerThreshold { threshold } => self.handle_set_speaker_threshold(threshold).await,
            Command::EnableAutoMicThreshold { enabled } => self.handle_enable_auto_mic_threshold(enabled).await,
            Command::EnableAutoSpeakerThreshold { enabled } => self.handle_enable_auto_speaker_threshold(enabled).await,
            Command::EnableMicThresholdCheck => self.handle_enable_mic_threshold_check().await,
            Command::DisableMicThresholdCheck => self.handle_disable_mic_threshold_check().await,
            Command::EnableSpeakerThresholdCheck => self.handle_enable_speaker_threshold_check().await,
            Command::DisableSpeakerThresholdCheck => self.handle_disable_speaker_threshold_check().await,
            
            // Transcription Commands
            Command::StartTranscription => self.handle_start_transcription().await,
            Command::StopTranscription => self.handle_stop_transcription().await,
            Command::SetTranscriptionEngine { engine } => self.handle_set_transcription_engine(engine).await,
            Command::SetWhisperModel { model } => self.handle_set_whisper_model(model).await,
            Command::DownloadWhisperModel { model } => self.handle_download_whisper_model(model).await,
            
            // Translation Commands
            Command::TranslateText { text, source, target } => self.handle_translate_text(text, source, target).await,
            Command::SetTranslationEngine { engine } => self.handle_set_translation_engine(engine).await,
            Command::SetSourceLanguage { language } => self.handle_set_source_language(language).await,
            Command::SetTargetLanguage { language } => self.handle_set_target_language(language).await,
            
            // Configuration Commands
            Command::GetConfig => self.handle_get_config().await,
            Command::SetConfig { config } => self.handle_set_config(config).await,
            Command::SetTransparency { value } => self.handle_set_transparency(value).await,
            Command::SetUILanguage { language } => self.handle_set_ui_language(language).await,
            Command::SetUIScaling { scaling } => self.handle_set_ui_scaling(scaling).await,
            
            // OSC Commands
            Command::SetOscSettings { ip, port } => self.handle_set_osc_settings(ip, port).await,
            Command::EnableOscSend { enabled } => self.handle_enable_osc_send(enabled).await,
            Command::EnableMicMuteSync { enabled } => self.handle_enable_mic_mute_sync(enabled).await,
            
            // WebSocket Commands
            Command::SetWebSocketSettings { host, port } => self.handle_set_websocket_settings(host, port).await,
            Command::EnableWebSocketServer { enabled } => self.handle_enable_websocket_server(enabled).await,
            
            // Overlay Commands
            Command::EnableSmallLogOverlay { enabled } => self.handle_enable_small_log_overlay(enabled).await,
            Command::EnableLargeLogOverlay { enabled } => self.handle_enable_large_log_overlay(enabled).await,
            Command::SetOverlaySettings { settings } => self.handle_set_overlay_settings(settings).await,
            
            // Word Filter Commands
            Command::SetWordFilter { words } => self.handle_set_word_filter(words).await,
            
            // Logging Commands
            Command::EnableLogging { enabled } => self.handle_enable_logging(enabled).await,
            Command::OpenLogsFolder => self.handle_open_logs_folder().await,
            Command::OpenConfigFolder => self.handle_open_config_folder().await,
            
            // System Commands
            Command::Shutdown => self.handle_shutdown().await,
            Command::GetVersion => self.handle_get_version().await,
            Command::SwapLanguages => self.handle_swap_languages().await,
            
            // Commands handled directly in commands.rs, not through controller
            Command::GetZludaInfo => {
                Response::error("get_zluda_info", "ZLUDA commands should be called directly, not through controller")
            },
            Command::CheckLMStudioConnection { .. } => {
                Response::error("check_lmstudio_connection", "LLM connection commands should be called directly, not through controller")
            },
            Command::CheckOllamaConnection => {
                Response::error("check_ollama_connection", "LLM connection commands should be called directly, not through controller")
            },
            
            // Update Commands - handled directly in commands.rs, not through controller
            Command::CheckForUpdates => {
                Response::error("check_for_updates", "Update commands should be called directly, not through controller")
            },
            Command::DownloadUpdate { .. } => {
                Response::error("download_update", "Update commands should be called directly, not through controller")
            },
        }
    }

    // Main Window Command Handlers
    async fn handle_enable_translation(&self) -> Response {
        let mut state = self.state.write().await;
        state.translation_enabled = true;
        info!("Translation enabled");
        Response::success("enable_translation", serde_json::json!({"enabled": true}))
    }

    async fn handle_disable_translation(&self) -> Response {
        let mut state = self.state.write().await;
        state.translation_enabled = false;
        info!("Translation disabled");
        Response::success("disable_translation", serde_json::json!({"enabled": false}))
    }

    async fn handle_enable_transcription_send(&self) -> Response {
        let mut state = self.state.write().await;
        state.transcription_send_enabled = true;
        info!("Transcription send enabled");
        Response::success("enable_transcription_send", serde_json::json!({"enabled": true}))
    }

    async fn handle_disable_transcription_send(&self) -> Response {
        let mut state = self.state.write().await;
        state.transcription_send_enabled = false;
        info!("Transcription send disabled");
        Response::success("disable_transcription_send", serde_json::json!({"enabled": false}))
    }

    async fn handle_enable_transcription_receive(&self) -> Response {
        let mut state = self.state.write().await;
        state.transcription_receive_enabled = true;
        info!("Transcription receive enabled");
        Response::success("enable_transcription_receive", serde_json::json!({"enabled": true}))
    }

    async fn handle_disable_transcription_receive(&self) -> Response {
        let mut state = self.state.write().await;
        state.transcription_receive_enabled = false;
        info!("Transcription receive disabled");
        Response::success("disable_transcription_receive", serde_json::json!({"enabled": false}))
    }

    async fn handle_send_message_box(&self, message: String) -> Response {
        info!("Sending message box: {}", message);
        // This will be implemented in the OSC integration section
        Response::success("send_message_box", serde_json::json!({"sent": true}))
    }

    // Audio Device Command Handlers
    async fn handle_get_audio_devices(&self) -> Response {
        match (
            self.device_manager.enumerate_input_devices(),
            self.device_manager.enumerate_output_devices()
        ) {
            (Ok(input_devices), Ok(output_devices)) => {
                Response::success("get_audio_devices", serde_json::json!({
                    "input_devices": input_devices,
                    "output_devices": output_devices,
                }))
            }
            (Err(e), _) | (_, Err(e)) => {
                error!("Failed to enumerate audio devices: {}", e);
                Response::error("get_audio_devices", format!("Failed to enumerate devices: {}", e))
            }
        }
    }

    async fn handle_set_mic_device(&self, host: String, device: String) -> Response {
        match self.config.set_mic_device(host.clone(), device.clone()).await {
            Ok(_) => {
                info!("Microphone device set to: {} - {}", host, device);
                Response::success("set_mic_device", serde_json::json!({"host": host, "device": device}))
            }
            Err(e) => {
                error!("Failed to set microphone device: {}", e);
                Response::error("set_mic_device", format!("Failed to set device: {}", e))
            }
        }
    }

    async fn handle_set_speaker_device(&self, device: String) -> Response {
        match self.config.set_speaker_device(device.clone()).await {
            Ok(_) => {
                info!("Speaker device set to: {}", device);
                Response::success("set_speaker_device", serde_json::json!({"device": device}))
            }
            Err(e) => {
                error!("Failed to set speaker device: {}", e);
                Response::error("set_speaker_device", format!("Failed to set device: {}", e))
            }
        }
    }

    async fn handle_set_mic_threshold(&self, threshold: u32) -> Response {
        match self.config.set(|data| data.mic_threshold = threshold).await {
            Ok(_) => {
                info!("Microphone threshold set to: {}", threshold);
                Response::success("set_mic_threshold", serde_json::json!({"threshold": threshold}))
            }
            Err(e) => {
                error!("Failed to set microphone threshold: {}", e);
                Response::error("set_mic_threshold", format!("Failed to set threshold: {}", e))
            }
        }
    }

    async fn handle_set_speaker_threshold(&self, threshold: u32) -> Response {
        match self.config.set(|data| data.speaker_threshold = threshold).await {
            Ok(_) => {
                info!("Speaker threshold set to: {}", threshold);
                Response::success("set_speaker_threshold", serde_json::json!({"threshold": threshold}))
            }
            Err(e) => {
                error!("Failed to set speaker threshold: {}", e);
                Response::error("set_speaker_threshold", format!("Failed to set threshold: {}", e))
            }
        }
    }

    async fn handle_enable_auto_mic_threshold(&self, enabled: bool) -> Response {
        match self.config.set(|data| data.mic_automatic_threshold = enabled).await {
            Ok(_) => {
                info!("Auto microphone threshold set to: {}", enabled);
                Response::success("enable_auto_mic_threshold", serde_json::json!({"enabled": enabled}))
            }
            Err(e) => {
                error!("Failed to set auto microphone threshold: {}", e);
                Response::error("enable_auto_mic_threshold", format!("Failed to set: {}", e))
            }
        }
    }

    async fn handle_enable_auto_speaker_threshold(&self, enabled: bool) -> Response {
        match self.config.set(|data| data.speaker_automatic_threshold = enabled).await {
            Ok(_) => {
                info!("Auto speaker threshold set to: {}", enabled);
                Response::success("enable_auto_speaker_threshold", serde_json::json!({"enabled": enabled}))
            }
            Err(e) => {
                error!("Failed to set auto speaker threshold: {}", e);
                Response::error("enable_auto_speaker_threshold", format!("Failed to set: {}", e))
            }
        }
    }

    // Transcription Command Handlers (stubs for now)
    async fn handle_start_transcription(&self) -> Response {
        info!("Start transcription command received");
        Response::success("start_transcription", serde_json::json!({"started": true}))
    }

    async fn handle_stop_transcription(&self) -> Response {
        info!("Stop transcription command received");
        Response::success("stop_transcription", serde_json::json!({"stopped": true}))
    }

    async fn handle_set_transcription_engine(&self, engine: String) -> Response {
        match self.config.set(|data| data.selected_transcription_engine = engine.clone()).await {
            Ok(_) => {
                info!("Transcription engine set to: {}", engine);
                Response::success("set_transcription_engine", serde_json::json!({"engine": engine}))
            }
            Err(e) => {
                error!("Failed to set transcription engine: {}", e);
                Response::error("set_transcription_engine", format!("Failed to set engine: {}", e))
            }
        }
    }

    async fn handle_set_whisper_model(&self, model: String) -> Response {
        match self.config.set(|data| data.whisper_weight_type = model.clone()).await {
            Ok(_) => {
                info!("Whisper model set to: {}", model);
                Response::success("set_whisper_model", serde_json::json!({"model": model}))
            }
            Err(e) => {
                error!("Failed to set Whisper model: {}", e);
                Response::error("set_whisper_model", format!("Failed to set model: {}", e))
            }
        }
    }

    async fn handle_download_whisper_model(&self, model: String) -> Response {
        info!("Download Whisper model command received: {}", model);
        Response::success("download_whisper_model", serde_json::json!({"model": model, "status": "started"}))
    }

    // Translation Command Handlers (stubs for now)
    async fn handle_translate_text(&self, text: String, source: String, target: String) -> Response {
        info!("Translate text command received: {} -> {}", source, target);
        Response::success("translate_text", serde_json::json!({
            "original": text,
            "translation": "Translation not yet implemented",
            "source": source,
            "target": target,
        }))
    }

    async fn handle_set_translation_engine(&self, engine: String) -> Response {
        info!("Set translation engine: {}", engine);
        Response::success("set_translation_engine", serde_json::json!({"engine": engine}))
    }

    async fn handle_set_source_language(&self, language: String) -> Response {
        info!("Set source language: {}", language);
        Response::success("set_source_language", serde_json::json!({"language": language}))
    }

    async fn handle_set_target_language(&self, language: String) -> Response {
        info!("Set target language: {}", language);
        Response::success("set_target_language", serde_json::json!({"language": language}))
    }

    // Configuration Command Handlers
    async fn handle_get_config(&self) -> Response {
        let config_data = self.config.get_all().await;
        Response::success("get_config", serde_json::to_value(config_data).unwrap_or_default())
    }

    async fn handle_set_config(&self, config: serde_json::Value) -> Response {
        match serde_json::from_value(config) {
            Ok(config_data) => {
                match self.config.set_all(config_data).await {
                    Ok(_) => {
                        info!("Configuration updated");
                        Response::success("set_config", serde_json::json!({"updated": true}))
                    }
                    Err(e) => {
                        error!("Failed to set configuration: {}", e);
                        Response::error("set_config", format!("Failed to set config: {}", e))
                    }
                }
            }
            Err(e) => {
                error!("Failed to parse configuration: {}", e);
                Response::error("set_config", format!("Invalid config format: {}", e))
            }
        }
    }

    async fn handle_set_transparency(&self, value: u8) -> Response {
        match self.config.set_transparency(value).await {
            Ok(_) => {
                info!("Transparency set to: {}", value);
                Response::success("set_transparency", serde_json::json!({"transparency": value}))
            }
            Err(e) => {
                error!("Failed to set transparency: {}", e);
                Response::error("set_transparency", format!("Failed to set: {}", e))
            }
        }
    }

    async fn handle_set_ui_language(&self, language: String) -> Response {
        match self.config.set_ui_language(language.clone()).await {
            Ok(_) => {
                info!("UI language set to: {}", language);
                Response::success("set_ui_language", serde_json::json!({"language": language}))
            }
            Err(e) => {
                error!("Failed to set UI language: {}", e);
                Response::error("set_ui_language", format!("Failed to set: {}", e))
            }
        }
    }

    async fn handle_set_ui_scaling(&self, scaling: u8) -> Response {
        match self.config.set(|data| data.ui_scaling = scaling).await {
            Ok(_) => {
                info!("UI scaling set to: {}", scaling);
                Response::success("set_ui_scaling", serde_json::json!({"scaling": scaling}))
            }
            Err(e) => {
                error!("Failed to set UI scaling: {}", e);
                Response::error("set_ui_scaling", format!("Failed to set: {}", e))
            }
        }
    }

    // OSC Command Handlers
    async fn handle_set_osc_settings(&self, ip: String, port: u16) -> Response {
        match self.config.set_osc_settings(ip.clone(), port).await {
            Ok(_) => {
                info!("OSC settings set to: {}:{}", ip, port);
                Response::success("set_osc_settings", serde_json::json!({"ip": ip, "port": port}))
            }
            Err(e) => {
                error!("Failed to set OSC settings: {}", e);
                Response::error("set_osc_settings", format!("Failed to set: {}", e))
            }
        }
    }

    async fn handle_enable_osc_send(&self, enabled: bool) -> Response {
        match self.config.set(|data| data.send_message_to_vrc = enabled).await {
            Ok(_) => {
                info!("OSC send set to: {}", enabled);
                Response::success("enable_osc_send", serde_json::json!({"enabled": enabled}))
            }
            Err(e) => {
                error!("Failed to set OSC send: {}", e);
                Response::error("enable_osc_send", format!("Failed to set: {}", e))
            }
        }
    }

    async fn handle_enable_mic_mute_sync(&self, enabled: bool) -> Response {
        match self.config.set(|data| data.vrc_mic_mute_sync = enabled).await {
            Ok(_) => {
                info!("Mic mute sync set to: {}", enabled);
                Response::success("enable_mic_mute_sync", serde_json::json!({"enabled": enabled}))
            }
            Err(e) => {
                error!("Failed to set mic mute sync: {}", e);
                Response::error("enable_mic_mute_sync", format!("Failed to set: {}", e))
            }
        }
    }

    // WebSocket Command Handlers
    async fn handle_set_websocket_settings(&self, host: String, port: u16) -> Response {
        match self.config.set(|data| {
            data.websocket_host = host.clone();
            data.websocket_port = port;
        }).await {
            Ok(_) => {
                info!("WebSocket settings set to: {}:{}", host, port);
                Response::success("set_websocket_settings", serde_json::json!({"host": host, "port": port}))
            }
            Err(e) => {
                error!("Failed to set WebSocket settings: {}", e);
                Response::error("set_websocket_settings", format!("Failed to set: {}", e))
            }
        }
    }

    async fn handle_enable_websocket_server(&self, enabled: bool) -> Response {
        match self.config.set(|data| data.websocket_server = enabled).await {
            Ok(_) => {
                info!("WebSocket server set to: {}", enabled);
                Response::success("enable_websocket_server", serde_json::json!({"enabled": enabled}))
            }
            Err(e) => {
                error!("Failed to set WebSocket server: {}", e);
                Response::error("enable_websocket_server", format!("Failed to set: {}", e))
            }
        }
    }

    // Overlay Command Handlers
    async fn handle_enable_small_log_overlay(&self, enabled: bool) -> Response {
        match self.config.set(|data| data.overlay_small_log = enabled).await {
            Ok(_) => {
                info!("Small log overlay set to: {}", enabled);
                Response::success("enable_small_log_overlay", serde_json::json!({"enabled": enabled}))
            }
            Err(e) => {
                error!("Failed to set small log overlay: {}", e);
                Response::error("enable_small_log_overlay", format!("Failed to set: {}", e))
            }
        }
    }

    async fn handle_enable_large_log_overlay(&self, enabled: bool) -> Response {
        match self.config.set(|data| data.overlay_large_log = enabled).await {
            Ok(_) => {
                info!("Large log overlay set to: {}", enabled);
                Response::success("enable_large_log_overlay", serde_json::json!({"enabled": enabled}))
            }
            Err(e) => {
                error!("Failed to set large log overlay: {}", e);
                Response::error("enable_large_log_overlay", format!("Failed to set: {}", e))
            }
        }
    }

    async fn handle_set_overlay_settings(&self, _settings: serde_json::Value) -> Response {
        info!("Set overlay settings command received");
        Response::success("set_overlay_settings", serde_json::json!({"updated": true}))
    }

    // Word Filter Command Handlers
    async fn handle_set_word_filter(&self, words: Vec<String>) -> Response {
        match self.config.set(|data| data.mic_word_filter = words.clone()).await {
            Ok(_) => {
                // Update the keyword filter
                let filter = self.keyword_filter.read().await;
                filter.load_filters(words.clone());
                info!("Word filter updated with {} words", words.len());
                Response::success("set_word_filter", serde_json::json!({"words": words}))
            }
            Err(e) => {
                error!("Failed to set word filter: {}", e);
                Response::error("set_word_filter", format!("Failed to set: {}", e))
            }
        }
    }

    // Logging Command Handlers
    async fn handle_enable_logging(&self, enabled: bool) -> Response {
        match self.config.set(|data| data.logger_feature = enabled).await {
            Ok(_) => {
                info!("Logging set to: {}", enabled);
                Response::success("enable_logging", serde_json::json!({"enabled": enabled}))
            }
            Err(e) => {
                error!("Failed to set logging: {}", e);
                Response::error("enable_logging", format!("Failed to set: {}", e))
            }
        }
    }

    async fn handle_open_logs_folder(&self) -> Response {
        info!("Open logs folder command received");
        Response::success("open_logs_folder", serde_json::json!({"opened": true}))
    }

    // System Command Handlers
    async fn handle_shutdown(&self) -> Response {
        info!("Shutdown command received");
        if let Err(e) = self.shutdown().await {
            error!("Error during shutdown: {}", e);
            Response::error("shutdown", format!("Shutdown error: {}", e))
        } else {
            Response::success("shutdown", serde_json::json!({"shutdown": true}))
        }
    }

    async fn handle_get_version(&self) -> Response {
        let version = env!("CARGO_PKG_VERSION");
        Response::success("get_version", serde_json::json!({"version": version}))
    }

    // Typing Indicator Handlers
    async fn handle_start_typing(&self) -> Response {
        let mut state = self.state.write().await;
        state.is_typing = true;
        info!("Typing indicator started");
        
        // Send typing indicator via OSC if available
        if let Some(ref osc) = *self.osc_handler.read().await {
            if let Err(e) = osc.send_typing(true).await {
                warn!("Failed to send typing indicator via OSC: {}", e);
            }
        }
        
        Response::success("start_typing", serde_json::json!({"typing": true}))
    }

    async fn handle_stop_typing(&self) -> Response {
        let mut state = self.state.write().await;
        state.is_typing = false;
        info!("Typing indicator stopped");
        
        // Send typing indicator via OSC if available
        if let Some(ref osc) = *self.osc_handler.read().await {
            if let Err(e) = osc.send_typing(false).await {
                warn!("Failed to send typing indicator via OSC: {}", e);
            }
        }
        
        Response::success("stop_typing", serde_json::json!({"typing": false}))
    }

    // Threshold Check Handlers
    async fn handle_enable_mic_threshold_check(&self) -> Response {
        info!("Microphone threshold check enabled");
        // This enables a mode where the UI can monitor mic energy levels
        // to help users set appropriate thresholds
        Response::success("enable_mic_threshold_check", serde_json::json!({"enabled": true}))
    }

    async fn handle_disable_mic_threshold_check(&self) -> Response {
        info!("Microphone threshold check disabled");
        Response::success("disable_mic_threshold_check", serde_json::json!({"enabled": false}))
    }

    async fn handle_enable_speaker_threshold_check(&self) -> Response {
        info!("Speaker threshold check enabled");
        Response::success("enable_speaker_threshold_check", serde_json::json!({"enabled": true}))
    }

    async fn handle_disable_speaker_threshold_check(&self) -> Response {
        info!("Speaker threshold check disabled");
        Response::success("disable_speaker_threshold_check", serde_json::json!({"enabled": false}))
    }

    // Folder Handlers
    async fn handle_open_config_folder(&self) -> Response {
        info!("Open config folder command received");
        
        // Get the config directory path
        if let Some(config_dir) = dirs::config_dir() {
            let vrct_config_dir = config_dir.join("vrct");
            
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("explorer")
                    .arg(vrct_config_dir.to_string_lossy().to_string())
                    .spawn();
            }
            
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg(vrct_config_dir.to_string_lossy().to_string())
                    .spawn();
            }
            
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open")
                    .arg(vrct_config_dir.to_string_lossy().to_string())
                    .spawn();
            }
            
            Response::success("open_config_folder", serde_json::json!({
                "opened": true,
                "path": vrct_config_dir.to_string_lossy()
            }))
        } else {
            Response::error("open_config_folder", "Could not determine config directory")
        }
    }

    // Language Swap Handler
    async fn handle_swap_languages(&self) -> Response {
        info!("Swap languages command received");
        
        // Get current source and target languages from config
        let (source, target) = self.config.get(|data| {
            (data.selected_your_languages.clone(), data.selected_target_languages.clone())
        }).await;
        
        // Swap them
        match self.config.set(|data| {
            data.selected_your_languages = target.clone();
            data.selected_target_languages = source.clone();
        }).await {
            Ok(_) => {
                info!("Languages swapped successfully");
                Response::success("swap_languages", serde_json::json!({
                    "swapped": true,
                    "source": serde_json::to_value(&target).unwrap_or_default(),
                    "target": serde_json::to_value(&source).unwrap_or_default(),
                }))
            }
            Err(e) => {
                error!("Failed to swap languages: {}", e);
                Response::error("swap_languages", format!("Failed to swap: {}", e))
            }
        }
    }
}


// Transcription workflow implementation
impl Controller {
    /// Start transcription workflow
    /// Coordinates audio recorder, transcriber, and translator
    pub async fn start_transcription(&self) -> Result<()> {
        info!("Starting transcription workflow");
        
        let mut state = self.state.write().await;
        state.transcription_enabled = true;
        drop(state);
        
        // Get microphone device from config
        let (_host, device) = self.config.get_mic_device().await;
        
        // Find the audio device
        let audio_device = self.device_manager
            .get_device_by_name(&device)?
            .ok_or_else(|| crate::utils::error::VrctError::AudioDevice(
                format!("Device not found: {}", device)
            ))?;
        
        // Get threshold from config
        let threshold = self.config.get(|data| data.mic_threshold).await;
        
        // Create audio recorder
        let recorder_config = crate::audio::recorder::RecorderConfig {
            energy_threshold: threshold as f32,
            vad_enabled: true,
            buffer_size: 48000,
            enable_energy_reporting: true,
            energy_report_interval_ms: 100,
        };
        
        let recorder = AudioRecorder::new(audio_device, recorder_config)?;
        
        // Start recording
        let mut recorder_guard = self.audio_recorder.write().await;
        *recorder_guard = Some(recorder);
        if let Some(ref mut rec) = *recorder_guard {
            rec.start()?;
        }
        
        info!("Transcription started");
        Ok(())
    }
    
    /// Stop transcription workflow
    pub async fn stop_transcription(&self) -> Result<()> {
        info!("Stopping transcription workflow");
        
        let mut state = self.state.write().await;
        state.transcription_enabled = false;
        drop(state);
        
        // Stop audio recorder
        if let Some(mut recorder) = self.audio_recorder.write().await.take() {
            recorder.stop()?;
        }
        
        info!("Transcription stopped");
        Ok(())
    }
    
    /// Process transcription result
    /// Applies word filters and sends to translation if enabled
    #[allow(dead_code)] // Placeholder for full transcription pipeline
    async fn process_transcription(&self, text: String, language: String) -> Result<()> {
        info!("Processing transcription: {} ({})", text, language);
        
        // Check word filter
        let filter = self.keyword_filter.read().await;
        if filter.contains_filtered_word(&text) {
            warn!("Message blocked by word filter: {}", text);
            // Send notification to frontend
            self.broadcast_event("message_blocked", serde_json::json!({
                "reason": "word_filter",
                "text": text,
            })).await?;
            return Ok(());
        }
        drop(filter);
        
        // Check if translation is enabled
        let state = self.state.read().await;
        let translation_enabled = state.translation_enabled;
        let send_enabled = state.transcription_send_enabled;
        drop(state);
        
        let mut translation = None;
        if translation_enabled {
            // Translate the text
            match self.translate_text_internal(&text, &language, "en").await {
                Ok(translated) => {
                    translation = Some(translated);
                }
                Err(e) => {
                    error!("Translation failed: {}", e);
                }
            }
        }
        
        // Format and send message
        if send_enabled {
            self.send_formatted_message(&text, translation.as_deref()).await?;
        }
        
        // Broadcast to WebSocket clients
        self.broadcast_transcription(&text, translation.as_deref()).await?;
        
        // Update overlays
        self.update_overlays(&text, translation.as_deref()).await?;
        
        Ok(())
    }
    
    /// Send formatted message via OSC
    #[allow(dead_code)] // Placeholder for full transcription pipeline
    async fn send_formatted_message(&self, original: &str, translation: Option<&str>) -> Result<()> {
        let formatter = self.message_formatter.read().await;
        let formatted = formatter.format_sent_message(original, translation)?;
        drop(formatter);
        
        // Send via OSC if enabled
        if let Some(ref osc) = *self.osc_handler.read().await {
            osc.send_message(&formatted, false).await?;
        }
        
        Ok(())
    }
    
    /// Broadcast transcription event to WebSocket clients
    #[allow(dead_code)] // Placeholder for full transcription pipeline
    async fn broadcast_transcription(&self, original: &str, translation: Option<&str>) -> Result<()> {
        let message = crate::communication::websocket::WebSocketMessage {
            message_type: "transcription".to_string(),
            data: serde_json::json!({
                "original": original,
                "translation": translation,
            }),
        };
        
        let server = self.websocket_server.read().await;
        server.broadcast(&message).await?;
        
        Ok(())
    }
    
    /// Update VR overlays with transcription
    #[allow(dead_code)] // Placeholder for full transcription pipeline
    async fn update_overlays(&self, original: &str, translation: Option<&str>) -> Result<()> {
        if let Some(ref overlay) = *self.overlay_manager.read().await {
            let text = if let Some(trans) = translation {
                format!("{}\n{}", original, trans)
            } else {
                original.to_string()
            };
            
            overlay.update_overlay("small_log", &text)?;
        }
        
        Ok(())
    }
    
    /// Broadcast a generic event to frontend
    async fn broadcast_event(&self, event_type: &str, data: serde_json::Value) -> Result<()> {
        let message = crate::communication::websocket::WebSocketMessage {
            message_type: event_type.to_string(),
            data,
        };
        
        let server = self.websocket_server.read().await;
        server.broadcast(&message).await?;
        
        Ok(())
    }
}


// Translation workflow implementation
impl Controller {
    /// Translate text using the configured translation engine
    /// Handles engine selection and fallback logic
    pub async fn translate_text_internal(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String> {
        info!("Translating text: {} -> {}", source_lang, target_lang);
        
        // The TranslationManager handles active engine selection and fallback internally
        // It will automatically fall back to CTranslate2 if the active engine fails
        match self.translator.translate(text, source_lang, target_lang).await {
            Ok(translation) => {
                info!("Translation successful");
                Ok(translation)
            }
            Err(e) => {
                error!("Translation failed: {}", e);
                Err(e)
            }
        }
    }
}


// OSC integration implementation
impl Controller {
    /// Send typing indicator via OSC
    pub async fn send_typing_indicator(&self, is_typing: bool) -> Result<()> {
        if let Some(ref osc) = *self.osc_handler.read().await {
            osc.send_typing(is_typing).await?;
        }
        Ok(())
    }
    
    /// Handle VRC mic mute sync
    /// This would be called when OSC parameters are received
    pub async fn handle_mic_mute_sync(&self, is_muted: bool) -> Result<()> {
        info!("VRC mic mute sync: {}", is_muted);
        
        let state = self.state.read().await;
        let transcription_enabled = state.transcription_enabled;
        drop(state);
        
        // If muted, stop transcription; if unmuted, start transcription
        if is_muted && transcription_enabled {
            self.stop_transcription().await?;
        } else if !is_muted && !transcription_enabled {
            self.start_transcription().await?;
        }
        
        Ok(())
    }
}


// Error handling implementation
impl Controller {
    /// Handle errors from subsystems and report to frontend
    /// Logs errors with context and returns structured error responses
    pub async fn handle_error(&self, error: crate::utils::error::VrctError, context: &str) -> Response {
        error!("Error in {}: {}", context, error);
        
        // Determine error type and user-friendly message
        let (error_type, user_message) = match &error {
            crate::utils::error::VrctError::AudioDevice(msg) => {
                ("audio_device", format!("Audio device error: {}", msg))
            }
            crate::utils::error::VrctError::Transcription(msg) => {
                ("transcription", format!("Transcription error: {}", msg))
            }
            crate::utils::error::VrctError::Translation(msg) => {
                ("translation", format!("Translation error: {}", msg))
            }
            crate::utils::error::VrctError::Osc(msg) => {
                ("osc", format!("OSC communication error: {}", msg))
            }
            crate::utils::error::VrctError::WebSocket(msg) => {
                ("websocket", format!("WebSocket error: {}", msg))
            }
            crate::utils::error::VrctError::Overlay(msg) => {
                ("overlay", format!("Overlay error: {}", msg))
            }
            crate::utils::error::VrctError::Config(msg) => {
                ("config", format!("Configuration error: {}", msg))
            }
            crate::utils::error::VrctError::VramOverflow(msg) => {
                ("vram_overflow", format!("VRAM overflow: {}. Falling back to CPU.", msg))
            }
            crate::utils::error::VrctError::ZludaRuntime(msg) => {
                ("zluda_runtime", format!("ZLUDA runtime error: {}. Falling back to CPU.", msg))
            }
            crate::utils::error::VrctError::AudioRecording(msg) => {
                ("audio_recording", format!("Audio recording error: {}", msg))
            }
            crate::utils::error::VrctError::ModelLoading(msg) => {
                ("model_loading", format!("Model loading error: {}", msg))
            }
            crate::utils::error::VrctError::ModelDownload(msg) => {
                ("model_download", format!("Model download error: {}", msg))
            }
            crate::utils::error::VrctError::Initialization(msg) => {
                ("initialization", format!("Initialization error: {}", msg))
            }
            crate::utils::error::VrctError::Io(e) => {
                ("io", format!("I/O error: {}", e))
            }
            crate::utils::error::VrctError::Serialization(e) => {
                ("serialization", format!("Serialization error: {}", e))
            }
            crate::utils::error::VrctError::Http(e) => {
                ("http", format!("HTTP error: {}", e))
            }
            crate::utils::error::VrctError::Candle(msg) => {
                ("candle", format!("ML framework error: {}", msg))
            }
            crate::utils::error::VrctError::Update(msg) => {
                ("update", format!("Update error: {}", msg))
            }
            crate::utils::error::VrctError::Unknown(msg) => {
                ("unknown", format!("Unknown error: {}", msg))
            }
        };
        
        // Broadcast error event to frontend
        let _ = self.broadcast_event("error", serde_json::json!({
            "error_type": error_type,
            "message": user_message,
            "context": context,
        })).await;
        
        Response {
            status: 500,
            endpoint: context.to_string(),
            result: serde_json::json!({
                "error_type": error_type,
                "message": user_message,
            }),
        }
    }
}
