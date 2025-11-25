# Auralis Audio

**The 1-Click Audio Mesh for Linux**

A native GTK4 application that makes managing multiple audio devices as simple as drag-and-drop. Auralis brings visual, intuitive audio routing to PipeWire on Fedora and other modern Linux distributions.

---

## ✨ Features

### 🎯 Visual Device Clustering
Drag and drop audio devices to create synchronized multi-device playback groups. No terminal commands, no complex configurations—just drag, drop, and play.

### 🔊 N-Way Synchronization
Combine unlimited audio devices into a single logical output. Perfect for:
- Multi-room audio setups
- Synchronized speakers across different devices
- Creating complex audio routing scenarios

### 🎨 Beautiful Native Interface
Built with GTK4 and Libadwaita for a seamless Fedora desktop experience. Features:
- Dark mode support
- Smooth animations
- Intuitive drag-and-drop interface
- Real-time device discovery

### ⚡ PipeWire Native
Deep integration with PipeWire for:
- Automatic device detection
- Hot-plug support
- Low-latency audio routing
- Clean, reliable connections

---

## 🚀 Getting Started

### Prerequisites

- **Operating System:** Fedora Workstation 38+ (or compatible Linux)
- **Audio System:** PipeWire
- **Display Server:** Wayland or X11
- **Rust:** 1.70 or later

### Building from Source

```bash
# Clone the repository
git clone https://github.com/Trex099/AURALIS-AUDIO.git
cd AURALIS-AUDIO

# Build release version
cargo build --release

# Run the application
cargo run --release -p auralis-ui
```

### Running Tests

```bash
cargo test
```

All 15 unit tests should pass successfully.

---

## 🎮 Usage

1. **Launch Auralis**
   ```bash
   cargo run --release -p auralis-ui
   ```

2. **Discover Devices**
   - Available audio devices appear automatically in the device list
   - Both physical hardware and application streams are detected

3. **Create a Cluster**
   - Drag a device icon from the list
   - Drop it into the clustering zone
   - Drag additional devices to add them to the cluster
   - Audio instantly routes to all devices in sync

4. **Separate Devices**
   - Click the "Separate" button on any cluster
   - Devices return to individual operation

---

## 🏗️ Architecture

Auralis is built as a Rust workspace with clear separation of concerns:

```
auralis-audio/
├── auralis-core/      # PipeWire integration & audio logic
├── auralis-ui/        # GTK4/Libadwaita interface
├── auralis-net/       # Network features (planned)
├── auralis-web/       # Web client assets (planned)
└── auralis-cli/       # Command-line interface (planned)
```

### Technology Stack

- **Language:** Rust
- **UI Framework:** GTK4 / Libadwaita
- **Audio Backend:** PipeWire (`pipewire-rs`)
- **Async Runtime:** Tokio
- **State Management:** Thread-safe shared state with Mutex

---

## 🛣️ Roadmap

### Current Features (Implemented)
- ✅ Visual device discovery
- ✅ Drag-and-drop clustering
- ✅ Multi-device synchronization
- ✅ Real-time device hot-plug
- ✅ Cluster management (create/separate)

### Planned Features
- 🔄 **Latency Compensation** - Automatic delay adjustment for perfect sync
- 🔄 **Physics-Based UI** - Floating orbs with orbital animations
- 🔄 **Beam Mode** - Stream audio to phones/tablets via WebRTC
- 🔄 **Context-Aware Routing** - Auto-detect calls and route appropriately
- 🔄 **Advanced Mode** - Direct PipeWire graph visualization

---

## 🤝 Contributing

Contributions are welcome! This project is in active development.

### Development Setup

```bash
# Install dependencies (Fedora)
sudo dnf install gtk4-devel libadwaita-devel pipewire-devel

# Build in debug mode for development
cargo build

# Run with logging
RUST_LOG=debug cargo run -p auralis-ui
```

### Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Ensure all tests pass with `cargo test`
- Keep commits atomic and well-described

---

## 📋 Requirements

### Runtime Dependencies
- PipeWire (audio server)
- GTK4 (>= 4.0)
- Libadwaita (>= 1.0)
- PulseAudio compatibility layer (for `pactl`)

### Build Dependencies
- Rust toolchain (>= 1.70)
- `pkg-config`
- GTK4 development files
- Libadwaita development files
- PipeWire development files

### Installation (Fedora)
```bash
sudo dnf install gtk4-devel libadwaita-devel pipewire-devel pkg-config
```

---

## 🐛 Known Issues

- Latency compensation is not yet implemented (devices may have slight sync delays)
- Physics simulation is partially implemented (orbs don't float yet)
- Beam mode (WebRTC streaming) is planned but not started

See [Issues](https://github.com/Trex099/AURALIS-AUDIO/issues) for current bugs and feature requests.

---

## 📜 License

Copyright © 2025 Auralis Audio Contributors

[Add your preferred license here - MIT, GPL, Apache 2.0, etc.]

---

## 🙏 Acknowledgments

- **PipeWire** - For the excellent modern Linux audio architecture
- **GTK/GNOME** - For the amazing UI toolkit
- **The Rust Community** - For the incredible ecosystem

---

## 📞 Contact & Support

- **Issues:** [GitHub Issues](https://github.com/Trex099/AURALIS-AUDIO/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Trex099/AURALIS-AUDIO/discussions)

---

<div align="center">

**Made with ❤️ for Linux audio enthusiasts**

[Report Bug](https://github.com/Trex099/AURALIS-AUDIO/issues) · [Request Feature](https://github.com/Trex099/AURALIS-AUDIO/issues) · [Documentation](https://github.com/Trex099/AURALIS-AUDIO/wiki)

</div>
