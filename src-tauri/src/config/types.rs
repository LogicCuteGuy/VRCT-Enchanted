// Configuration data structures
// Implements serde serialization/deserialization for all configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration data structure containing all application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    // UI Settings
    #[serde(rename = "TRANSPARENCY")]
    pub transparency: u8,
    
    #[serde(rename = "UI_SCALING")]
    pub ui_scaling: u8,
    
    #[serde(rename = "TEXTBOX_UI_SCALING")]
    pub textbox_ui_scaling: u8,
    
    #[serde(rename = "MESSAGE_BOX_RATIO")]
    pub message_box_ratio: u8,
    
    #[serde(rename = "UI_LANGUAGE")]
    pub ui_language: String,
    
    #[serde(rename = "FONT_FAMILY")]
    pub font_family: String,
    
    #[serde(rename = "MAIN_WINDOW_SIDEBAR_COMPACT_MODE")]
    pub main_window_sidebar_compact_mode: bool,
    
    #[serde(rename = "SEND_MESSAGE_BUTTON_TYPE")]
    pub send_message_button_type: String,
    
    #[serde(rename = "SHOW_RESEND_BUTTON")]
    pub show_resend_button: bool,
    
    #[serde(rename = "MAIN_WINDOW_GEOMETRY")]
    pub main_window_geometry: WindowGeometry,
    
    // Audio Settings - Microphone
    #[serde(rename = "AUTO_MIC_SELECT")]
    pub auto_mic_select: bool,
    
    #[serde(rename = "SELECTED_MIC_HOST")]
    pub selected_mic_host: String,
    
    #[serde(rename = "SELECTED_MIC_DEVICE")]
    pub selected_mic_device: String,
    
    #[serde(rename = "MIC_THRESHOLD")]
    pub mic_threshold: u32,
    
    #[serde(rename = "MIC_AUTOMATIC_THRESHOLD")]
    pub mic_automatic_threshold: bool,
    
    #[serde(rename = "MIC_RECORD_TIMEOUT")]
    pub mic_record_timeout: u32,
    
    #[serde(rename = "MIC_PHRASE_TIMEOUT")]
    pub mic_phrase_timeout: u32,
    
    #[serde(rename = "MIC_MAX_PHRASES")]
    pub mic_max_phrases: u32,
    
    #[serde(rename = "MIC_AVG_LOGPROB")]
    pub mic_avg_logprob: f32,
    
    #[serde(rename = "MIC_NO_SPEECH_PROB")]
    pub mic_no_speech_prob: f32,
    
    #[serde(rename = "MIC_NO_REPEAT_NGRAM_SIZE")]
    pub mic_no_repeat_ngram_size: u32,
    
    #[serde(rename = "MIC_VAD_FILTER")]
    pub mic_vad_filter: bool,
    
    #[serde(rename = "MIC_VAD_PARAMETERS")]
    pub mic_vad_parameters: VadParameters,
    
    #[serde(rename = "MIC_WORD_FILTER")]
    pub mic_word_filter: Vec<String>,
    
    // Audio Settings - Speaker
    #[serde(rename = "AUTO_SPEAKER_SELECT")]
    pub auto_speaker_select: bool,
    
    #[serde(rename = "SELECTED_SPEAKER_DEVICE")]
    pub selected_speaker_device: String,
    
    #[serde(rename = "SPEAKER_THRESHOLD")]
    pub speaker_threshold: u32,
    
    #[serde(rename = "SPEAKER_AUTOMATIC_THRESHOLD")]
    pub speaker_automatic_threshold: bool,
    
    #[serde(rename = "SPEAKER_RECORD_TIMEOUT")]
    pub speaker_record_timeout: u32,
    
    #[serde(rename = "SPEAKER_PHRASE_TIMEOUT")]
    pub speaker_phrase_timeout: u32,
    
    #[serde(rename = "SPEAKER_MAX_PHRASES")]
    pub speaker_max_phrases: u32,
    
    #[serde(rename = "SPEAKER_AVG_LOGPROB")]
    pub speaker_avg_logprob: f32,
    
    #[serde(rename = "SPEAKER_NO_SPEECH_PROB")]
    pub speaker_no_speech_prob: f32,
    
    #[serde(rename = "SPEAKER_NO_REPEAT_NGRAM_SIZE")]
    pub speaker_no_repeat_ngram_size: u32,
    
    #[serde(rename = "SPEAKER_VAD_FILTER")]
    pub speaker_vad_filter: bool,
    
    #[serde(rename = "SPEAKER_VAD_PARAMETERS")]
    pub speaker_vad_parameters: VadParameters,
    
    // Transcription Settings
    #[serde(rename = "SELECTED_TRANSCRIPTION_ENGINE")]
    pub selected_transcription_engine: String,
    
    #[serde(rename = "WHISPER_WEIGHT_TYPE")]
    pub whisper_weight_type: String,
    
    #[serde(rename = "SELECTED_TRANSCRIPTION_COMPUTE_DEVICE")]
    pub selected_transcription_compute_device: ComputeDevice,
    
    #[serde(rename = "SELECTED_TRANSCRIPTION_COMPUTE_TYPE")]
    pub selected_transcription_compute_type: String,
    
    // Translation Settings
    #[serde(rename = "SELECTED_TRANSLATION_ENGINES")]
    pub selected_translation_engines: HashMap<String, String>,
    
    #[serde(rename = "CTRANSLATE2_WEIGHT_TYPE")]
    pub ctranslate2_weight_type: String,
    
    #[serde(rename = "SELECTED_TRANSLATION_COMPUTE_DEVICE")]
    pub selected_translation_compute_device: ComputeDevice,
    
    #[serde(rename = "SELECTED_TRANSLATION_COMPUTE_TYPE")]
    pub selected_translation_compute_type: String,
    
    #[serde(rename = "SELECTED_YOUR_LANGUAGES")]
    pub selected_your_languages: HashMap<String, HashMap<String, LanguageConfig>>,
    
    #[serde(rename = "SELECTED_TARGET_LANGUAGES")]
    pub selected_target_languages: HashMap<String, HashMap<String, LanguageConfig>>,
    
    // API Keys
    #[serde(rename = "AUTH_KEYS")]
    pub auth_keys: AuthKeys,
    
    #[serde(rename = "LMSTUDIO_URL")]
    pub lmstudio_url: String,
    
    #[serde(rename = "SELECTED_PLAMO_MODEL")]
    pub selected_plamo_model: Option<String>,
    
    #[serde(rename = "SELECTED_GEMINI_MODEL")]
    pub selected_gemini_model: Option<String>,
    
    #[serde(rename = "SELECTED_OPENAI_MODEL")]
    pub selected_openai_model: Option<String>,
    
    #[serde(rename = "SELECTED_LMSTUDIO_MODEL")]
    pub selected_lmstudio_model: Option<String>,
    
    #[serde(rename = "SELECTED_OLLAMA_MODEL")]
    pub selected_ollama_model: Option<String>,
    
    // OSC Settings
    #[serde(rename = "OSC_IP_ADDRESS")]
    pub osc_ip_address: String,
    
    #[serde(rename = "OSC_PORT")]
    pub osc_port: u16,
    
    #[serde(rename = "SEND_MESSAGE_TO_VRC")]
    pub send_message_to_vrc: bool,
    
    #[serde(rename = "SEND_RECEIVED_MESSAGE_TO_VRC")]
    pub send_received_message_to_vrc: bool,
    
    #[serde(rename = "VRC_MIC_MUTE_SYNC")]
    pub vrc_mic_mute_sync: bool,
    
    #[serde(rename = "NOTIFICATION_VRC_SFX")]
    pub notification_vrc_sfx: bool,
    
    // WebSocket Settings
    #[serde(rename = "WEBSOCKET_HOST")]
    pub websocket_host: String,
    
    #[serde(rename = "WEBSOCKET_PORT")]
    pub websocket_port: u16,
    
    #[serde(rename = "WEBSOCKET_SERVER")]
    pub websocket_server: bool,
    
    // Overlay Settings
    #[serde(rename = "OVERLAY_SMALL_LOG")]
    pub overlay_small_log: bool,
    
    #[serde(rename = "OVERLAY_LARGE_LOG")]
    pub overlay_large_log: bool,
    
    #[serde(rename = "OVERLAY_SHOW_ONLY_TRANSLATED_MESSAGES")]
    pub overlay_show_only_translated_messages: bool,
    
    #[serde(rename = "OVERLAY_SMALL_LOG_SETTINGS")]
    pub overlay_small_log_settings: OverlayConfig,
    
    #[serde(rename = "OVERLAY_LARGE_LOG_SETTINGS")]
    pub overlay_large_log_settings: OverlayConfig,
    
    // Message Formatting
    #[serde(rename = "SEND_MESSAGE_FORMAT_PARTS")]
    pub send_message_format_parts: MessageFormatParts,
    
    #[serde(rename = "RECEIVED_MESSAGE_FORMAT_PARTS")]
    pub received_message_format_parts: MessageFormatParts,
    
    #[serde(rename = "SEND_ONLY_TRANSLATED_MESSAGES")]
    pub send_only_translated_messages: bool,
    
    #[serde(rename = "AUTO_CLEAR_MESSAGE_BOX")]
    pub auto_clear_message_box: bool,
    
    // Transliteration Settings
    #[serde(rename = "CONVERT_MESSAGE_TO_ROMAJI")]
    pub convert_message_to_romaji: bool,
    
    #[serde(rename = "CONVERT_MESSAGE_TO_HIRAGANA")]
    pub convert_message_to_hiragana: bool,
    
    // Other Settings
    #[serde(rename = "LOGGER_FEATURE")]
    pub logger_feature: bool,
    
    #[serde(rename = "USE_EXCLUDE_WORDS")]
    pub use_exclude_words: bool,
    
    #[serde(rename = "SELECTED_TAB_NO")]
    pub selected_tab_no: String,
    
    #[serde(rename = "HOTKEYS")]
    pub hotkeys: Hotkeys,
    
    #[serde(rename = "PLUGINS_STATUS")]
    pub plugins_status: Vec<serde_json::Value>,
    
    #[serde(rename = "ZLUDA_PATH")]
    pub zluda_path: Option<String>,
}

/// Window geometry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x_pos: i32,
    pub y_pos: i32,
    pub width: u32,
    pub height: u32,
}

/// Voice Activity Detection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadParameters {
    pub threshold: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neg_threshold: Option<f32>,
    pub min_speech_duration_ms: u32,
    #[serde(serialize_with = "serialize_f32_or_infinity")]
    #[serde(deserialize_with = "deserialize_f32_or_infinity")]
    pub max_speech_duration_s: f32,
    pub min_silence_duration_ms: u32,
    pub speech_pad_ms: u32,
}

// Helper functions for serializing/deserializing infinity
fn serialize_f32_or_infinity<S>(value: &f32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if value.is_infinite() {
        serializer.serialize_str("Infinity")
    } else {
        serializer.serialize_f32(*value)
    }
}

fn deserialize_f32_or_infinity<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;
    
    struct F32OrInfinityVisitor;
    
    impl<'de> Visitor<'de> for F32OrInfinityVisitor {
        type Value = f32;
        
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a float or the string 'Infinity'")
        }
        
        fn visit_f32<E>(self, value: f32) -> Result<f32, E>
        where
            E: de::Error,
        {
            Ok(value)
        }
        
        fn visit_f64<E>(self, value: f64) -> Result<f32, E>
        where
            E: de::Error,
        {
            Ok(value as f32)
        }
        
        fn visit_str<E>(self, value: &str) -> Result<f32, E>
        where
            E: de::Error,
        {
            if value == "Infinity" {
                Ok(f32::INFINITY)
            } else {
                Err(de::Error::custom(format!("expected 'Infinity', got '{}'", value)))
            }
        }
    }
    
    deserializer.deserialize_any(F32OrInfinityVisitor)
}

/// Compute device configuration for ML models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeDevice {
    pub device: String,
    pub device_index: usize,
    pub device_name: String,
    pub compute_types: Vec<String>,
}

/// Language configuration for transcription/translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub language: String,
    pub country: String,
    pub enable: bool,
}

/// Authentication keys for various services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthKeys {
    #[serde(rename = "DeepL_API")]
    pub deepl_api: Option<String>,
    
    #[serde(rename = "Plamo_API")]
    pub plamo_api: Option<String>,
    
    #[serde(rename = "Gemini_API")]
    pub gemini_api: Option<String>,
    
    #[serde(rename = "OpenAI_API")]
    pub openai_api: Option<String>,
}

/// VR overlay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub x_pos: f32,
    pub y_pos: f32,
    pub z_pos: f32,
    pub x_rotation: f32,
    pub y_rotation: f32,
    pub z_rotation: f32,
    pub display_duration: u32,
    pub fadeout_duration: u32,
    pub opacity: f32,
    pub ui_scaling: f32,
    pub tracker: String,
}

/// Message formatting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFormatParts {
    pub message: MessagePart,
    pub separator: String,
    pub translation: TranslationPart,
    pub translation_first: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    pub prefix: String,
    pub suffix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationPart {
    pub prefix: String,
    pub separator: String,
    pub suffix: String,
}

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkeys {
    pub toggle_vrct_visibility: Option<String>,
    pub toggle_translation: Option<String>,
    pub toggle_transcription_send: Option<String>,
    pub toggle_transcription_receive: Option<String>,
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            // UI Settings
            transparency: 100,
            ui_scaling: 100,
            textbox_ui_scaling: 100,
            message_box_ratio: 10,
            ui_language: "en".to_string(),
            font_family: "Yu Gothic UI".to_string(),
            main_window_sidebar_compact_mode: false,
            send_message_button_type: "show".to_string(),
            show_resend_button: false,
            main_window_geometry: WindowGeometry::default(),
            
            // Audio Settings - Microphone
            auto_mic_select: true,
            selected_mic_host: "MME".to_string(),
            selected_mic_device: String::new(),
            mic_threshold: 300,
            mic_automatic_threshold: false,
            mic_record_timeout: 3,
            mic_phrase_timeout: 3,
            mic_max_phrases: 10,
            mic_avg_logprob: -0.8,
            mic_no_speech_prob: 0.6,
            mic_no_repeat_ngram_size: 0,
            mic_vad_filter: false,
            mic_vad_parameters: VadParameters::default(),
            mic_word_filter: Vec::new(),
            
            // Audio Settings - Speaker
            auto_speaker_select: true,
            selected_speaker_device: String::new(),
            speaker_threshold: 300,
            speaker_automatic_threshold: false,
            speaker_record_timeout: 3,
            speaker_phrase_timeout: 3,
            speaker_max_phrases: 10,
            speaker_avg_logprob: -0.8,
            speaker_no_speech_prob: 0.6,
            speaker_no_repeat_ngram_size: 0,
            speaker_vad_filter: false,
            speaker_vad_parameters: VadParameters::default(),
            
            // Transcription Settings
            selected_transcription_engine: "Google".to_string(),
            whisper_weight_type: "base".to_string(),
            selected_transcription_compute_device: ComputeDevice::default(),
            selected_transcription_compute_type: "auto".to_string(),
            
            // Translation Settings
            selected_translation_engines: {
                let mut map = HashMap::new();
                map.insert("1".to_string(), "CTranslate2".to_string());
                map.insert("2".to_string(), "CTranslate2".to_string());
                map.insert("3".to_string(), "CTranslate2".to_string());
                map
            },
            ctranslate2_weight_type: "m2m100_418M-ct2-int8".to_string(),
            selected_translation_compute_device: ComputeDevice::default(),
            selected_translation_compute_type: "auto".to_string(),
            selected_your_languages: create_default_language_map("Japanese", "Japan"),
            selected_target_languages: create_default_language_map("English", "United States"),
            
            // API Keys
            auth_keys: AuthKeys::default(),
            lmstudio_url: "http://127.0.0.1:1234/v1".to_string(),
            selected_plamo_model: None,
            selected_gemini_model: None,
            selected_openai_model: None,
            selected_lmstudio_model: None,
            selected_ollama_model: None,
            
            // OSC Settings
            osc_ip_address: "127.0.0.1".to_string(),
            osc_port: 9000,
            send_message_to_vrc: true,
            send_received_message_to_vrc: false,
            vrc_mic_mute_sync: false,
            notification_vrc_sfx: true,
            
            // WebSocket Settings
            websocket_host: "127.0.0.1".to_string(),
            websocket_port: 2231,
            websocket_server: false,
            
            // Overlay Settings
            overlay_small_log: false,
            overlay_large_log: false,
            overlay_show_only_translated_messages: false,
            overlay_small_log_settings: OverlayConfig::default_small_log(),
            overlay_large_log_settings: OverlayConfig::default_large_log(),
            
            // Message Formatting
            send_message_format_parts: MessageFormatParts::default(),
            received_message_format_parts: MessageFormatParts::default(),
            send_only_translated_messages: false,
            auto_clear_message_box: true,
            
            // Transliteration Settings
            convert_message_to_romaji: false,
            convert_message_to_hiragana: false,
            
            // Other Settings
            logger_feature: false,
            use_exclude_words: true,
            selected_tab_no: "1".to_string(),
            hotkeys: Hotkeys::default(),
            plugins_status: Vec::new(),
            zluda_path: None,
        }
    }
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            x_pos: 0,
            y_pos: 0,
            width: 870,
            height: 654,
        }
    }
}

impl Default for VadParameters {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            neg_threshold: None,
            min_speech_duration_ms: 0,
            max_speech_duration_s: f32::INFINITY,
            min_silence_duration_ms: 2000,
            speech_pad_ms: 400,
        }
    }
}

impl Default for ComputeDevice {
    fn default() -> Self {
        Self {
            device: "cpu".to_string(),
            device_index: 0,
            device_name: "cpu".to_string(),
            compute_types: vec![
                "auto".to_string(),
                "float32".to_string(),
                "int16".to_string(),
                "int8".to_string(),
                "int8_float32".to_string(),
            ],
        }
    }
}

impl Default for AuthKeys {
    fn default() -> Self {
        Self {
            deepl_api: None,
            plamo_api: None,
            gemini_api: None,
            openai_api: None,
        }
    }
}

impl OverlayConfig {
    pub fn default_small_log() -> Self {
        Self {
            x_pos: 0.0,
            y_pos: 0.0,
            z_pos: 0.0,
            x_rotation: 0.0,
            y_rotation: 0.0,
            z_rotation: 0.0,
            display_duration: 5,
            fadeout_duration: 2,
            opacity: 1.0,
            ui_scaling: 1.0,
            tracker: "HMD".to_string(),
        }
    }
    
    pub fn default_large_log() -> Self {
        Self {
            x_pos: 0.0,
            y_pos: 0.0,
            z_pos: 0.0,
            x_rotation: 0.0,
            y_rotation: 0.0,
            z_rotation: 0.0,
            display_duration: 5,
            fadeout_duration: 2,
            opacity: 1.0,
            ui_scaling: 1.0,
            tracker: "LeftHand".to_string(),
        }
    }
}

impl Default for MessageFormatParts {
    fn default() -> Self {
        Self {
            message: MessagePart {
                prefix: String::new(),
                suffix: String::new(),
            },
            separator: "\n".to_string(),
            translation: TranslationPart {
                prefix: String::new(),
                separator: "\n".to_string(),
                suffix: String::new(),
            },
            translation_first: false,
        }
    }
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            toggle_vrct_visibility: None,
            toggle_translation: None,
            toggle_transcription_send: None,
            toggle_transcription_receive: None,
        }
    }
}

/// Helper function to create default language map structure
fn create_default_language_map(language: &str, country: &str) -> HashMap<String, HashMap<String, LanguageConfig>> {
    let mut outer_map = HashMap::new();
    
    for tab in ["1", "2", "3"] {
        let mut inner_map = HashMap::new();
        
        for slot in ["1", "2", "3"] {
            inner_map.insert(
                slot.to_string(),
                LanguageConfig {
                    language: language.to_string(),
                    country: country.to_string(),
                    enable: slot == "1", // Only first slot enabled by default
                },
            );
        }
        
        outer_map.insert(tab.to_string(), inner_map);
    }
    
    outer_map
}


// Configuration validation
impl ConfigData {
    /// Validate the configuration and apply defaults for invalid values
    /// 
    /// # Arguments
    /// * `available_devices` - Optional list of available audio device names for validation
    /// 
    /// # Returns
    /// * `Vec<String>` - List of validation warnings/errors
    pub fn validate(&mut self, available_devices: Option<&[String]>) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Validate transparency (0-100)
        if self.transparency > 100 {
            warnings.push(format!(
                "Invalid transparency value: {}. Using default: 100",
                self.transparency
            ));
            self.transparency = 100;
        }
        
        // Validate UI scaling (50-200)
        if self.ui_scaling < 50 || self.ui_scaling > 200 {
            warnings.push(format!(
                "Invalid UI scaling value: {}. Using default: 100",
                self.ui_scaling
            ));
            self.ui_scaling = 100;
        }
        
        // Validate textbox UI scaling (50-200)
        if self.textbox_ui_scaling < 50 || self.textbox_ui_scaling > 200 {
            warnings.push(format!(
                "Invalid textbox UI scaling value: {}. Using default: 100",
                self.textbox_ui_scaling
            ));
            self.textbox_ui_scaling = 100;
        }
        
        // Validate message box ratio (0-100)
        if self.message_box_ratio > 100 {
            warnings.push(format!(
                "Invalid message box ratio: {}. Using default: 10",
                self.message_box_ratio
            ));
            self.message_box_ratio = 10;
        }
        
        // Validate audio device selections if device list provided
        if let Some(devices) = available_devices {
            if !self.auto_mic_select && !self.selected_mic_device.is_empty() {
                if !devices.contains(&self.selected_mic_device) {
                    warnings.push(format!(
                        "Selected microphone device '{}' not found. Will use default.",
                        self.selected_mic_device
                    ));
                    self.auto_mic_select = true;
                }
            }
            
            if !self.auto_speaker_select && !self.selected_speaker_device.is_empty() {
                if !devices.contains(&self.selected_speaker_device) {
                    warnings.push(format!(
                        "Selected speaker device '{}' not found. Will use default.",
                        self.selected_speaker_device
                    ));
                    self.auto_speaker_select = true;
                }
            }
        }
        
        // Validate thresholds (0-10000)
        if self.mic_threshold > 10000 {
            warnings.push(format!(
                "Invalid mic threshold: {}. Using default: 300",
                self.mic_threshold
            ));
            self.mic_threshold = 300;
        }
        
        if self.speaker_threshold > 10000 {
            warnings.push(format!(
                "Invalid speaker threshold: {}. Using default: 300",
                self.speaker_threshold
            ));
            self.speaker_threshold = 300;
        }
        
        // Validate timeouts (1-30 seconds)
        if self.mic_record_timeout < 1 || self.mic_record_timeout > 30 {
            warnings.push(format!(
                "Invalid mic record timeout: {}. Using default: 3",
                self.mic_record_timeout
            ));
            self.mic_record_timeout = 3;
        }
        
        if self.speaker_record_timeout < 1 || self.speaker_record_timeout > 30 {
            warnings.push(format!(
                "Invalid speaker record timeout: {}. Using default: 3",
                self.speaker_record_timeout
            ));
            self.speaker_record_timeout = 3;
        }
        
        // Validate phrase timeouts (1-10 seconds)
        if self.mic_phrase_timeout < 1 || self.mic_phrase_timeout > 10 {
            warnings.push(format!(
                "Invalid mic phrase timeout: {}. Using default: 3",
                self.mic_phrase_timeout
            ));
            self.mic_phrase_timeout = 3;
        }
        
        if self.speaker_phrase_timeout < 1 || self.speaker_phrase_timeout > 10 {
            warnings.push(format!(
                "Invalid speaker phrase timeout: {}. Using default: 3",
                self.speaker_phrase_timeout
            ));
            self.speaker_phrase_timeout = 3;
        }
        
        // Validate max phrases (1-100)
        if self.mic_max_phrases < 1 || self.mic_max_phrases > 100 {
            warnings.push(format!(
                "Invalid mic max phrases: {}. Using default: 10",
                self.mic_max_phrases
            ));
            self.mic_max_phrases = 10;
        }
        
        if self.speaker_max_phrases < 1 || self.speaker_max_phrases > 100 {
            warnings.push(format!(
                "Invalid speaker max phrases: {}. Using default: 10",
                self.speaker_max_phrases
            ));
            self.speaker_max_phrases = 10;
        }
        
        // Validate avg_logprob (-1.0 to 0.0)
        if self.mic_avg_logprob < -1.0 || self.mic_avg_logprob > 0.0 {
            warnings.push(format!(
                "Invalid mic avg_logprob: {}. Using default: -0.8",
                self.mic_avg_logprob
            ));
            self.mic_avg_logprob = -0.8;
        }
        
        if self.speaker_avg_logprob < -1.0 || self.speaker_avg_logprob > 0.0 {
            warnings.push(format!(
                "Invalid speaker avg_logprob: {}. Using default: -0.8",
                self.speaker_avg_logprob
            ));
            self.speaker_avg_logprob = -0.8;
        }
        
        // Validate no_speech_prob (0.0 to 1.0)
        if self.mic_no_speech_prob < 0.0 || self.mic_no_speech_prob > 1.0 {
            warnings.push(format!(
                "Invalid mic no_speech_prob: {}. Using default: 0.6",
                self.mic_no_speech_prob
            ));
            self.mic_no_speech_prob = 0.6;
        }
        
        if self.speaker_no_speech_prob < 0.0 || self.speaker_no_speech_prob > 1.0 {
            warnings.push(format!(
                "Invalid speaker no_speech_prob: {}. Using default: 0.6",
                self.speaker_no_speech_prob
            ));
            self.speaker_no_speech_prob = 0.6;
        }
        
        // Validate VAD parameters
        Self::validate_vad_parameters(&mut self.mic_vad_parameters, &mut warnings, "mic");
        Self::validate_vad_parameters(&mut self.speaker_vad_parameters, &mut warnings, "speaker");
        
        // Validate overlay settings
        Self::validate_overlay_config(&mut self.overlay_small_log_settings, &mut warnings, "small_log");
        Self::validate_overlay_config(&mut self.overlay_large_log_settings, &mut warnings, "large_log");
        
        // Validate OSC port (1-65535)
        if self.osc_port == 0 {
            warnings.push(format!(
                "Invalid OSC port: {}. Using default: 9000",
                self.osc_port
            ));
            self.osc_port = 9000;
        }
        
        // Validate WebSocket port (1-65535)
        if self.websocket_port == 0 {
            warnings.push(format!(
                "Invalid WebSocket port: {}. Using default: 2231",
                self.websocket_port
            ));
            self.websocket_port = 2231;
        }
        
        // Validate compute device
        if !["cpu", "cuda", "zluda"].contains(&self.selected_transcription_compute_device.device.as_str()) {
            warnings.push(format!(
                "Invalid transcription compute device: {}. Using default: cpu",
                self.selected_transcription_compute_device.device
            ));
            self.selected_transcription_compute_device = ComputeDevice::default();
        }
        
        if !["cpu", "cuda", "zluda"].contains(&self.selected_translation_compute_device.device.as_str()) {
            warnings.push(format!(
                "Invalid translation compute device: {}. Using default: cpu",
                self.selected_translation_compute_device.device
            ));
            self.selected_translation_compute_device = ComputeDevice::default();
        }
        
        warnings
    }
    
    fn validate_vad_parameters(vad: &mut VadParameters, warnings: &mut Vec<String>, prefix: &str) {
        // Validate threshold (0.0 to 1.0)
        if vad.threshold < 0.0 || vad.threshold > 1.0 {
            warnings.push(format!(
                "Invalid {} VAD threshold: {}. Using default: 0.5",
                prefix, vad.threshold
            ));
            vad.threshold = 0.5;
        }
        
        // Validate neg_threshold if present (0.0 to 1.0)
        if let Some(neg_threshold) = vad.neg_threshold {
            if neg_threshold < 0.0 || neg_threshold > 1.0 {
                warnings.push(format!(
                    "Invalid {} VAD neg_threshold: {}. Resetting to None",
                    prefix, neg_threshold
                ));
                vad.neg_threshold = None;
            }
        }
        
        // Validate durations (reasonable ranges)
        if vad.min_silence_duration_ms > 10000 {
            warnings.push(format!(
                "Invalid {} VAD min_silence_duration_ms: {}. Using default: 2000",
                prefix, vad.min_silence_duration_ms
            ));
            vad.min_silence_duration_ms = 2000;
        }
        
        if vad.speech_pad_ms > 5000 {
            warnings.push(format!(
                "Invalid {} VAD speech_pad_ms: {}. Using default: 400",
                prefix, vad.speech_pad_ms
            ));
            vad.speech_pad_ms = 400;
        }
    }
    
    fn validate_overlay_config(overlay: &mut OverlayConfig, warnings: &mut Vec<String>, name: &str) {
        // Validate opacity (0.0 to 1.0)
        if overlay.opacity < 0.0 || overlay.opacity > 1.0 {
            warnings.push(format!(
                "Invalid {} overlay opacity: {}. Using default: 1.0",
                name, overlay.opacity
            ));
            overlay.opacity = 1.0;
        }
        
        // Validate ui_scaling (0.1 to 5.0)
        if overlay.ui_scaling < 0.1 || overlay.ui_scaling > 5.0 {
            warnings.push(format!(
                "Invalid {} overlay ui_scaling: {}. Using default: 1.0",
                name, overlay.ui_scaling
            ));
            overlay.ui_scaling = 1.0;
        }
        
        // Validate display_duration (1 to 60 seconds)
        if overlay.display_duration < 1 || overlay.display_duration > 60 {
            warnings.push(format!(
                "Invalid {} overlay display_duration: {}. Using default: 5",
                name, overlay.display_duration
            ));
            overlay.display_duration = 5;
        }
        
        // Validate fadeout_duration (0 to 10 seconds)
        if overlay.fadeout_duration > 10 {
            warnings.push(format!(
                "Invalid {} overlay fadeout_duration: {}. Using default: 2",
                name, overlay.fadeout_duration
            ));
            overlay.fadeout_duration = 2;
        }
        
        // Validate tracker
        if !["HMD", "LeftHand", "RightHand"].contains(&overlay.tracker.as_str()) {
            warnings.push(format!(
                "Invalid {} overlay tracker: {}. Using default: HMD",
                name, overlay.tracker
            ));
            overlay.tracker = "HMD".to_string();
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    
    #[test]
    fn test_validate_transparency() {
        let mut config = ConfigData::default();
        config.transparency = 150;
        
        let warnings = config.validate(None);
        assert_eq!(config.transparency, 100);
        assert!(!warnings.is_empty());
    }
    
    #[test]
    fn test_validate_device_selection() {
        let mut config = ConfigData::default();
        config.auto_mic_select = false;
        config.selected_mic_device = "NonExistentDevice".to_string();
        
        let devices = vec!["Device1".to_string(), "Device2".to_string()];
        let warnings = config.validate(Some(&devices));
        
        assert!(config.auto_mic_select);
        assert!(!warnings.is_empty());
    }
    
    #[test]
    fn test_validate_thresholds() {
        let mut config = ConfigData::default();
        config.mic_threshold = 20000;
        config.speaker_threshold = 20000;
        
        let warnings = config.validate(None);
        assert_eq!(config.mic_threshold, 300);
        assert_eq!(config.speaker_threshold, 300);
        assert!(warnings.len() >= 2);
    }
    
    #[test]
    fn test_validate_overlay_opacity() {
        let mut config = ConfigData::default();
        config.overlay_small_log_settings.opacity = 2.0;
        
        let warnings = config.validate(None);
        assert_eq!(config.overlay_small_log_settings.opacity, 1.0);
        assert!(!warnings.is_empty());
    }
    
    #[test]
    fn test_validate_ports() {
        let mut config = ConfigData::default();
        config.osc_port = 0;
        config.websocket_port = 0;
        
        let warnings = config.validate(None);
        assert_eq!(config.osc_port, 9000);
        assert_eq!(config.websocket_port, 2231);
        assert!(warnings.len() >= 2);
    }
}
