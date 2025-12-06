# Requirements Document

## Introduction

This document specifies the requirements for completely replacing the VRCT (VRChat Translator) Python backend (`src-python`) with a pure Rust implementation within the existing Tauri application structure (`src-tauri`). The goal is to eliminate all Python dependencies and consolidate the application into a single Rust binary, leveraging existing Rust crates to minimize custom code while maintaining all current functionality including real-time audio transcription, translation, OSC communication, overlay rendering, and WebSocket server capabilities. After migration, the `src-python` directory will be removed entirely.

## Glossary

- **VRCT**: VRChat Translator - the application being migrated
- **Backend**: The Python server process that handles audio processing, translation, and communication
- **Frontend**: The Tauri-based UI written in JavaScript/React
- **Tauri**: A framework for building desktop applications with web technologies and Rust
- **OSC**: Open Sound Control - a protocol for communication with VRChat
- **Whisper**: OpenAI's speech recognition model used for transcription
- **CTranslate2**: An optimized inference engine for Transformer models
- **WebSocket Server**: Real-time bidirectional communication server for external integrations
- **Overlay**: VR overlay system for displaying text in VRChat
- **Device Manager**: System for enumerating and managing audio input/output devices
- **Controller**: The main orchestration layer that handles UI requests and coordinates subsystems
- **Model**: The business logic layer containing translation, transcription, and processing logic
- **Config**: Configuration management system with JSON persistence
- **Mainloop**: The event loop that processes commands from the frontend

## Requirements

### Requirement 1

**User Story:** As a developer, I want to consolidate the Python backend into the Rust Tauri binary, so that the application has a single executable with reduced dependencies and improved performance.

#### Acceptance Criteria

1. WHEN the application starts THEN the Rust binary SHALL initialize all backend services without requiring Python or any Python dependencies
2. WHEN the Rust backend initializes THEN the system SHALL load configuration from the same JSON file format as the previous Python version
3. WHEN the application exits THEN the Rust binary SHALL cleanly shut down all services and persist configuration changes
4. WHEN comparing memory usage THEN the Rust implementation SHALL use less memory than the Python implementation's baseline
5. WHEN the Rust backend encounters initialization errors THEN the system SHALL log detailed error messages and gracefully degrade functionality

### Requirement 2

**User Story:** As a user, I want audio device enumeration and selection to work identically to the current Python implementation, so that I can continue using my existing audio setup without reconfiguration.

#### Acceptance Criteria

1. WHEN the system enumerates audio devices THEN the Rust implementation SHALL detect all microphone and speaker devices that the previous Python version detected
2. WHEN a user selects an audio device THEN the system SHALL persist the selection to the configuration file immediately
3. WHEN an audio device becomes unavailable THEN the system SHALL detect the change and notify the frontend within 2 seconds
4. WHEN the system supports loopback devices THEN the Rust implementation SHALL enumerate both loopback and regular input devices for speaker capture
5. WHEN automatic device selection is enabled THEN the system SHALL select the default system device and update the configuration

### Requirement 3

**User Story:** As a user, I want real-time audio transcription to work with the same accuracy and latency as the Python implementation, so that my VRChat conversations are transcribed reliably.

#### Acceptance Criteria

1. WHEN audio is captured from a microphone THEN the system SHALL apply the configured energy threshold and voice activity detection using Rust audio libraries
2. WHEN speech is detected THEN the system SHALL transcribe the audio using a Rust-based Whisper implementation within 500ms of speech completion
3. WHEN transcription completes THEN the system SHALL return the transcribed text and detected language to the frontend
4. WHEN the Whisper model is not loaded THEN the system SHALL download the model weights with progress reporting using Rust HTTP clients
5. WHEN transcription parameters change THEN the system SHALL apply the new parameters to subsequent transcriptions without restart

### Requirement 4

**User Story:** As a user, I want text translation to work with all supported translation engines, so that I can communicate across language barriers in VRChat.

#### Acceptance Criteria

1. WHEN translation is requested THEN the system SHALL support CTranslate2, DeepL, OpenAI, Gemini, LM Studio, Ollama, and Plamo translation engines using Rust implementations or Rust bindings
2. WHEN using CTranslate2 THEN the system SHALL load the model on the configured compute device (CPU, CUDA, or ZLUDA) using Rust bindings to CTranslate2
3. WHEN using API-based translators THEN the system SHALL validate API keys and handle rate limiting gracefully using Rust HTTP clients
4. WHEN translation fails THEN the system SHALL fall back to CTranslate2 and notify the frontend of the failure
5. WHEN VRAM overflow occurs THEN the system SHALL detect the error, disable translation, and notify the user

### Requirement 5

**User Story:** As a user, I want OSC communication with VRChat to work identically to the Python implementation, so that translated messages appear in my VRChat chatbox.

#### Acceptance Criteria

1. WHEN a message is sent THEN the system SHALL transmit the message via OSC to the configured IP address and port
2. WHEN typing indicators are enabled THEN the system SHALL send typing start/stop signals via OSC
3. WHEN VRC mic mute sync is enabled THEN the system SHALL receive OSC parameters and synchronize transcription state
4. WHEN OSC Query is available THEN the system SHALL use OSC Query for automatic parameter discovery
5. WHEN notification sounds are enabled THEN the system SHALL trigger VRChat sound effects via OSC parameters

### Requirement 6

**User Story:** As a user, I want the WebSocket server to continue working for external integrations, so that third-party tools can receive transcription and translation events.

#### Acceptance Criteria

1. WHEN the WebSocket server is enabled THEN the system SHALL listen on the configured host and port
2. WHEN a client connects THEN the system SHALL accept the connection and maintain it until the client disconnects
3. WHEN transcription or translation occurs THEN the system SHALL broadcast the event to all connected WebSocket clients
4. WHEN the WebSocket server configuration changes THEN the system SHALL restart the server with the new configuration
5. WHEN the server encounters errors THEN the system SHALL log the errors and attempt to restart the server

### Requirement 7

**User Story:** As a user, I want VR overlay rendering to work identically to the Python implementation, so that I can see transcriptions and translations in my VR headset.

#### Acceptance Criteria

1. WHEN overlay is enabled THEN the system SHALL render text overlays at the configured position and rotation
2. WHEN new text is displayed THEN the system SHALL apply the configured display duration and fadeout animation
3. WHEN overlay settings change THEN the system SHALL update the overlay appearance without restart
4. WHEN multiple overlay types are enabled THEN the system SHALL manage small log and large log overlays independently
5. WHEN overlay rendering fails THEN the system SHALL log the error and continue operating without overlays

### Requirement 8

**User Story:** As a developer, I want to use existing Rust crates for core functionality, so that the migration minimizes custom code and leverages battle-tested libraries.

#### Acceptance Criteria

1. WHEN implementing audio device enumeration THEN the system SHALL use the `cpal` crate for cross-platform audio device access
2. WHEN implementing audio recording THEN the system SHALL use `cpal` for audio stream capture
3. WHEN implementing Whisper inference THEN the system SHALL use `whisper-rs` or `candle` for model execution
4. WHEN implementing HTTP clients THEN the system SHALL use `reqwest` for API communication
5. WHEN implementing WebSocket server THEN the system SHALL use `tokio-tungstenite` or `axum` for WebSocket handling

### Requirement 9

**User Story:** As a user, I want configuration management to work identically to the Python implementation, so that my settings are preserved during the migration.

#### Acceptance Criteria

1. WHEN the system reads configuration THEN the Rust implementation SHALL parse the same JSON format as the previous Python version for backward compatibility
2. WHEN configuration changes THEN the system SHALL debounce writes and persist to disk after 2 seconds of inactivity using Rust async timers
3. WHEN configuration is invalid THEN the system SHALL use default values and log validation errors
4. WHEN the configuration file is missing THEN the system SHALL create a new file with default values
5. WHEN configuration includes device selections THEN the system SHALL validate that selected devices still exist

### Requirement 10

**User Story:** As a user, I want the command/response protocol between frontend and backend to remain unchanged, so that the UI continues to work without modifications.

#### Acceptance Criteria

1. WHEN the frontend sends a command THEN the Rust backend SHALL parse the JSON command with the same endpoint structure
2. WHEN the backend processes a command THEN the system SHALL return responses in the same JSON format as the Python version
3. WHEN the backend sends status updates THEN the system SHALL use the same endpoint paths as the Python version
4. WHEN errors occur THEN the system SHALL return error responses with the same structure as the Python version
5. WHEN the backend initializes THEN the system SHALL send initialization progress updates using the same format

### Requirement 11

**User Story:** As a user, I want transliteration (romaji/hiragana conversion) to work identically to the Python implementation, so that Japanese text is displayed in my preferred format.

#### Acceptance Criteria

1. WHEN Japanese text is transcribed THEN the system SHALL optionally convert it to hiragana using the configured rules
2. WHEN Japanese text is transcribed THEN the system SHALL optionally convert it to romaji (Hepburn romanization)
3. WHEN transliteration is enabled THEN the system SHALL apply context-aware conversion rules
4. WHEN transliteration settings change THEN the system SHALL apply the new settings to subsequent text
5. WHEN transliteration fails THEN the system SHALL return the original text without conversion

### Requirement 12

**User Story:** As a user, I want word filtering to work identically to the Python implementation, so that unwanted words are blocked from transcription output.

#### Acceptance Criteria

1. WHEN word filters are configured THEN the system SHALL load the filter list into a keyword processor
2. WHEN transcribed text contains filtered words THEN the system SHALL block the message and notify the frontend
3. WHEN the filter list changes THEN the system SHALL reload the keyword processor with the new filters
4. WHEN checking for filtered words THEN the system SHALL use case-insensitive matching
5. WHEN a message is blocked THEN the system SHALL log the detection event

### Requirement 13

**User Story:** As a user, I want automatic threshold detection for audio devices to work identically to the Python implementation, so that transcription adapts to my environment.

#### Acceptance Criteria

1. WHEN automatic threshold is enabled for microphone THEN the system SHALL dynamically adjust the energy threshold based on ambient noise
2. WHEN automatic threshold is enabled for speaker THEN the system SHALL dynamically adjust the energy threshold based on audio levels
3. WHEN threshold checking is enabled THEN the system SHALL report current energy levels to the frontend for visualization
4. WHEN energy levels are reported THEN the system SHALL send updates at least 10 times per second
5. WHEN automatic threshold detects silence THEN the system SHALL lower the threshold to improve sensitivity

### Requirement 14

**User Story:** As a developer, I want the Rust implementation to support ZLUDA for AMD GPU acceleration, so that users with AMD GPUs can benefit from hardware acceleration.

#### Acceptance Criteria

1. WHEN ZLUDA is detected THEN the system SHALL enumerate ZLUDA devices as available compute devices
2. WHEN a ZLUDA device is selected THEN the system SHALL attempt to load models on the ZLUDA device
3. WHEN ZLUDA initialization fails THEN the system SHALL fall back to CPU and notify the user
4. WHEN ZLUDA runtime errors occur THEN the system SHALL detect the error and fall back to CPU
5. WHEN ZLUDA is not installed THEN the system SHALL operate normally without ZLUDA support

### Requirement 15

**User Story:** As a user, I want model weight downloading to work identically to the Python implementation, so that I can download Whisper and CTranslate2 models from the UI.

#### Acceptance Criteria

1. WHEN a model download is requested THEN the system SHALL download the model weights with progress reporting
2. WHEN download progress updates THEN the system SHALL report percentage completion to the frontend
3. WHEN a download completes THEN the system SHALL verify the model integrity and notify the frontend
4. WHEN a download fails THEN the system SHALL report the error and allow retry
5. WHEN multiple downloads are requested THEN the system SHALL queue downloads and process them sequentially

### Requirement 16

**User Story:** As a user, I want logging functionality to work identically to the Python implementation, so that I can review transcription and translation history.

#### Acceptance Criteria

1. WHEN logging is enabled THEN the system SHALL write transcription and translation events to timestamped log files
2. WHEN a log entry is created THEN the system SHALL include timestamp, message type, original text, and translations
3. WHEN the log directory is requested THEN the system SHALL open the logs folder in the system file explorer
4. WHEN logging is disabled THEN the system SHALL stop writing to log files immediately
5. WHEN log files accumulate THEN the system SHALL not automatically delete old logs

### Requirement 17

**User Story:** As a developer, I want the Rust implementation to use async/await for I/O operations, so that the backend remains responsive during network and disk operations.

#### Acceptance Criteria

1. WHEN performing network requests THEN the system SHALL use async HTTP clients to avoid blocking
2. WHEN reading or writing configuration THEN the system SHALL use async file I/O where beneficial
3. WHEN processing audio streams THEN the system SHALL use async streams for audio data
4. WHEN handling WebSocket connections THEN the system SHALL use async WebSocket handlers
5. WHEN coordinating multiple async tasks THEN the system SHALL use Tokio runtime for task management

### Requirement 18

**User Story:** As a user, I want software updates to work identically to the Python implementation, so that I can update VRCT from within the application.

#### Acceptance Criteria

1. WHEN checking for updates THEN the system SHALL query the GitHub API for the latest release version
2. WHEN a new version is available THEN the system SHALL notify the frontend with version information
3. WHEN an update is requested THEN the system SHALL download the updater executable
4. WHEN the updater is downloaded THEN the system SHALL launch the updater and exit the application
5. WHEN update checking fails THEN the system SHALL log the error and continue operating

### Requirement 19

**User Story:** As a user, I want message formatting to work identically to the Python implementation, so that messages sent to VRChat follow my configured format.

#### Acceptance Criteria

1. WHEN formatting a sent message THEN the system SHALL apply the configured prefix, suffix, and separator
2. WHEN formatting a received message THEN the system SHALL apply the configured format for received messages
3. WHEN translation-first mode is enabled THEN the system SHALL place translations before the original message
4. WHEN send-only-translated is enabled THEN the system SHALL omit the original message from the output
5. WHEN formatting fails THEN the system SHALL fall back to a simple format with original and translation

### Requirement 20

**User Story:** As a developer, I want comprehensive error handling throughout the Rust implementation, so that errors are logged and the application remains stable.

#### Acceptance Criteria

1. WHEN any subsystem encounters an error THEN the system SHALL log the error with context information
2. WHEN critical errors occur THEN the system SHALL attempt graceful degradation rather than crashing
3. WHEN errors are reported to the frontend THEN the system SHALL include error type and user-friendly messages
4. WHEN errors are logged THEN the system SHALL include stack traces in debug builds
5. WHEN repeated errors occur THEN the system SHALL implement backoff strategies to prevent error storms
