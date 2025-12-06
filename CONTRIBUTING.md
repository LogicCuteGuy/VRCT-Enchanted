# Contributing to VRCT

Thank you for your interest in contributing to VRCT! This document provides guidelines for contributing to the project.

## Development Setup

### Prerequisites

1. **Rust Toolchain**
   ```bash
   # Install rustup (Rust installer and toolchain manager)
   # Visit https://rustup.rs/ for installation instructions
   
   # After installation, ensure you have the latest stable Rust
   rustup update stable
   
   # Verify installation
   rustc --version
   cargo --version
   ```

2. **Node.js and npm**
   ```bash
   # Node.js 18+ required
   # Visit https://nodejs.org/ for installation
   
   # Verify installation
   node --version
   npm --version
   ```

3. **System Dependencies**
   
   **Windows:**
   - Visual Studio Build Tools 2019 or later with C++ support
   - Or Visual Studio 2019/2022 with "Desktop development with C++" workload
   - Download: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
   
   **Required Components:**
   - MSVC v142+ (or latest)
   - Windows 10 SDK
   - C++ CMake tools
   
   **Audio:**
   - Audio libraries are handled automatically by `cpal`
   - No additional audio drivers needed

4. **Optional Tools**
   ```bash
   # Rust analyzer for IDE support (highly recommended)
   # Install via your IDE's extension marketplace
   
   # Additional cargo tools
   cargo install cargo-watch    # Auto-rebuild on file changes
   cargo install cargo-edit      # Manage dependencies easily
   cargo install cargo-outdated  # Check for outdated dependencies
   ```

### Initial Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/misyaguziya/VRCT.git
   cd VRCT
   ```

2. **Install npm dependencies**
   ```bash
   npm install
   ```

3. **Verify Rust setup**
   ```bash
   cd src-tauri
   cargo check
   cd ..
   ```

4. **Run development build**
   ```bash
   npm run dev
   ```
   
   This will:
   - Start the Vite dev server for the frontend
   - Compile and run the Rust backend
   - Open the application with hot-reload enabled

### First Build Notes

- **First build takes longer**: Rust compiles all dependencies from source (10-20 minutes)
- **Subsequent builds are fast**: Incremental compilation reuses previous work (seconds)
- **Debug builds are slower**: Use `npm run dev` for development, `npm run build` for production
- **Disk space**: Build artifacts can use 2-3GB; run `cargo clean` to free space

## Project Structure

```
VRCT/
├── src-tauri/              # Rust backend
│   ├── src/                # Rust source code
│   │   ├── lib.rs          # Library entry point
│   │   ├── main.rs         # Binary entry point
│   │   ├── commands.rs     # Tauri command handlers
│   │   ├── controller.rs   # Main orchestration
│   │   ├── audio/          # Audio processing
│   │   ├── transcription/  # Speech-to-text
│   │   ├── translation/    # Translation engines
│   │   ├── communication/  # OSC, WebSocket
│   │   ├── overlay/        # VR overlay
│   │   ├── config/         # Configuration
│   │   ├── utils/          # Utilities
│   │   └── models/         # Data models
│   ├── Cargo.toml          # Rust dependencies
│   ├── Cargo.lock          # Locked dependency versions
│   ├── tauri.conf.json     # Tauri configuration
│   └── target/             # Build output (gitignored)
├── src-ui/                 # React frontend
│   ├── views/              # React components
│   ├── logics/             # Business logic
│   └── plugins/            # Plugin system
├── locales/                # Internationalization files
├── docs/                   # Documentation
├── package.json            # npm dependencies and scripts
├── vite.config.js          # Vite configuration
├── README.md               # User documentation
├── ARCHITECTURE.md         # Technical architecture
├── CONTRIBUTING.md         # This file
└── MIGRATION_GUIDE.md      # Migration from Python
```

## Development Workflow

### Making Changes

1. Create a feature branch
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes
   - Follow Rust conventions (see Code Style below)
   - Add tests for new functionality
   - Update documentation as needed

3. Test your changes
   ```bash
   # Run Rust tests
   cd src-tauri
   cargo test
   
   # Run in development mode
   cd ..
   npm run dev
   ```

4. Format and lint
   ```bash
   cd src-tauri
   cargo fmt
   cargo clippy -- -D warnings
   ```

5. Commit your changes
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

6. Push and create a pull request
   ```bash
   git push origin feature/your-feature-name
   ```

## Code Style

### Rust Code

- **Formatting**: Use `rustfmt` (automatically applied with `cargo fmt`)
  ```bash
  # Format all code
  cargo fmt
  
  # Check formatting without modifying
  cargo fmt -- --check
  ```

- **Linting**: Pass `clippy` checks (`cargo clippy`)
  ```bash
  # Run clippy
  cargo clippy
  
  # Treat warnings as errors (CI mode)
  cargo clippy -- -D warnings
  
  # Fix automatically where possible
  cargo clippy --fix
  ```

- **Naming Conventions**:
  - `snake_case` for functions, variables, modules, fields
  - `PascalCase` for types, traits, enums, type parameters
  - `SCREAMING_SNAKE_CASE` for constants and statics
  - Prefix unused variables with `_` (e.g., `_unused_param`)
  
  ```rust
  // Good
  const MAX_BUFFER_SIZE: usize = 1024;
  struct AudioDevice { ... }
  fn process_audio_samples(input: &[f32]) -> Vec<f32> { ... }
  
  // Bad
  const maxBufferSize: usize = 1024;  // Wrong case
  struct audioDevice { ... }           // Wrong case
  fn ProcessAudioSamples(...) { ... }  // Wrong case
  ```

- **Documentation**: Add doc comments for public APIs
  ```rust
  /// Transcribes audio using the Whisper model.
  ///
  /// This function processes raw audio samples and returns transcribed text
  /// along with metadata such as detected language and confidence score.
  ///
  /// # Arguments
  /// * `audio` - Audio samples as f32 slice (normalized to -1.0 to 1.0)
  ///
  /// # Returns
  /// Result containing transcription text and metadata
  ///
  /// # Errors
  /// Returns `VrctError::Transcription` if:
  /// - Audio buffer is empty
  /// - Model is not loaded
  /// - VRAM overflow occurs
  ///
  /// # Examples
  /// ```no_run
  /// let transcriber = WhisperTranscriber::new(model_path, device).await?;
  /// let audio = vec![0.0; 16000]; // 1 second at 16kHz
  /// let result = transcriber.transcribe(&audio).await?;
  /// println!("Transcribed: {}", result.text);
  /// ```
  pub async fn transcribe(&self, audio: &[f32]) -> Result<TranscriptionResult>
  ```

- **Module Documentation**:
  ```rust
  //! Audio processing module.
  //!
  //! This module provides audio device enumeration, recording, and
  //! energy-based voice activity detection.
  ```

### Rust Best Practices

1. **Prefer borrowing over cloning**:
   ```rust
   // Good
   fn process(data: &[u8]) { ... }
   
   // Avoid (unless necessary)
   fn process(data: Vec<u8>) { ... }
   ```

2. **Use `?` operator for error propagation**:
   ```rust
   // Good
   let device = device_manager.get_device(name)?;
   
   // Avoid
   let device = match device_manager.get_device(name) {
       Ok(d) => d,
       Err(e) => return Err(e),
   };
   ```

3. **Leverage type inference**:
   ```rust
   // Good
   let samples = vec![0.0; 1024];
   
   // Unnecessary
   let samples: Vec<f32> = vec![0.0; 1024];
   ```

4. **Use iterators instead of loops**:
   ```rust
   // Good
   let sum: f32 = samples.iter().sum();
   
   // Less idiomatic
   let mut sum = 0.0;
   for sample in &samples {
       sum += sample;
   }
   ```

5. **Avoid `unwrap()` in production code**:
   ```rust
   // Good
   let device = device_manager.get_device(name)
       .ok_or_else(|| VrctError::AudioDevice("Device not found".into()))?;
   
   // Bad (can panic)
   let device = device_manager.get_device(name).unwrap();
   ```

6. **Use `Arc` for shared ownership, `Rc` for single-threaded**:
   ```rust
   // Multi-threaded
   let config = Arc::new(RwLock::new(ConfigData::default()));
   
   // Single-threaded
   let config = Rc::new(RefCell::new(ConfigData::default()));
   ```

7. **Prefer `&str` over `String` for function parameters**:
   ```rust
   // Good (more flexible)
   fn log_message(msg: &str) { ... }
   
   // Less flexible
   fn log_message(msg: String) { ... }
   ```

### Error Handling

- Use `Result<T, VrctError>` for fallible operations
- Provide context with error messages
- Implement graceful degradation where possible
  ```rust
  // Good
  device_manager.get_device(name)
      .ok_or_else(|| VrctError::AudioDevice(format!("Device not found: {}", name)))?
  
  // Avoid unwrap() in production code
  ```

### Async Code

- Use `async/await` for I/O operations
- Avoid blocking in async contexts
- Use `tokio::spawn` for concurrent tasks
  ```rust
  // Good
  let result = tokio::spawn(async move {
      // async work
  }).await?;
  
  // Avoid blocking
  // Bad: std::thread::sleep in async context
  ```

## Testing

### Unit Tests

Place unit tests in the same file as the code:

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

### Integration Tests

Place integration tests in `src-tauri/src/*/integration_tests.rs`:

```rust
#[tokio::test]
async fn test_audio_recording_workflow() {
    let device_manager = DeviceManager::new();
    let devices = device_manager.enumerate_input_devices();
    assert!(!devices.is_empty());
}
```

### Running Tests

```bash
cd src-tauri

# Run all tests
cargo test

# Run specific test
cargo test test_energy_calculation

# Run with output
cargo test -- --nocapture
```

## Adding New Features

### 1. Audio Features
- Modify `src-tauri/src/audio/`
- Update `DeviceManager` or `AudioRecorder` as needed
- Add tests in `integration_tests.rs`

### 2. Translation Engines
- Create new file in `src-tauri/src/translation/engines/`
- Implement `Translator` trait
- Register in `TranslationManager`
- Add configuration options

### 3. Communication Protocols
- Modify `src-tauri/src/communication/`
- Update protocol handlers
- Add integration tests

### 4. UI Changes
- Modify `src-ui/` components
- Update localization files in `locales/`
- Test with `npm run dev`

## Debugging

### Rust Backend

1. **Enable debug logging:**
   ```bash
   # Windows Command Prompt
   set RUST_LOG=debug
   npm run dev
   
   # Windows PowerShell
   $env:RUST_LOG="debug"
   npm run dev
   
   # Levels: error, warn, info, debug, trace
   # Module-specific: RUST_LOG=vrct::audio=debug,vrct::translation=trace
   ```

2. **Quick debugging with `dbg!()` macro:**
   ```rust
   let result = some_function();
   dbg!(&result);  // Prints to stderr with file:line info
   ```

3. **Structured logging with `tracing`:**
   ```rust
   use tracing::{info, debug, error};
   
   info!("Processing audio");
   debug!(device = ?device_name, "Selected device");
   error!(error = ?err, "Failed to initialize");
   ```

4. **VSCode debugging:**
   - Install "rust-analyzer" extension
   - Install "CodeLLDB" extension
   - Add breakpoints in Rust code
   - Press F5 to start debugging
   
   Example `.vscode/launch.json`:
   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug VRCT",
         "cargo": {
           "args": ["build", "--manifest-path=src-tauri/Cargo.toml"]
         },
         "args": [],
         "cwd": "${workspaceFolder}"
       }
     ]
   }
   ```

5. **Cargo commands for debugging:**
   ```bash
   # Check for errors without building
   cargo check
   
   # Build with debug symbols
   cargo build
   
   # Run tests with output
   cargo test -- --nocapture
   
   # Run specific test
   cargo test test_name -- --nocapture
   
   # Show expanded macros
   cargo expand
   ```

### Frontend

1. **Open DevTools** in the application (F12 or Ctrl+Shift+I)
2. **Console tab**: Check for JavaScript errors
3. **Network tab**: Monitor API calls and WebSocket connections
4. **React DevTools**: Install browser extension for component inspection
5. **Tauri DevTools**: Built-in developer tools for Tauri-specific debugging

## Performance Profiling

### CPU Profiling

**Using cargo-flamegraph (recommended):**
```bash
cd src-tauri

# Install cargo-flamegraph
cargo install flamegraph

# Generate flamegraph (requires admin/root)
cargo flamegraph --bin VRCT

# Opens flamegraph.svg in browser
```

**Using perf (Linux):**
```bash
# Record performance data
perf record --call-graph dwarf target/release/VRCT

# Generate report
perf report
```

**Using Windows Performance Analyzer:**
1. Install Windows Performance Toolkit
2. Record trace: `wpr -start CPU -filemode`
3. Stop trace: `wpr -stop trace.etl`
4. Analyze with Windows Performance Analyzer

### Memory Profiling

**Using cargo-instruments (macOS):**
```bash
cargo install cargo-instruments
cargo instruments -t Allocations
```

**Using heaptrack (Linux):**
```bash
heaptrack target/release/VRCT
heaptrack_gui heaptrack.VRCT.*.gz
```

**Using Windows Performance Analyzer:**
- Use "Memory" profile instead of "CPU"
- Analyze heap allocations and memory usage

**Using Rust-specific tools:**
```bash
# Memory usage tracking
cargo install cargo-bloat
cargo bloat --release

# Binary size analysis
cargo install cargo-binutils
cargo size --release
```

### Benchmarking

**Using criterion (for micro-benchmarks):**
```rust
// benches/audio_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_energy_calculation(c: &mut Criterion) {
    let samples = vec![0.5; 1000];
    c.bench_function("energy_calc", |b| {
        b.iter(|| calculate_energy(black_box(&samples)))
    });
}

criterion_group!(benches, benchmark_energy_calculation);
criterion_main!(benches);
```

```bash
# Run benchmarks
cargo bench
```

### Optimization Tips

1. **Profile before optimizing**: Measure to find actual bottlenecks
2. **Use release builds**: `cargo build --release` enables optimizations
3. **Check assembly**: `cargo asm` to see generated code
4. **Reduce allocations**: Use `&str` instead of `String` where possible
5. **Use appropriate data structures**: `Vec` vs `HashMap` vs `BTreeMap`
6. **Leverage parallelism**: Use `rayon` for data parallelism
7. **Async for I/O**: Use `tokio` for concurrent I/O operations

## Documentation

### Code Documentation

- Add doc comments to public APIs
- Include examples in doc comments
- Run `cargo doc --open` to view generated docs

### User Documentation

- Update README.md for user-facing changes
- Update ARCHITECTURE.md for architectural changes
- Add examples to docs/ directory

## Commit Message Guidelines

Follow conventional commits format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(audio): add automatic threshold adjustment

Implements dynamic threshold adjustment based on ambient noise levels.
Improves voice activity detection accuracy.

Closes #123
```

```
fix(translation): handle API rate limiting

Add exponential backoff for translation API requests.
Prevents errors when rate limits are exceeded.
```

## Pull Request Process

1. **Before Submitting**
   - Ensure all tests pass
   - Run `cargo fmt` and `cargo clippy`
   - Update documentation
   - Add tests for new features

2. **PR Description**
   - Describe what changes were made
   - Explain why the changes were necessary
   - Reference related issues
   - Include screenshots for UI changes

3. **Review Process**
   - Address reviewer feedback
   - Keep commits clean and logical
   - Squash commits if requested

4. **After Merge**
   - Delete your feature branch
   - Update your local main branch

## Common Issues and Solutions

### Build Issues

**Issue: "linker 'link.exe' not found"**
- **Solution**: Install Visual Studio Build Tools with C++ support
- Download: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022

**Issue: "failed to run custom build command for `openssl-sys`"**
- **Solution**: This shouldn't occur as we use `rustls` instead of OpenSSL
- If it does, check that all dependencies in `Cargo.toml` use `rustls` features

**Issue: "cargo build" is very slow**
- **Solution**: First build compiles all dependencies (10-20 min is normal)
- Subsequent builds are much faster due to incremental compilation
- Use `cargo build --release` only when needed (slower but optimized)

**Issue: Out of disk space during build**
- **Solution**: Build artifacts can use 2-3GB
- Clean with: `cargo clean`
- Or clean specific target: `cargo clean --release`

### Runtime Issues

**Issue: Audio devices not detected**
- **Solution**: Check Windows audio settings
- Restart the application
- Verify device is not in use by another application

**Issue: "VRAM overflow" error**
- **Solution**: Model doesn't fit in GPU memory
- Switch to CPU in settings
- Or use a smaller model variant

**Issue: Translation API errors**
- **Solution**: Verify API key is correct
- Check internet connection
- Verify API quota/rate limits

### Development Issues

**Issue: rust-analyzer is slow or unresponsive**
- **Solution**: Restart rust-analyzer
- Check that `Cargo.toml` is valid
- Try: `cargo clean` then restart IDE

**Issue: "cannot find macro `dbg` in this scope"**
- **Solution**: `dbg!` is in the prelude, but check Rust version
- Update Rust: `rustup update stable`

**Issue: Clippy warnings about unused code**
- **Solution**: Prefix with underscore: `_unused_var`
- Or use `#[allow(dead_code)]` if intentional

## Getting Help

- **Issues**: Check [existing issues](https://github.com/misyaguziya/VRCT/issues) or create a new one
- **Discussions**: Use [GitHub Discussions](https://github.com/misyaguziya/VRCT/discussions) for questions
- **Documentation**: Read [ARCHITECTURE.md](ARCHITECTURE.md) for technical details
- **Community**: Join the community Discord (if available)

When asking for help, please include:
- Your OS and version
- Rust version (`rustc --version`)
- Node.js version (`node --version`)
- Full error message or log output
- Steps to reproduce the issue

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Help others learn and grow

## License

By contributing to VRCT, you agree that your contributions will be licensed under the same license as the project (see LICENSE file).

## Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- GitHub contributors page

Thank you for contributing to VRCT! 🎉
