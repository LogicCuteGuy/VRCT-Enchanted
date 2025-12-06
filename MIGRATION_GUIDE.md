# Migration Guide: Python to Rust

This guide helps users transition from the Python-based VRCT to the new Rust-based version.

## What Changed?

### Architecture
- **Before**: Python backend + Tauri frontend
- **After**: Pure Rust backend + Tauri frontend

### Installation
- **Before**: Required Python 3.11 installation and virtual environment setup
- **After**: Single executable, no Python required

### Performance
- **Startup Time**: ~3x faster
- **Memory Usage**: ~50% reduction
- **Binary Size**: Smaller single executable

## Upgrading

### For End Users

1. **Backup Your Configuration**
   - Your configuration file is located at: `%APPDATA%/VRCT/config.json`
   - Make a backup copy before upgrading

2. **Uninstall Old Version** (Optional)
   - You can simply replace the old executable with the new one
   - Or uninstall the old version first

3. **Install New Version**
   - Download the latest release
   - Run the new executable
   - Your configuration will be automatically migrated

4. **Verify Settings**
   - Check that your audio devices are still selected correctly
   - Verify translation engine settings
   - Test OSC connection to VRChat

### Configuration Compatibility

The new Rust version uses the **same configuration format** as the Python version. Your existing settings will work without modification.

Configuration file location remains the same:
- Windows: `%APPDATA%/VRCT/config.json`

### What's Preserved

✅ All your settings and preferences
✅ Audio device selections
✅ Translation engine configurations
✅ OSC settings
✅ Overlay settings
✅ Word filters
✅ Language preferences

### What's Different

#### System Requirements
- **Before**: Python 3.11 required
- **After**: No Python required

#### Installation Size
- **Before**: ~500MB (Python + dependencies)
- **After**: ~50MB (single executable)

#### Startup Behavior
- **Before**: Python process + Tauri process
- **After**: Single Rust process (faster startup)

#### Model Storage
- Models are still stored in the same location
- No need to re-download Whisper or CTranslate2 models

## Troubleshooting

### Configuration Not Loading

If your configuration doesn't load:

1. Check that `config.json` exists in `%APPDATA%/VRCT/`
2. Verify the JSON is valid (use a JSON validator)
3. Delete `config.json` to reset to defaults (backup first!)

### Audio Devices Not Detected

If audio devices aren't showing up:

1. Restart the application
2. Check Windows audio settings
3. Try selecting "Default" device first

### Translation Not Working

If translation fails:

1. Verify your API keys are still valid
2. Check internet connection for API-based translators
3. Ensure CTranslate2 models are downloaded

### Performance Issues

If you experience performance issues:

1. Check that you're running the release build (not debug)
2. Verify compute device settings (CPU/CUDA/ZLUDA)
3. Check available system memory

### Models Not Found

If Whisper or CTranslate2 models aren't found:

1. Models should be in: `%APPDATA%/VRCT/models/`
2. Re-download models from the UI if needed
3. Check disk space

## Feature Parity

All features from the Python version are available in the Rust version:

| Feature | Python | Rust |
|---------|--------|------|
| Audio Device Enumeration | ✅ | ✅ |
| Microphone Transcription | ✅ | ✅ |
| Speaker Transcription | ✅ | ✅ |
| Whisper Models | ✅ | ✅ |
| CTranslate2 Translation | ✅ | ✅ |
| API Translators (DeepL, OpenAI, etc.) | ✅ | ✅ |
| OSC Communication | ✅ | ✅ |
| WebSocket Server | ✅ | ✅ |
| VR Overlay | ✅ | ✅ |
| Japanese Transliteration | ✅ | ✅ |
| Word Filtering | ✅ | ✅ |
| Logging | ✅ | ✅ |
| ZLUDA Support | ✅ | ✅ |
| Auto-Update | ✅ | ✅ |

## For Developers

### Building from Source

**Before (Python version):**
```bash
npm run setup-python
npm run build-python
npm run build
```

**After (Rust version):**
```bash
npm install
npm run build
```

### Development Workflow

**Before:**
```bash
npm run dev  # Starts Python backend + Tauri
```

**After:**
```bash
npm run dev  # Starts Rust backend + Tauri (faster)
```

### Dependencies

**Before:**
- Python 3.11
- PyInstaller
- NumPy, PyTorch, etc.

**After:**
- Rust toolchain (rustup)
- Cargo (Rust package manager)

### Code Location

**Before:**
- Backend: `src-python/`
- Frontend: `src-ui/`

**After:**
- Backend: `src-tauri/src/`
- Frontend: `src-ui/`

### Testing

**Before:**
```bash
python -m pytest src-python/
```

**After:**
```bash
cd src-tauri
cargo test
```

## Benefits of Migration

### For Users
1. **Faster Performance**: Native Rust code runs significantly faster
2. **Lower Memory Usage**: More efficient memory management
3. **Simpler Installation**: No Python installation required
4. **Smaller Download**: Single executable vs Python + dependencies
5. **Better Reliability**: Type-safe code reduces runtime errors

### For Developers
1. **Better Type Safety**: Rust's type system catches errors at compile time
2. **Modern Tooling**: Cargo, rustfmt, clippy
3. **Easier Debugging**: Better error messages and stack traces
4. **Faster Iteration**: Faster compile times than PyInstaller
5. **Better Documentation**: Rust's doc system

## Getting Help

If you encounter issues during migration:

1. **Check the Issues**: [GitHub Issues](https://github.com/misyaguziya/VRCT/issues)
2. **Read the Docs**: [Documentation](https://mzsoftware.notion.site/VRCT-Documents-be79b7a165f64442ad8f326d86c22246?pvs=4)
3. **Ask for Help**: Create a new issue with:
   - Your OS version
   - VRCT version
   - Steps to reproduce the problem
   - Error messages or logs

## Rollback

If you need to rollback to the Python version:

1. Download the last Python-based release from [Releases](https://github.com/misyaguziya/VRCT/releases)
2. Your configuration file will still work
3. Report the issue so we can fix it in the Rust version

## Future Updates

The Rust version will receive all future updates and improvements. The Python version is no longer maintained.

---

Thank you for using VRCT! We hope the new Rust version provides a better experience. 🎉
