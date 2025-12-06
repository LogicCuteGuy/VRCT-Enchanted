use tauri::Manager;
use tracing::info;

// Module declarations
pub mod utils;
pub mod config;
pub mod audio;
pub mod transcription;
pub mod translation;
pub mod transliteration;
pub mod communication;
pub mod overlay;
pub mod models;
pub mod controller;
pub mod commands;
pub mod events;

// Re-export commonly used types
pub use utils::error::{Result, VrctError};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging (without file output for now, will be configured later)
    let _ = utils::logging::init_logging(None);
    
    info!("Starting VRCT application");
    
    tauri::Builder::default()
        .setup(|app| {
            let _main_window = app.get_webview_window("main").unwrap();  // `main_window` is declared here for all builds

            #[cfg(debug_assertions)]
            { _main_window.open_devtools(); }
            
            // Set global app handle for event emission
            events::set_global_app_handle(app.handle().clone());
            
            info!("Tauri setup complete");

            Ok(())
        })
        .manage(commands::create_controller_state())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Legacy commands
            get_font_list,
            download_zip_asset,
            
            // Main command handler
            commands::handle_command,
            
            // Main window commands
            commands::enable_translation,
            commands::disable_translation,
            commands::enable_transcription_send,
            commands::disable_transcription_send,
            commands::enable_transcription_receive,
            commands::disable_transcription_receive,
            commands::send_message_box,
            
            // Audio device commands
            commands::get_audio_devices,
            commands::set_mic_device,
            commands::set_speaker_device,
            commands::set_mic_threshold,
            commands::set_speaker_threshold,
            commands::enable_auto_mic_threshold,
            commands::enable_auto_speaker_threshold,
            
            // Transcription commands
            commands::start_transcription,
            commands::stop_transcription,
            commands::set_transcription_engine,
            commands::set_whisper_model,
            commands::download_whisper_model,
            
            // Translation commands
            commands::translate_text,
            commands::set_translation_engine,
            commands::set_source_language,
            commands::set_target_language,
            
            // Configuration commands
            commands::get_config,
            commands::set_config,
            commands::set_transparency,
            commands::set_ui_language,
            commands::set_ui_scaling,
            
            // OSC commands
            commands::set_osc_settings,
            commands::enable_osc_send,
            commands::enable_mic_mute_sync,
            
            // WebSocket commands
            commands::set_websocket_settings,
            commands::enable_websocket_server,
            
            // Overlay commands
            commands::enable_small_log_overlay,
            commands::enable_large_log_overlay,
            commands::set_overlay_settings,
            
            // Word filter commands
            commands::set_word_filter,
            
            // Logging commands
            commands::enable_logging,
            commands::open_logs_folder,
            
            // System commands
            commands::shutdown,
            commands::get_version,
            
            // Typing indicator commands
            commands::start_typing,
            commands::stop_typing,
            
            // Threshold check commands
            commands::enable_mic_threshold_check,
            commands::disable_mic_threshold_check,
            commands::enable_speaker_threshold_check,
            commands::disable_speaker_threshold_check,
            
            // Folder and utility commands
            commands::open_config_folder,
            commands::swap_languages,
            commands::get_zluda_info,
            
            // LLM connection check commands
            commands::check_lmstudio_connection,
            commands::check_ollama_connection,
            
            // Initialization commands
            commands::initialize_controller,
            commands::is_controller_initialized,
            
            // Update commands
            commands::check_for_updates,
            commands::download_update,
            
            // Profiling commands
            commands::get_profiling_report,
            commands::run_benchmarks,
            commands::reset_profiling,
            commands::set_profiling_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


use font_kit::{source::SystemSource};
use std::collections::HashSet;

#[tauri::command]
async fn get_font_list() -> Vec<String> {
    let source = SystemSource::new();
    let mut font_families = HashSet::new();

    if let Ok(fonts) = source.all_fonts() {
        for font in fonts {
            if let Ok(info) = font.load() {
                font_families.insert(info.family_name().to_string());
            }
        }
    }

    font_families.into_iter().collect()
}


use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

#[tauri::command]
async fn download_zip_asset(url: String) -> std::result::Result<String, String> {
    use reqwest;

    let client = reqwest::Client::new();
    let resp = client.get(&url)
        .header("Accept", "application/octet-stream")
        .send()
        .await.map_err(|e| format!("Request error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let bytes = resp.bytes().await.map_err(|e| format!("Reading bytes error: {}", e))?;

    Ok(BASE64.encode(&bytes))
}
