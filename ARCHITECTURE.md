# VRCT Architecture Documentation

## Overview

VRCT is a pure Rust application built with the Tauri framework, providing high performance, efficient memory usage, and a single self-contained executable.

## Architecture

### Technology Stack

- **Frontend**: React + JavaScript
- **Backend**: Pure Rust
- **Framework**: Tauri 2.x
- **Build System**: Cargo + Vite

### Module Structure

```
src-tauri/src/
├── lib.rs                    # Main library entry point
├── main.rs                   # Binary entry point
├── commands.rs               # Tauri command handlers
├── controller.rs             # Main orchestration layer
├── events.rs                 # Event definitions
├── config/                   # Configuration management
│   ├── mod.rs
│   ├── types.rs
│   └── persistence.rs
├── audio/                    # Audio processing
│   ├── mod.rs
│   ├── device_manager.rs
│   ├── recorder.rs
│   ├── energy.rs
│   └── integration_tests.rs
├── transcription/            # Speech-to-text
│   ├── mod.rs
│   ├── whisper.rs
│   └── languages.rs
├── translation/              # Translation engines
│   ├── mod.rs
│   ├── ctranslate2.rs
│   ├── languages.rs
│   ├── fallback_tests.rs
│   └── engines/
│       ├── deepl.rs
│       ├── openai.rs
│       ├── gemini.rs
│       ├── lmstudio.rs
│       ├── ollama.rs
│       └── plamo.rs
├── transliteration/          # Japanese text conversion
│   ├── mod.rs
│   ├── kana.rs
│   └── romaji.rs
├── communication/            # Network protocols
│   ├── mod.rs
│   ├── osc.rs
│   └── websocket.rs
├── overlay/                  # VR overlay rendering
│   ├── mod.rs
│   └── renderer.rs
├── utils/                    # Utilities
│   ├── mod.rs
│   ├── error.rs
│   ├── logging.rs
│   ├── keyword_filter.rs
│   ├── message_filter.rs
│   ├── message_formatter.rs
│   ├── updater.rs
│   ├── zluda.rs
│   ├── profiling.rs
│   └── benchmarks.rs
└── models/                   # Data models
    ├── mod.rs
    ├── device.rs
    └── message.rs
```

## Key Components

### 1. Configuration Management (`config/`)
- JSON-based configuration with serde
- Debounced writes (2 second delay)
- Thread-safe access with Arc<RwLock>
- Automatic validation and defaults

### 2. Audio Processing (`audio/`)
- Device enumeration using `cpal`
- Real-time audio capture
- Energy-based voice activity detection
- Lock-free ring buffers for audio data

### 3. Transcription (`transcription/`)
- Whisper model integration using `candle`
- Model downloading from Hugging Face
- CPU/CUDA/ZLUDA device support
- Automatic fallback on VRAM overflow

### 4. Translation (`translation/`)
- Multiple engine support:
  - CTranslate2 (local)
  - DeepL API
  - OpenAI API
  - Google Gemini API
  - LM Studio (local)
  - Ollama (local)
  - Plamo API
- Automatic fallback to CTranslate2 on API failures
- Rate limiting and retry logic

### 5. Communication (`communication/`)
- **OSC**: VRChat chatbox integration using `rosc`
- **WebSocket**: External integration API using `axum`
- Typing indicators and mic mute sync

### 6. VR Overlay (`overlay/`)
- OpenVR integration (optional feature)
- Text rendering with `rusttype`
- Multiple overlay types (small log, large log)

### 7. Transliteration (`transliteration/`)
- Japanese kana conversion
- Hepburn romanization
- Context-aware rules

## Build Instructions

### Development Build

```bash
# Install dependencies
npm install

# Run in development mode (with hot reload)
npm run dev
```

Development builds:
- Include debug symbols
- No optimizations (faster compilation)
- Larger binary size
- Better error messages and stack traces

### Production Build

```bash
# Build optimized release
npm run build
```

The release build uses aggressive optimizations:
- **LTO (Link Time Optimization)**: "fat" - Optimizes across all crates
- **Optimization level**: 3 - Maximum optimization
- **Codegen units**: 1 - Single unit for best optimization
- **Binary stripping**: true - Removes debug symbols
- **Panic strategy**: "abort" - Smaller binary, no unwinding

Release profile in `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

### Build Artifacts

**Windows:**
- Executable: `src-tauri/target/release/VRCT.exe`
- Installer: `src-tauri/target/release/bundle/nsis/VRCT_x.x.x_x64-setup.exe`
- MSI: `src-tauri/target/release/bundle/msi/VRCT_x.x.x_x64_en-US.msi`

### Build Times

Typical build times on modern hardware:

| Build Type | First Build | Incremental |
|------------|-------------|-------------|
| Debug | 10-15 min | 5-30 sec |
| Release | 15-25 min | 1-5 min |

**Tips for faster builds:**
- Use `cargo check` instead of `cargo build` for quick error checking
- Use `cargo build` (debug) for development
- Only use `cargo build --release` when testing performance
- Use `sccache` or `mold` linker for faster linking

### Cross-Compilation

Currently, VRCT is built for Windows x64. For other platforms:

```bash
# Add target
rustup target add x86_64-pc-windows-msvc

# Build for target
cargo build --target x86_64-pc-windows-msvc --release
```

### Build Optimization Tips

1. **Use faster linker (Windows):**
   ```toml
   # .cargo/config.toml
   [target.x86_64-pc-windows-msvc]
   linker = "rust-lld.exe"
   ```

2. **Parallel compilation:**
   ```bash
   # Set number of parallel jobs
   cargo build -j 8
   ```

3. **Incremental compilation (enabled by default in debug):**
   ```toml
   [profile.dev]
   incremental = true
   ```

4. **Reduce debug info for faster builds:**
   ```toml
   [profile.dev]
   debug = 1  # Line tables only (instead of full debug info)
   ```

## Dependencies

### Core Dependencies
- `tauri` (2.x): Application framework for desktop apps with web frontends
- `tokio` (1.x): Async runtime for concurrent operations
- `serde` (1.x): Serialization/deserialization framework
- `serde_json`: JSON support for serde
- `tracing`: Structured logging and diagnostics
- `tracing-subscriber`: Log formatting and output

### Audio Processing
- `cpal` (0.15): Cross-platform audio I/O library
- `ringbuf` (0.3): Lock-free SPSC ring buffer for audio data
- `hound`: WAV file reading/writing (for testing)

### ML/AI
- `candle-core`: Rust ML framework (similar to PyTorch)
- `candle-transformers`: Pre-built transformer architectures
- `candle-nn`: Neural network layers and operations
- `hf-hub`: Hugging Face Hub API for model downloading
- `tokenizers`: Fast tokenization library

### Network & Communication
- `reqwest`: Async HTTP client with rustls
- `axum`: Web framework for WebSocket server
- `tokio-tungstenite`: WebSocket protocol implementation
- `rosc`: Open Sound Control (OSC) protocol
- `tower`: Middleware for axum

### Text Processing
- `unicode-normalization`: Unicode text normalization
- `regex`: Regular expressions
- `aho-corasick`: Fast multi-pattern string matching (for word filters)

### Utilities
- `thiserror`: Ergonomic error handling
- `anyhow`: Flexible error handling for applications
- `chrono`: Date and time handling
- `dirs`: Platform-specific directory paths
- `once_cell`: Lazy static initialization

### Optional Features
- `openvr`: VR overlay support (feature: `overlay`)
- `rusttype` or `fontdue`: Font rendering for overlays

### Development Dependencies
- `criterion`: Benchmarking framework
- `proptest`: Property-based testing
- `mockall`: Mocking framework for tests
- `tempfile`: Temporary file handling in tests

## Cargo Features

The project uses Cargo features for optional functionality:

```toml
[features]
default = []
overlay = ["openvr", "rusttype", "image"]  # VR overlay support
cuda = ["candle-core/cuda"]                 # CUDA acceleration
```

Usage:
```bash
# Build with overlay support
cargo build --features overlay

# Build with CUDA support
cargo build --features cuda

# Build with all features
cargo build --all-features
```

## Concurrency Model

### Async Runtime

VRCT uses Tokio as its async runtime for handling concurrent operations:

```rust
#[tokio::main]
async fn main() {
    // Tokio runtime handles all async operations
}
```

**Key async operations:**
- HTTP requests (translation APIs, model downloads)
- WebSocket connections
- File I/O (configuration, logging)
- OSC communication

### Thread Model

```
Main Thread (Tokio Runtime)
├── Audio Capture Thread (cpal)
│   └── Lock-free ring buffer → Main thread
├── Transcription Task (async)
├── Translation Task (async)
├── WebSocket Server Task (async)
├── OSC Receiver Task (async)
└── Configuration Save Task (async, debounced)
```

**Thread safety:**
- `Arc<RwLock<T>>` for shared mutable state
- `Arc<T>` for shared immutable state
- Channels (`mpsc`, `broadcast`) for message passing
- Lock-free ring buffers for audio data

### Synchronization Primitives

```rust
// Shared configuration
Arc<RwLock<ConfigData>>

// Shared transcriber (read-only after init)
Arc<WhisperTranscriber>

// Message passing
tokio::sync::mpsc::channel()
tokio::sync::broadcast::channel()

// One-time initialization
once_cell::sync::Lazy
```

## Performance Characteristics

### Memory Usage
- **Baseline**: ~100MB
- **With models loaded**: ~500MB-2GB (depending on model size)
- **No garbage collection**: Deterministic memory management
- **Zero-copy where possible**: Using `&[u8]`, `Bytes`, etc.

**Memory optimization techniques:**
- Arc for shared ownership (no cloning)
- Ring buffers for audio (no allocations in hot path)
- String interning for repeated strings
- Lazy loading of models

### Startup Time
- **Cold start**: ~1 second
- **Warm start**: <500ms (with cached config)
- **Model loading**: Deferred until first use
- **Configuration**: Loaded asynchronously

**Startup optimization:**
- Parallel initialization of subsystems
- Lazy model loading
- Cached configuration validation
- Minimal dependencies in critical path

### Runtime Performance
- **Native code**: No interpreter overhead
- **Zero-cost abstractions**: Rust's abstractions compile to optimal code
- **SIMD**: Automatic vectorization where applicable
- **Inlining**: Aggressive inlining in release builds

**Performance benchmarks:**
| Operation | Time | Notes |
|-----------|------|-------|
| Audio energy calc | 5 µs | Per 1000 samples |
| Config load | 1 ms | From disk |
| JSON parsing | 0.5 ms | Typical config size |
| Transcription | 450 ms | Depends on model and hardware |

### Latency Characteristics

**Audio pipeline latency:**
```
Microphone → cpal (10ms) → Ring buffer (0ms) → 
Energy detection (1ms) → Transcription (200-500ms) → 
Translation (50-200ms) → OSC/WebSocket (5ms)

Total: ~270-720ms
```

**Optimization opportunities:**
- Reduce audio buffer size (trade-off: quality vs latency)
- Use smaller Whisper models (trade-off: accuracy vs speed)
- Local translation (CTranslate2) vs API (network latency)
- Batch processing for multiple translations

## Architecture Benefits

1. **Single Executable**: No external runtime required
2. **Efficient Memory**: Optimized memory management
3. **Fast Startup**: Quick application launch
4. **Better Error Handling**: Type-safe error propagation
5. **Improved Concurrency**: Tokio async runtime
6. **Smaller Distribution**: Single self-contained binary

## Development Guidelines

### Code Style
- Follow Rust standard conventions
- Use `rustfmt` for formatting
- Use `clippy` for linting

### Testing
- Unit tests in module files
- Integration tests in separate files
- Property-based tests for critical logic

### Error Handling
- Use `Result<T, VrctError>` for fallible operations
- Implement graceful degradation
- Log errors with context

### Async/Await
- Use `tokio` for async operations
- Avoid blocking in async contexts
- Use channels for inter-task communication

## Troubleshooting

### Build Issues

**"linker 'link.exe' not found"**
- Install Visual Studio Build Tools with C++ support
- Ensure MSVC toolchain is in PATH

**"failed to run custom build command"**
- Check that all system dependencies are installed
- Try: `cargo clean` then rebuild

**Out of memory during build**
- Close other applications
- Reduce parallel jobs: `cargo build -j 2`
- Increase virtual memory (Windows: System Properties > Advanced > Performance Settings)

**Slow builds**
- First build is always slow (10-20 minutes)
- Use `cargo check` for quick error checking
- Consider using `sccache` for caching

### Runtime Issues

**Application won't start**
- Check logs in: `%APPDATA%/VRCT/logs/`
- Enable debug logging: `set RUST_LOG=debug`
- Verify all required DLLs are present

**Audio devices not detected**
- Check Windows audio settings
- Restart audio service: `net stop audiosrv && net start audiosrv`
- Verify device is not exclusively locked by another app

**VRAM overflow errors**
- Switch to CPU compute device
- Use smaller model variants
- Close other GPU-intensive applications

**Translation API failures**
- Verify API keys are correct
- Check internet connectivity
- Review API rate limits and quotas

**WebSocket connection issues**
- Check firewall settings
- Verify port is not in use: `netstat -ano | findstr :PORT`
- Try different port in configuration

### Performance Issues

**High CPU usage**
- Check if transcription is running continuously
- Verify energy threshold settings
- Profile with: `cargo flamegraph`

**High memory usage**
- Check for memory leaks with profiler
- Verify models are being unloaded properly
- Monitor with Task Manager or Process Explorer

**Slow transcription**
- Verify correct compute device (GPU vs CPU)
- Check model size (smaller = faster)
- Ensure GPU drivers are up to date

### Development Issues

**rust-analyzer not working**
- Restart rust-analyzer: Ctrl+Shift+P > "Restart Rust Analyzer"
- Check `Cargo.toml` syntax
- Try: `cargo clean` then restart IDE

**Tests failing**
- Ensure test dependencies are installed
- Check that audio devices are available for tests
- Run with: `cargo test -- --nocapture` for detailed output

**Clippy warnings**
- Fix automatically: `cargo clippy --fix`
- Allow specific warnings: `#[allow(clippy::lint_name)]`
- Update clippy: `rustup update`

## Testing Strategy

### Unit Tests

Located in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_calculation() {
        let samples = vec![0.5, -0.5, 0.3, -0.3];
        let energy = calculate_rms_energy(&samples);
        assert!(energy > 0.0);
    }
}
```

Run with: `cargo test`

### Integration Tests

Located in separate files (e.g., `audio/integration_tests.rs`):

```rust
#[tokio::test]
async fn test_audio_device_enumeration() {
    let manager = DeviceManager::new();
    let devices = manager.enumerate_input_devices();
    assert!(!devices.is_empty());
}
```

Run with: `cargo test --test integration_tests`

### Property-Based Tests

Using `proptest` for testing properties across many inputs:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_energy_always_positive(samples in prop::collection::vec(-1.0f32..1.0, 1..1000)) {
        let energy = calculate_rms_energy(&samples);
        assert!(energy >= 0.0);
    }
}
```

### Benchmarks

Using `criterion` for performance benchmarks:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_energy(c: &mut Criterion) {
    let samples = vec![0.5; 1000];
    c.bench_function("energy_calc", |b| {
        b.iter(|| calculate_energy(black_box(&samples)))
    });
}

criterion_group!(benches, benchmark_energy);
criterion_main!(benches);
```

Run with: `cargo bench`

### Test Coverage

Generate coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage
```

### Continuous Integration

The project uses GitHub Actions for CI:

```yaml
# .github/workflows/rust.yml
name: Rust CI

on: [push, pull_request]

jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

## Security Considerations

### Memory Safety
- Rust's ownership system prevents:
  - Use-after-free
  - Double-free
  - Buffer overflows
  - Data races

### Input Validation
- All user inputs are validated
- API responses are validated before use
- Configuration files are validated on load

### Dependency Security
- Regular dependency audits: `cargo audit`
- Minimal dependency tree
- Prefer well-maintained crates

### API Key Handling
- API keys stored in configuration file
- Not logged or exposed in error messages
- Transmitted over HTTPS only

### Network Security
- HTTPS for all API calls (using `rustls`)
- WebSocket connections can be secured with TLS
- OSC is UDP (no built-in security)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and contribution guidelines.

## License

See [LICENSE](LICENSE) for license information.
