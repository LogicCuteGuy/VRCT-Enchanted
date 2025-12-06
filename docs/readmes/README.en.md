<div align="center">

<picture>
    <source srcset="/docs/img/vrct_logo_white.png" media="(prefers-color-scheme: dark)" width="50%">
    <source srcset="/docs/img/vrct_logo_black.png" media="(prefers-color-scheme: light)" width="50%">
    <img src="/docs/img/vrct_logo.png" alt="VRCT Logo" width="50%">
</picture>

<br>
<br>

[![GitHub release](https://img.shields.io/github/v/release/misyaguziya/VRCT.svg)](https://github.com/misyaguziya/VRCT/releases)
[![Downloads](https://img.shields.io/github/downloads/misyaguziya/VRCT/total)](https://github.com/misyaguziya/VRCT/releases)
[![Licence](https://img.shields.io/github/license/misyaguziya/VRCT)](https://github.com/misyaguziya/VRCT/blob/master/LICENSE)
[![Booth](https://img.shields.io/badge/Store-Booth.pm-red)](https://misyaguziya.booth.pm/items/5155325)
[![Github Sponsors](https://img.shields.io/badge/GitHub%20Sponsors-30363D?&logo=GitHub-Sponsors&logoColor=EA4AAA)](https://github.com/sponsors/misyaguziya)

<h3>
Become a VRCT Supporter on:
</h3>

<a href="https://vrct-dev.fanbox.cc">
    <picture>
        <source srcset="/docs/img/pixiv_fanbox_white.png" media="(prefers-color-scheme: dark)" height="18px">
        <source srcset="/docs/img/pixiv_fanbox_black.png" media="(prefers-color-scheme: light)" height="18px">
        <img src="/docs/img/pixiv_fanbox_black.png" alt="PIXIV FANBOX" height="18px">
    </picture>
</a>&emsp;&nbsp;

<a href="https://patreon.com/vrct_dev">
    <picture>
        <source srcset="/docs/img/patreon_logo_white.png" media="(prefers-color-scheme: dark)" height="22px">
        <source srcset="/docs/img/patreon_logo_black.png" media="(prefers-color-scheme: light)" height="22px">
        <img src="/docs/img/patreon_logo_black.png" alt="Patreon" height="22px">
    </picture>
</a>&emsp;&nbsp;

<a href="https://ko-fi.com/vrct_dev">
    <picture>
        <img src="/docs/img/kofi_logo.png" alt="Ko-fi" height="22px">
    </picture>
</a>&emsp;&nbsp;

<br>

<picture>
    <source srcset="/docs/img/supporter_section_border_d.png" media="(prefers-color-scheme: dark)">
    <source srcset="/docs/img/supporter_section_border_l.png" media="(prefers-color-scheme: light)">
    <img src="/docs/img/supporter_section_border_d.png" alt="Supporter Section Border">
</picture>

<br>
<br>

| **English** | [日本語](/docs/readmes/README.ja.md) | [한국어](/docs/readmes/README.ko.md) | [繁體中文](/docs/readmes/README.zh-Hant.md) |

<h3>
VRCT is software that supports VRChat conversations with translation and transcription.
</h3>

![](/docs/img/main_window.png)

<div align="left">

# Download & Install
Download from anywhere you like.
- [Github.com](https://github.com/misyaguziya/VRCT/releases/)
- [BOOTH.pm](https://misyaguziya.booth.pm/items/5155325)

Just download and run the exe.

## System Requirements

### For Users
- **OS**: Windows 10 or later (64-bit)
- **Memory**: 4GB RAM minimum, 8GB recommended
- **Storage**: 500MB for application + models
- **Self-Contained**: Single executable with no external dependencies

### For Developers
- **Rust**: Latest stable toolchain (install via [rustup](https://rustup.rs/))
- **Node.js**: Version 18 or later
- **Build Tools**: 
  - Windows: Visual Studio Build Tools with C++ support
  - Or: Visual Studio 2019/2022 with "Desktop development with C++" workload

# What is VRCT?
VRCT is software that supports conversations between people who speak different languages by providing chat or voice translation.
These features are designed for use within VRChat.
*Although not supported, it is also used for other purposes such as watching movies.

VRCT supports your conversations with
- 💬 **Send chat to VRChat**
- 🌐 **Translation**
- 🎙 **Transcription of audio from microphone**
- 🔈 **Transcription of audio from Speaker**

# Documentation

### For Users
- [User Documentation](https://mzsoftware.notion.site/VRCT-Documents-be79b7a165f64442ad8f326d86c22246?pvs=4) - Setup, features, and usage
- [Migration Guide](MIGRATION_GUIDE.md) - Upgrading from legacy versions

### For Developers
- [Architecture Documentation](ARCHITECTURE.md) - Technical architecture and design
- [Contributing Guide](CONTRIBUTING.md) - Development setup and guidelines
- [Build Instructions](#building-from-source) - See below for quick start

# How to Use (YouTube)
<div align="center">

[![](https://img.youtube.com/vi/rUTad037n8Q/0.jpg)](https://www.youtube.com/watch?v=rUTad037n8Q)

<div align="left">

## Technology Stack

VRCT is built with modern, high-performance technologies:

### Core Technologies
- **Backend**: Pure Rust for high performance and reliability
- **Frontend**: React + JavaScript
- **Framework**: Tauri 2.x
- **Build System**: Cargo + Vite

### Key Libraries
- **Audio**: `cpal` - Cross-platform audio I/O
- **ML/AI**: `candle` - Rust ML framework for Whisper transcription
- **Async Runtime**: `tokio` - High-performance async runtime
- **Networking**: `reqwest`, `axum` - HTTP client and web framework
- **Protocols**: `rosc` (OSC), `tokio-tungstenite` (WebSocket)
- **Serialization**: `serde` - Type-safe serialization

### Key Benefits of Rust Architecture
- ⚡ **Fast startup time** - Native code execution
- 💾 **Efficient memory usage** - Optimized memory management
- 📦 **Single self-contained executable** - No external dependencies required
- 🔒 **Enhanced type safety** - Compile-time error detection
- 🚀 **Native performance** - Zero-cost abstractions
- 🛡️ **Better reliability** - Memory safety without garbage collection

## Building from Source

### Prerequisites
1. Install [Rust](https://rustup.rs/) (latest stable)
2. Install [Node.js](https://nodejs.org/) (v18+)
3. Install build tools:
   - Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) with C++ support

### Build Steps
```bash
# Clone the repository
git clone https://github.com/misyaguziya/VRCT.git
cd VRCT

# Install dependencies
npm install

# Development build (with hot reload)
npm run dev

# Production build
npm run build
```

### Build Output
- Executable: `src-tauri/target/release/VRCT.exe`
- Installer: `src-tauri/target/release/bundle/nsis/VRCT_x.x.x_x64-setup.exe`

### Troubleshooting Build Issues
- **Rust not found**: Run `rustup update stable`
- **Link errors**: Ensure Visual Studio Build Tools are installed
- **Out of memory**: Close other applications or increase virtual memory
- **Slow builds**: First build takes longer; subsequent builds are faster

For more details, see [CONTRIBUTING.md](CONTRIBUTING.md)

## Author
- [みしゃ(misyaguzi)](https://github.com/misyaguziya) (Main Development)
- [しいな(Shiina_12siy)](https://twitter.com/Shiina_12siy) (UI/UX, UI multilingual support)
- [レラ](https://github.com/soumt-r) (Technical Advisor)
- [どね](https://twitter.com/done_vrc) (Logo Design)

## Thanks to our contributors
<a href="https://github.com/misyaguziya/VRCT/graphs/contributors" target="_blank">
  <img src="https://contrib.rocks/image?repo=misyaguziya/VRCT" />
</a>

---

VRCT is not endorsed by VRChat and does not reflect the views or opinions of VRChat or anyone officially involved in producing or managing VRChat properties. VRChat and all associated properties are trademarks or registered trademarks of VRChat Inc. VRChat © VRChat Inc.