# Design Document

## Overview

This design document outlines the architecture for migrating VRCT from a Python backend to a pure Rust implementation within the Tauri framework. The design emphasizes leveraging existing, well-maintained Rust crates to minimize custom code while maintaining full feature parity with the Python implementation. The architecture follows a modular design with clear separation of concerns between audio processing, transcription, translation, communication protocols, and configuration management.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Tauri Frontend (JS/React)                │
│                  (Existing - No Changes)                     │
└────────────────────────┬────────────────────────────────────┘
                         │ IPC Commands/Events
                         │
┌────────────────────────▼────────────────────────────────────┐
│                   Tauri Command Layer                        │
│              (Rust - New Implementation)                     │
│  - Command routing and validation                           │
│  - Response formatting                                       │
│  - Event emission to frontend                               │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                    Controller Layer                          │
│              (Rust - New Implementation)                     │
│  - Orchestrates subsystems                                   │
│  - Manages application state                                 │
│  - Coordinates async tasks                                   │
└─────┬──────┬──────┬──────┬──────┬──────┬──────┬────────────┘
      │      │      │      │      │      │      │
      ▼      ▼      ▼      ▼      ▼      ▼      ▼
   ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐
   │Audio│ │Trans│ │Trans│ │OSC │ │Web │ │Over│ │Config│
   │Mgr  │ │cribe│ │late │ │    │ │Socket│ │lay │ │      │
   └────┘ └────┘ └────┘ └────┘ └────┘ └────┘ └────┘
```

### Module Structure

The Rust implementation will be organized into the following modules within `src-tauri/src/`:

```
src-tauri/src/
├── lib.rs                    # Main entry point, Tauri setup
├── main.rs                   # Binary entry point
├── commands.rs               # Tauri command handlers
├── controller.rs             # Main controller orchestration
├── config/
│   ├── mod.rs               # Configuration management
│   ├── types.rs             # Configuration data structures
│   └── persistence.rs       # JSON serialization/deserialization
├── audio/
│   ├── mod.rs               # Audio subsystem
│   ├── device_manager.rs    # Device enumeration (cpal)
│   ├── recorder.rs          # Audio recording streams
│   └── energy.rs            # Energy/threshold detection
├── transcription/
│   ├── mod.rs               # Transcription subsystem
│   ├── whisper.rs           # Whisper model integration
│   ├── languages.rs         # Language definitions
│   └── vad.rs               # Voice activity detection
├── translation/
│   ├── mod.rs               # Translation subsystem
│   ├── ctranslate2.rs       # CTranslate2 bindings
│   ├── api_clients.rs       # API-based translators
│   ├── languages.rs         # Language mappings
│   └── engines/
│       ├── deepl.rs
│       ├── openai.rs
│       ├── gemini.rs
│       ├── lmstudio.rs
│       ├── ollama.rs
│       └── plamo.rs
├── transliteration/
│   ├── mod.rs               # Transliteration subsystem
│   ├── kana.rs              # Kana conversion
│   └── romaji.rs            # Romaji conversion
├── communication/
│   ├── mod.rs               # Communication subsystem
│   ├── osc.rs               # OSC protocol (rosc crate)
│   └── websocket.rs         # WebSocket server (axum/tokio-tungstenite)
├── overlay/
│   ├── mod.rs               # VR overlay subsystem
│   ├── renderer.rs          # Text rendering
│   └── openvr.rs            # OpenVR integration
├── utils/
│   ├── mod.rs               # Utility functions
│   ├── logging.rs           # Logging setup
│   ├── error.rs             # Error types
│   └── keyword_filter.rs    # Word filtering (aho-corasick)
└── models/
    ├── mod.rs               # Shared data models
    ├── message.rs           # Message types
    └── device.rs            # Device types
```

## Components and Interfaces

### 1. Configuration Management (`config/`)

**Purpose:** Manage application configuration with JSON persistence and debounced writes.

**Key Crates:**
- `serde` + `serde_json` - JSON serialization
- `tokio::time` - Debounce timers

**Interface:**
```rust
pub struct Config {
    // Singleton instance with Arc<RwLock<ConfigData>>
}

impl Config {
    pub async fn load() -> Result<Self>;
    pub async fn save(&self) -> Result<()>;
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    pub async fn set<T: Serialize>(&self, key: &str, value: T);
    // Debounced save - actual write happens 2s after last change
    pub async fn schedule_save(&self);
}
```

**Design Notes:**
- Use `Arc<RwLock<ConfigData>>` for thread-safe access
- Implement debouncing with `tokio::time::sleep` and cancellation tokens
- Validate configuration on load with serde validation
- Maintain backward compatibility with Python JSON format

### 2. Audio Device Management (`audio/device_manager.rs`)

**Purpose:** Enumerate and manage audio input/output devices.

**Key Crates:**
- `cpal` - Cross-platform audio library

**Interface:**
```rust
pub struct DeviceManager {
    host: cpal::Host,
}

impl DeviceManager {
    pub fn new() -> Self;
    pub fn enumerate_input_devices(&self) -> Vec<AudioDevice>;
    pub fn enumerate_output_devices(&self) -> Vec<AudioDevice>;
    pub fn get_default_input(&self) -> Option<AudioDevice>;
    pub fn get_default_output(&self) -> Option<AudioDevice>;
    pub fn get_device_by_name(&self, name: &str) -> Option<AudioDevice>;
}

pub struct AudioDevice {
    pub name: String,
    pub host_name: String,
    pub index: usize,
    pub is_loopback: bool,
    pub sample_rate: u32,
    pub channels: u16,
}
```

**Design Notes:**
- Use `cpal::Host::default()` for primary audio backend
- Enumerate both input and loopback devices for speaker capture
- Cache device list and refresh on demand
- Handle device disconnection gracefully

### 3. Audio Recording (`audio/recorder.rs`)

**Purpose:** Capture audio streams with energy-based voice activity detection.

**Key Crates:**
- `cpal` - Audio stream capture
- `ringbuf` - Lock-free ring buffer for audio data

**Interface:**
```rust
pub struct AudioRecorder {
    device: AudioDevice,
    stream: Option<cpal::Stream>,
    buffer: Arc<RingBuffer<f32>>,
    energy_threshold: Arc<AtomicU32>,
    vad_enabled: bool,
}

impl AudioRecorder {
    pub fn new(device: AudioDevice, config: RecorderConfig) -> Self;
    pub async fn start(&mut self) -> Result<()>;
    pub async fn stop(&mut self) -> Result<()>;
    pub fn set_energy_threshold(&self, threshold: f32);
    pub fn get_current_energy(&self) -> f32;
    pub async fn read_audio(&self) -> Option<Vec<f32>>;
}
```

**Design Notes:**
- Use `cpal::Stream` for continuous audio capture
- Implement energy calculation using RMS (root mean square)
- Support automatic threshold adjustment based on ambient noise
- Use ring buffer for lock-free audio data transfer

### 4. Transcription (`transcription/whisper.rs`)

**Purpose:** Transcribe audio using Whisper models.

**Key Crates:**
- `whisper-rs` - Rust bindings for whisper.cpp (preferred)
- OR `candle` + `candle-transformers` - Pure Rust ML framework
- `hf-hub` - Download models from Hugging Face

**Interface:**
```rust
pub struct WhisperTranscriber {
    model: WhisperModel,
    config: TranscriptionConfig,
}

impl WhisperTranscriber {
    pub async fn new(model_path: &Path, device: ComputeDevice) -> Result<Self>;
    pub async fn transcribe(&self, audio: &[f32]) -> Result<TranscriptionResult>;
    pub async fn download_model(model_type: &str, progress: impl Fn(f32)) -> Result<PathBuf>;
}

pub struct TranscriptionResult {
    pub text: String,
    pub language: String,
    pub confidence: f32,
}
```

**Design Notes:**
- Prefer `whisper-rs` for performance (uses whisper.cpp)
- Fall back to `candle` if whisper-rs unavailable
- Support CPU, CUDA, and ZLUDA compute devices
- Implement model caching and lazy loading
- Handle VRAM overflow with graceful fallback to CPU

### 5. Translation (`translation/`)

**Purpose:** Translate text using multiple translation engines.

**Key Crates:**
- `ctranslate2-rs` - Rust bindings for CTranslate2 (may need to create)
- `reqwest` - HTTP client for API-based translators
- `tokio` - Async runtime

**Interface:**
```rust
pub trait Translator: Send + Sync {
    async fn translate(&self, text: &str, source: &str, target: &str) -> Result<String>;
    async fn validate_config(&self) -> Result<bool>;
}

pub struct TranslationManager {
    engines: HashMap<String, Box<dyn Translator>>,
    active_engine: String,
}

impl TranslationManager {
    pub fn new() -> Self;
    pub fn register_engine(&mut self, name: String, engine: Box<dyn Translator>);
    pub async fn translate(&self, text: &str, source: &str, target: &str) -> Result<String>;
    pub async fn set_active_engine(&mut self, name: &str) -> Result<()>;
}
```

**CTranslate2 Integration:**
```rust
pub struct CTranslate2Engine {
    model: CTranslate2Model,
    device: ComputeDevice,
}

impl Translator for CTranslate2Engine {
    async fn translate(&self, text: &str, source: &str, target: &str) -> Result<String> {
        // Use FFI bindings to CTranslate2 C++ library
        // OR use pure Rust implementation if available
    }
}
```

**API-Based Translators:**
```rust
pub struct DeepLTranslator {
    client: reqwest::Client,
    api_key: String,
}

pub struct OpenAITranslator {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

// Similar for Gemini, LMStudio, Ollama, Plamo
```

**Design Notes:**
- Create Rust bindings for CTranslate2 if none exist
- Use trait objects for polymorphic translator interface
- Implement rate limiting and retry logic for API clients
- Cache translations to reduce API calls
- Handle VRAM errors and fall back to CPU

### 6. OSC Communication (`communication/osc.rs`)

**Purpose:** Send and receive OSC messages for VRChat integration.

**Key Crates:**
- `rosc` - OSC protocol implementation

**Interface:**
```rust
pub struct OscHandler {
    sender: OscSender,
    receiver: Option<OscReceiver>,
    ip_address: String,
    port: u16,
}

impl OscHandler {
    pub fn new(ip: &str, port: u16) -> Self;
    pub async fn send_message(&self, message: &str, notification: bool) -> Result<()>;
    pub async fn send_typing(&self, is_typing: bool) -> Result<()>;
    pub async fn start_receiver(&mut self, callback: impl Fn(OscMessage)) -> Result<()>;
    pub async fn stop_receiver(&mut self);
}
```

**Design Notes:**
- Use `rosc::OscPacket` for message construction
- Support OSC Query for automatic parameter discovery
- Implement VRC mic mute sync by listening to OSC parameters
- Handle network errors gracefully

### 7. WebSocket Server (`communication/websocket.rs`)

**Purpose:** Provide WebSocket API for external integrations.

**Key Crates:**
- `axum` - Web framework with WebSocket support
- `tokio-tungstenite` - WebSocket implementation

**Interface:**
```rust
pub struct WebSocketServer {
    addr: SocketAddr,
    clients: Arc<RwLock<Vec<WebSocketClient>>>,
    server_handle: Option<JoinHandle<()>>,
}

impl WebSocketServer {
    pub async fn start(&mut self, addr: SocketAddr) -> Result<()>;
    pub async fn stop(&mut self);
    pub async fn broadcast(&self, message: &WebSocketMessage);
    pub fn is_alive(&self) -> bool;
}

pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
}
```

**Design Notes:**
- Use `axum::extract::ws` for WebSocket handling
- Maintain list of connected clients
- Broadcast transcription/translation events to all clients
- Handle client disconnections gracefully
- Support server restart on configuration change

### 8. VR Overlay (`overlay/`)

**Purpose:** Render text overlays in VR using OpenVR.

**Key Crates:**
- `openvr` - OpenVR bindings
- `image` - Image manipulation
- `rusttype` or `fontdue` - Font rendering

**Interface:**
```rust
pub struct OverlayManager {
    system: Option<openvr::System>,
    overlays: HashMap<String, Overlay>,
}

impl OverlayManager {
    pub fn new() -> Result<Self>;
    pub async fn create_overlay(&mut self, name: &str, config: OverlayConfig) -> Result<()>;
    pub async fn update_overlay(&self, name: &str, text: &str) -> Result<()>;
    pub async fn set_overlay_position(&self, name: &str, transform: Transform) -> Result<()>;
}

pub struct OverlayConfig {
    pub tracker: Tracker,
    pub position: Vec3,
    pub rotation: Vec3,
    pub opacity: f32,
    pub ui_scaling: f32,
    pub display_duration: u32,
    pub fadeout_duration: u32,
}
```

**Design Notes:**
- Initialize OpenVR system on first overlay creation
- Render text to image using font rendering library
- Upload image to OpenVR overlay
- Support multiple overlay types (small log, large log)
- Handle OpenVR initialization failures gracefully

### 9. Transliteration (`transliteration/`)

**Purpose:** Convert Japanese text to hiragana and romaji.

**Key Crates:**
- `wana_kana` - Kana/romaji conversion (if available)
- OR custom implementation using Unicode tables

**Interface:**
```rust
pub struct Transliterator {
    context_rules: Vec<ContextRule>,
}

impl Transliterator {
    pub fn new() -> Self;
    pub fn to_hiragana(&self, text: &str) -> String;
    pub fn to_romaji(&self, text: &str, use_macron: bool) -> String;
    pub fn analyze(&self, text: &str) -> Vec<TransliterationSegment>;
}

pub struct TransliterationSegment {
    pub original: String,
    pub hiragana: Option<String>,
    pub romaji: Option<String>,
}
```

**Design Notes:**
- Implement context-aware conversion rules
- Support Hepburn romanization
- Handle special cases (long vowels, particles)
- Cache conversion results for performance

### 10. Controller (`controller.rs`)

**Purpose:** Orchestrate all subsystems and manage application state.

**Key Crates:**
- `tokio` - Async runtime
- `tokio::sync` - Channels and synchronization primitives

**Interface:**
```rust
pub struct Controller {
    config: Arc<Config>,
    audio_manager: Arc<DeviceManager>,
    transcriber: Arc<WhisperTranscriber>,
    translator: Arc<TranslationManager>,
    osc_handler: Arc<OscHandler>,
    websocket_server: Arc<WebSocketServer>,
    overlay_manager: Arc<OverlayManager>,
    state: Arc<RwLock<AppState>>,
}

impl Controller {
    pub async fn new() -> Result<Self>;
    pub async fn initialize(&self) -> Result<()>;
    pub async fn shutdown(&self) -> Result<()>;
    
    // Command handlers
    pub async fn handle_command(&self, cmd: Command) -> Result<Response>;
    pub async fn start_transcription(&self, device: &str) -> Result<()>;
    pub async fn stop_transcription(&self) -> Result<()>;
    pub async fn translate_text(&self, text: &str) -> Result<String>;
    // ... many more command handlers
}
```

**Design Notes:**
- Use `Arc` for shared ownership across async tasks
- Use `RwLock` for state that needs concurrent read access
- Use channels for communication between subsystems
- Implement graceful shutdown with cancellation tokens
- Handle errors at controller level and report to frontend

## Data Models

### Configuration Data

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    // UI Settings
    pub transparency: u8,
    pub ui_scaling: u8,
    pub ui_language: String,
    pub font_family: String,
    
    // Audio Settings
    pub selected_mic_host: String,
    pub selected_mic_device: String,
    pub mic_threshold: u32,
    pub mic_automatic_threshold: bool,
    pub selected_speaker_host: String,
    pub selected_speaker_device: String,
    pub speaker_threshold: u32,
    
    // Transcription Settings
    pub selected_transcription_engine: String,
    pub whisper_weight_type: String,
    pub selected_transcription_compute_device: ComputeDevice,
    pub selected_transcription_compute_type: String,
    
    // Translation Settings
    pub selected_translation_engines: HashMap<String, String>,
    pub ctranslate2_weight_type: String,
    pub selected_translation_compute_device: ComputeDevice,
    pub selected_your_languages: HashMap<String, HashMap<String, LanguageConfig>>,
    pub selected_target_languages: HashMap<String, HashMap<String, LanguageConfig>>,
    
    // OSC Settings
    pub osc_ip_address: String,
    pub osc_port: u16,
    pub send_message_to_vrc: bool,
    pub vrc_mic_mute_sync: bool,
    
    // WebSocket Settings
    pub websocket_host: String,
    pub websocket_port: u16,
    pub websocket_server_enabled: bool,
    
    // Overlay Settings
    pub overlay_small_log: bool,
    pub overlay_large_log: bool,
    pub overlay_small_log_settings: OverlayConfig,
    pub overlay_large_log_settings: OverlayConfig,
    
    // Other Settings
    pub logger_feature: bool,
    pub mic_word_filter: Vec<String>,
    pub convert_message_to_romaji: bool,
    pub convert_message_to_hiragana: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeDevice {
    pub device: String,  // "cpu", "cuda", "zluda"
    pub device_index: usize,
    pub compute_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub language: String,
    pub country: String,
    pub enable: bool,
}
```

### Message Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionMessage {
    pub original: MessageContent,
    pub translations: Vec<MessageContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    pub message: String,
    pub transliteration: Vec<TransliterationSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    // Main Window
    EnableTranslation,
    DisableTranslation,
    EnableTranscriptionSend,
    DisableTranscriptionSend,
    SendMessageBox { message: String },
    
    // Config Window
    SetTransparency { value: u8 },
    SetMicDevice { host: String, device: String },
    SetTranslationEngine { engine: String },
    
    // ... many more commands
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub endpoint: String,
    pub result: serde_json::Value,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum VrctError {
    #[error("Audio device error: {0}")]
    AudioDevice(String),
    
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
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, VrctError>;
```

### Error Handling Strategy

1. **Graceful Degradation:** When a subsystem fails, disable that feature but keep the application running
2. **Error Reporting:** Report errors to frontend with user-friendly messages
3. **Logging:** Log all errors with context using `tracing` crate
4. **Retry Logic:** Implement exponential backoff for transient failures
5. **Fallback Mechanisms:** 
   - ZLUDA → CUDA → CPU for compute devices
   - API translator → CTranslate2 for translation
   - Automatic threshold → Manual threshold for audio

## Testing Strategy

### Unit Tests

- Test each module independently with mock dependencies
- Test configuration serialization/deserialization
- Test audio energy calculation algorithms
- Test transliteration conversion rules
- Test message formatting logic

### Integration Tests

- Test audio device enumeration on real hardware
- Test Whisper transcription with sample audio files
- Test translation with all supported engines
- Test OSC message sending/receiving
- Test WebSocket server with real clients
- Test configuration persistence across restarts

### Property-Based Tests

Property-based tests will be defined in the next section based on the correctness properties.

## Performance Considerations

1. **Memory Management:**
   - Use `Arc` for shared ownership to minimize clones
   - Use ring buffers for audio data to avoid allocations
   - Implement model caching to avoid reloading
   - Use `bytes::Bytes` for zero-copy buffer sharing

2. **Concurrency:**
   - Use `tokio` for async I/O operations
   - Use separate tasks for audio capture, transcription, and translation
   - Use channels for inter-task communication
   - Avoid blocking operations in async contexts

3. **Optimization:**
   - Enable LTO (Link Time Optimization) in release builds
   - Use `--release` profile with `opt-level = 3`
   - Profile with `cargo flamegraph` to identify bottlenecks
   - Consider SIMD for audio processing if needed

## Migration Strategy

1. **Phase 1: Core Infrastructure**
   - Set up Rust module structure
   - Implement configuration management
   - Implement command routing layer
   - Create data models and error types

2. **Phase 2: Audio Subsystem**
   - Implement device enumeration with `cpal`
   - Implement audio recording
   - Implement energy detection and VAD

3. **Phase 3: Transcription**
   - Integrate Whisper model
   - Implement model downloading
   - Test transcription accuracy

4. **Phase 4: Translation**
   - Create CTranslate2 bindings
   - Implement API-based translators
   - Test translation quality

5. **Phase 5: Communication**
   - Implement OSC handler
   - Implement WebSocket server
   - Test with VRChat and external clients

6. **Phase 6: Overlay**
   - Integrate OpenVR
   - Implement text rendering
   - Test in VR environment

7. **Phase 7: Integration & Testing**
   - Connect all subsystems through controller
   - End-to-end testing
   - Performance profiling
   - Bug fixes

8. **Phase 8: Deployment**
   - Remove Python dependencies
   - Delete `src-python` directory
   - Update build scripts
   - Update documentation


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Configuration Round-Trip Consistency

*For any* valid configuration data structure, serializing to JSON and then deserializing should produce an equivalent configuration.

**Validates: Requirements 1.2, 9.1**

### Property 2: Configuration Persistence After Shutdown

*For any* configuration change made before shutdown, loading the configuration after restart should reflect that change.

**Validates: Requirements 1.3**

### Property 3: Device Selection Persistence

*For any* audio device selection (microphone or speaker), the configuration file should immediately contain that device name after the selection is made.

**Validates: Requirements 2.2**

### Property 4: Automatic Device Selection Matches System Default

*For any* system where automatic device selection is enabled, the selected device should match the system's default device.

**Validates: Requirements 2.5**

### Property 5: Audio Energy Threshold Filtering

*For any* audio sample with energy below the configured threshold, the sample should be filtered out and not passed to transcription.

**Validates: Requirements 3.1**

### Property 6: Transcription Result Structure

*For any* successful transcription, the result should contain both a text field and a language field.

**Validates: Requirements 3.3**

### Property 7: Transcription Parameter Application

*For any* transcription parameter change, the next transcription should use the new parameter values.

**Validates: Requirements 3.5**

### Property 8: Translation Fallback on Failure

*For any* translation request that fails with a non-CTranslate2 engine, the system should attempt translation with CTranslate2 and send an error notification to the frontend.

**Validates: Requirements 4.4**

### Property 9: OSC Message Transmission

*For any* message and valid OSC configuration (IP address and port), an OSC packet should be transmitted to that address and port.

**Validates: Requirements 5.1**

### Property 10: OSC Typing Indicator Synchronization

*For any* typing start event when typing indicators are enabled, an OSC typing-start message should be sent; for any typing stop event, an OSC typing-stop message should be sent.

**Validates: Requirements 5.2**

### Property 11: OSC Mic Mute Synchronization

*For any* OSC mute parameter received when VRC mic mute sync is enabled, the transcription state should change to match the mute state.

**Validates: Requirements 5.3**

### Property 12: OSC Notification Sound Trigger

*For any* message sent when notification sounds are enabled, an OSC sound effect parameter should be transmitted.

**Validates: Requirements 5.5**

### Property 13: WebSocket Server Listening

*For any* WebSocket server configuration when the server is enabled, a connection attempt to the configured host and port should succeed.

**Validates: Requirements 6.1**

### Property 14: WebSocket Event Broadcasting

*For any* transcription or translation event, all currently connected WebSocket clients should receive the event message.

**Validates: Requirements 6.3**

### Property 15: WebSocket Server Restart on Configuration Change

*For any* WebSocket server configuration change, the server should stop listening on the old host/port and start listening on the new host/port.

**Validates: Requirements 6.4**

### Property 16: Overlay Settings Application

*For any* overlay setting change, the overlay should reflect the new settings without requiring application restart.

**Validates: Requirements 7.3**

### Property 17: Overlay Independence

*For any* change to small log overlay settings, the large log overlay settings should remain unchanged, and vice versa.

**Validates: Requirements 7.4**

### Property 18: Configuration Validation with Defaults

*For any* invalid configuration value, the system should use the default value for that setting and log a validation error.

**Validates: Requirements 9.3**

### Property 19: Device Selection Validation

*For any* configured device selection where the device no longer exists, the system should either reset to default or mark the device as invalid.

**Validates: Requirements 9.5**

### Property 20: Command Parsing Consistency

*For any* valid JSON command from the frontend, the Rust backend should successfully parse the command and extract the endpoint and data fields.

**Validates: Requirements 10.1**

### Property 21: Response Format Consistency

*For any* backend response, the response should contain status, endpoint, and result fields in the expected JSON structure.

**Validates: Requirements 10.2**

### Property 22: Error Response Structure

*For any* error condition, the error response should contain status, endpoint, and result fields with error information.

**Validates: Requirements 10.4**

### Property 23: Japanese Hiragana Conversion

*For any* Japanese text when hiragana conversion is enabled, the transliteration output should contain hiragana representations.

**Validates: Requirements 11.1**

### Property 24: Japanese Romaji Conversion

*For any* Japanese text when romaji conversion is enabled, the transliteration output should contain romaji (Hepburn) representations.

**Validates: Requirements 11.2**

### Property 25: Transliteration Settings Application

*For any* transliteration setting change, the next transliteration operation should use the new settings.

**Validates: Requirements 11.4**

### Property 26: Transliteration Error Fallback

*For any* transliteration error, the original text should be returned unchanged.

**Validates: Requirements 11.5**

### Property 27: Word Filter Blocking

*For any* transcribed text containing a word from the filter list, the message should be blocked and a notification sent to the frontend.

**Validates: Requirements 12.2**

### Property 28: Word Filter Update Application

*For any* word filter list change, the next transcription should use the updated filter list.

**Validates: Requirements 12.3**

### Property 29: Case-Insensitive Word Filtering

*For any* filtered word, the filter should match the word regardless of case (uppercase, lowercase, or mixed case).

**Validates: Requirements 12.4**

### Property 30: Energy Level Reporting

*For any* period when threshold checking is enabled, energy level updates should be sent to the frontend.

**Validates: Requirements 13.3**

### Property 31: Download Progress Monotonicity

*For any* model download in progress, progress percentage values should be non-decreasing and within the range [0, 100].

**Validates: Requirements 15.2**

### Property 32: Download Queue Sequential Processing

*For any* multiple download requests, downloads should be processed one at a time, not concurrently.

**Validates: Requirements 15.5**

### Property 33: Logging Event Persistence

*For any* transcription or translation event when logging is enabled, the event should appear in the log file with timestamp, type, original text, and translations.

**Validates: Requirements 16.1, 16.2**

### Property 34: Logging Disable Immediate Effect

*For any* logging disable action, no new log entries should be written after the disable command completes.

**Validates: Requirements 16.4**

### Property 35: Message Format Prefix/Suffix Application

*For any* message and format configuration, the formatted output should contain the configured prefix before the message and suffix after the message.

**Validates: Requirements 19.1, 19.2**

### Property 36: Translation-First Ordering

*For any* message with translation when translation-first mode is enabled, the translation should appear before the original message in the formatted output.

**Validates: Requirements 19.3**

### Property 37: Send-Only-Translated Omission

*For any* message when send-only-translated mode is enabled, the formatted output should not contain the original message text.

**Validates: Requirements 19.4**

### Property 38: Formatting Error Fallback

*For any* message formatting error, the system should produce a fallback format containing both original and translation.

**Validates: Requirements 19.5**

### Property 39: Error Logging Completeness

*For any* error that occurs in any subsystem, an error log entry should be created with context information.

**Validates: Requirements 20.1**

### Property 40: Error Response Structure Consistency

*For any* error reported to the frontend, the response should include an error type field and a user-friendly message field.

**Validates: Requirements 20.3**

## Property Reflection

After reviewing all properties, the following observations ensure no redundancy:

1. **Configuration Properties (1-3, 18-19):** Each addresses a distinct aspect - round-trip serialization, persistence across restarts, immediate persistence, validation, and device validation.

2. **Audio Properties (4-5, 30):** Cover device selection, energy filtering, and energy reporting - all distinct concerns.

3. **Transcription Properties (6-7):** Cover result structure and parameter application - complementary properties.

4. **Translation Properties (8):** Single property for fallback behavior.

5. **OSC Properties (9-12):** Each covers a different OSC feature - message sending, typing indicators, mute sync, and sound effects.

6. **WebSocket Properties (13-15):** Cover server listening, broadcasting, and restart - all distinct.

7. **Overlay Properties (16-17):** Cover settings application and independence - complementary.

8. **Command/Response Properties (20-22):** Cover parsing, response format, and error format - all distinct.

9. **Transliteration Properties (23-26):** Cover hiragana, romaji, settings, and error handling - all distinct.

10. **Word Filter Properties (27-29):** Cover blocking, updates, and case-insensitivity - all distinct.

11. **Download Properties (31-32):** Cover progress monotonicity and sequential processing - complementary.

12. **Logging Properties (33-34):** Cover event persistence and disable behavior - complementary.

13. **Message Formatting Properties (35-38):** Each covers a different formatting aspect - prefix/suffix, ordering, omission, and fallback.

14. **Error Handling Properties (39-40):** Cover logging and response structure - complementary.

All properties provide unique validation value without logical redundancy.
