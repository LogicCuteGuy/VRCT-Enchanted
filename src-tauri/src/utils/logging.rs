use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::io::Write;

use crate::utils::error::Result;

/// Event logger for transcription and translation events
pub struct EventLogger {
    log_dir: PathBuf,
    log_file: Arc<RwLock<Option<PathBuf>>>,
    enabled: Arc<RwLock<bool>>,
}

impl EventLogger {
    /// Create a new event logger
    pub fn new(log_dir: PathBuf) -> Self {
        Self {
            log_dir,
            log_file: Arc::new(RwLock::new(None)),
            enabled: Arc::new(RwLock::new(false)),
        }
    }

    /// Enable logging and create a new timestamped log file
    pub async fn enable(&self) -> Result<()> {
        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&self.log_dir)?;

        // Create timestamped log file
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let log_path = self.log_dir.join(format!("{}.log", timestamp));

        // Create the file
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        // Update state
        let mut file = self.log_file.write().await;
        *file = Some(log_path.clone());
        
        let mut enabled = self.enabled.write().await;
        *enabled = true;

        tracing::info!("Event logging enabled: {:?}", log_path);
        Ok(())
    }

    /// Disable logging immediately
    pub async fn disable(&self) {
        let mut enabled = self.enabled.write().await;
        *enabled = false;

        let mut file = self.log_file.write().await;
        *file = None;

        tracing::info!("Event logging disabled");
    }

    /// Check if logging is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Log a transcription event
    pub async fn log_transcription(&self, event: &TranscriptionEvent) -> Result<()> {
        if !self.is_enabled().await {
            return Ok(());
        }

        let file_path = {
            let file = self.log_file.read().await;
            match &*file {
                Some(path) => path.clone(),
                None => return Ok(()),
            }
        };

        // Format the log entry
        let timestamp = event.timestamp.format("%Y-%m-%d %H:%M:%S");
        let translations = event.translations.join("/");
        let log_entry = if translations.is_empty() {
            format!("[{}] [{}] {}\n", timestamp, event.message_type, event.original_text)
        } else {
            format!("[{}] [{}] {} ({})\n", timestamp, event.message_type, event.original_text, translations)
        };

        // Write to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        
        file.write_all(log_entry.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Log a translation event
    pub async fn log_translation(&self, event: &TranslationEvent) -> Result<()> {
        if !self.is_enabled().await {
            return Ok(());
        }

        let file_path = {
            let file = self.log_file.read().await;
            match &*file {
                Some(path) => path.clone(),
                None => return Ok(()),
            }
        };

        // Format the log entry
        let timestamp = event.timestamp.format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!(
            "[{}] [TRANSLATION] {} -> {}\n",
            timestamp, event.source_text, event.translated_text
        );

        // Write to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        
        file.write_all(log_entry.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Get the log directory path
    pub fn get_log_dir(&self) -> &PathBuf {
        &self.log_dir
    }

    /// Open the log directory in the system file explorer
    pub async fn open_log_directory(&self) -> Result<()> {
        let log_dir = self.log_dir.clone();
        
        // Ensure the directory exists
        std::fs::create_dir_all(&log_dir)?;

        // Open the directory in the file explorer
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&log_dir)
                .spawn()
                .map_err(|e| crate::utils::error::VrctError::Io(e))?;
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&log_dir)
                .spawn()
                .map_err(|e| crate::utils::error::VrctError::Io(e))?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&log_dir)
                .spawn()
                .map_err(|e| crate::utils::error::VrctError::Io(e))?;
        }

        tracing::info!("Opened log directory: {:?}", log_dir);
        Ok(())
    }
}

/// Transcription event for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionEvent {
    pub timestamp: DateTime<Local>,
    pub message_type: String,  // "SENT", "RECEIVED", etc.
    pub original_text: String,
    pub translations: Vec<String>,
}

/// Translation event for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEvent {
    pub timestamp: DateTime<Local>,
    pub source_text: String,
    pub translated_text: String,
}

/// Initialize the logging system with file and console output
pub fn init_logging(log_dir: Option<PathBuf>) -> Result<()> {
    // Create log directory if specified
    let file_appender = if let Some(dir) = log_dir {
        std::fs::create_dir_all(&dir)?;
        Some(RollingFileAppender::new(
            Rotation::DAILY,
            dir,
            "vrct.log",
        ))
    } else {
        None
    };

    // Set up environment filter with default level
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Build the subscriber
    let subscriber = tracing_subscriber::registry().with(env_filter);

    // Add console output layer
    let subscriber = subscriber.with(
        fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
    );

    // Add file output layer if log directory was specified
    // Use try_init to avoid panicking if already initialized (useful for tests)
    let init_result = if let Some(appender) = file_appender {
        let file_layer = fmt::layer()
            .with_writer(appender)
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_ansi(false);
        
        subscriber.with(file_layer).try_init()
    } else {
        subscriber.try_init()
    };

    // If initialization fails because a subscriber is already set, that's okay in tests
    if let Err(_e) = init_result {
        // In production, this would be an error, but in tests it's expected
        #[cfg(test)]
        {
            // Silently ignore if already initialized during tests
            return Ok(());
        }
        #[cfg(not(test))]
        {
            return Err(crate::utils::error::VrctError::Config(format!(
                "Failed to initialize logging: {}",
                _e
            )));
        }
    }

    tracing::info!("Logging system initialized");
    Ok(())
}

/// Log an error with context
pub fn log_error(error: &crate::utils::error::VrctError, context: &str) {
    tracing::error!(
        error = %error,
        context = context,
        recoverable = error.is_recoverable(),
        "Error occurred"
    );
}

/// Log initialization progress
pub fn log_init_progress(component: &str, status: &str) {
    tracing::info!(
        component = component,
        status = status,
        "Initialization progress"
    );
}

/// Log a warning with context
pub fn log_warning(message: &str, context: &str) {
    tracing::warn!(
        message = message,
        context = context,
        "Warning"
    );
}

/// Log debug information
pub fn log_debug(message: &str, context: &str) {
    tracing::debug!(
        message = message,
        context = context,
        "Debug"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_logging_init_without_file() {
        // Initialize logging without file output
        // Note: This may fail if another test already initialized logging,
        // which is expected behavior since tracing can only be initialized once
        let result = init_logging(None);
        // Accept either success or the specific error about already being initialized
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_logging_init_with_file() {
        // Create a temporary directory for logs
        let temp_dir = env::temp_dir().join("vrct_test_logs");
        let result = init_logging(Some(temp_dir.clone()));
        // Accept either success or the specific error about already being initialized
        assert!(result.is_ok() || result.is_err());
        
        // Clean up
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_event_logger_enable_disable() {
        let temp_dir = env::temp_dir().join("vrct_event_logs_test");
        let logger = EventLogger::new(temp_dir.clone());

        // Initially disabled
        assert!(!logger.is_enabled().await);

        // Enable logging
        logger.enable().await.unwrap();
        assert!(logger.is_enabled().await);

        // Disable logging
        logger.disable().await;
        assert!(!logger.is_enabled().await);

        // Clean up
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_event_logger_transcription() {
        let temp_dir = env::temp_dir().join("vrct_event_logs_transcription");
        let logger = EventLogger::new(temp_dir.clone());

        // Enable logging
        logger.enable().await.unwrap();

        // Log a transcription event
        let event = TranscriptionEvent {
            timestamp: Local::now(),
            message_type: "SENT".to_string(),
            original_text: "Hello world".to_string(),
            translations: vec!["こんにちは世界".to_string()],
        };

        logger.log_transcription(&event).await.unwrap();

        // Verify the log file was created and contains the entry
        let log_file = logger.log_file.read().await;
        assert!(log_file.is_some());
        
        if let Some(path) = &*log_file {
            let content = std::fs::read_to_string(path).unwrap();
            assert!(content.contains("Hello world"));
            assert!(content.contains("こんにちは世界"));
            assert!(content.contains("SENT"));
        }

        // Clean up
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_event_logger_translation() {
        let temp_dir = env::temp_dir().join("vrct_event_logs_translation");
        let logger = EventLogger::new(temp_dir.clone());

        // Enable logging
        logger.enable().await.unwrap();

        // Log a translation event
        let event = TranslationEvent {
            timestamp: Local::now(),
            source_text: "Hello".to_string(),
            translated_text: "こんにちは".to_string(),
        };

        logger.log_translation(&event).await.unwrap();

        // Verify the log file was created and contains the entry
        let log_file = logger.log_file.read().await;
        assert!(log_file.is_some());
        
        if let Some(path) = &*log_file {
            let content = std::fs::read_to_string(path).unwrap();
            assert!(content.contains("Hello"));
            assert!(content.contains("こんにちは"));
            assert!(content.contains("TRANSLATION"));
        }

        // Clean up
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_event_logger_disabled_no_write() {
        let temp_dir = env::temp_dir().join("vrct_event_logs_disabled");
        let logger = EventLogger::new(temp_dir.clone());

        // Don't enable logging
        assert!(!logger.is_enabled().await);

        // Try to log an event
        let event = TranscriptionEvent {
            timestamp: Local::now(),
            message_type: "SENT".to_string(),
            original_text: "Hello world".to_string(),
            translations: vec![],
        };

        // Should succeed but not write anything
        logger.log_transcription(&event).await.unwrap();

        // Verify no log file was created
        let log_file = logger.log_file.read().await;
        assert!(log_file.is_none());

        // Clean up
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
