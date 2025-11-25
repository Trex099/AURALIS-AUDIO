# Auralis Audio

Visual audio routing for Linux. Drag and drop your audio devices to create synchronized playback across multiple speakers.

## What is this?

Ever wanted to play audio on multiple speakers at once without diving into PipeWire configs or terminal commands? That's what this does.

It's a GTK4 app that lets you visually connect audio devices together. Drag your laptop speakers onto your Bluetooth speaker, and boom—synchronized audio. No `pactl` commands to remember, no config files to edit.

## Why?

I got tired of manually creating PipeWire combine-sinks every time I wanted to play music on multiple devices. There had to be a better way. So I built this.

The goal is simple: make multi-device audio on Linux as easy as it is on proprietary systems. Drag, drop, done.

## Features

**Right now:**
- Visual device discovery (see all your audio devices)
- Drag-and-drop clustering (combine multiple devices)
- Real-time PipeWire integration
- Actually works (no segfaults anymore!)

**Eventually:**
- Automatic latency compensation (devices sometimes drift slightly)
- Physics-based orb animations (they're supposed to float and orbit)
- WebRTC streaming to phones/tablets
- Better visual feedback

## Building

You need:
- Fedora (or similar modern Linux with PipeWire)
- Rust toolchain
- GTK4 and Libadwaita development files

```bash
# Install dependencies on Fedora
sudo dnf install gtk4-devel libadwaita-devel pipewire-devel

# Clone and build
git clone https://github.com/Trex099/AURALIS-AUDIO.git
cd AURALIS-AUDIO
cargo build --release

# Run it
cargo run --release -p auralis-ui
```

## Using it

1. Start the app
2. You'll see your audio devices listed on the left
3. Drag a device into the center clustering zone
4. Drag another device on top of it
5. They combine into a cluster—audio now plays through both
6. Click "Separate" to break them apart

That's it.

## Architecture

It's a Rust workspace with these parts:

- `auralis-core` - PipeWire integration and clustering logic
- `auralis-ui` - The GTK4 interface you interact with
- `auralis-net` - Network features (not implemented yet)
- `auralis-web` - Web client for remote devices (also not done)

The core talks to PipeWire, the UI talks to the core. Standard stuff.

## Known Issues

**It's not perfect:**

- Latency compensation doesn't exist yet, so some devices might be slightly out of sync
- The physics/animation stuff is mostly stubbed out
- Currently uses `pactl` commands under the hood (should migrate to native PipeWire API)
- No packaging yet (you have to build from source)

## Contributing

If you want to help, cool. The codebase is fairly clean now—0 compiler warnings, 15 passing tests. 

Areas that need work:
- Replacing `pactl` with native PipeWire bindings
- Actually implementing latency compensation
- Making the orbs float and animate properly
- WebRTC streaming support

Standard Rust stuff applies: run `cargo fmt`, make sure tests pass, keep commits focused.

## Technical Details

**Stack:**
- Language: Rust
- UI: GTK4 + Libadwaita (native Fedora look)
- Audio: PipeWire via `pipewire-rs`
- Threading: Tokio for async, standard threads for PipeWire
- State: Shared mutexes, message passing between UI and core

**How clustering works:**
1. User drags device A onto device B
2. UI sends Connect command to core
3. Core creates a PipeWire combine-sink with both devices
4. PipeWire routes audio to the combine-sink
5. Both devices play the same audio

It's actually pretty straightforward once you understand PipeWire's module system.

## Requirements

**Runtime:**
- PipeWire (0.3+)
- GTK4
- Libadwaita
- PulseAudio compatibility layer (for `pactl` - temporary)

**Build:**
- Rust 1.70+
- pkg-config
- Development headers for GTK4, Libadwaita, PipeWire

## Testing

```bash
cargo test
```

15 unit tests covering the core data structures. No integration tests yet (PipeWire is hard to mock).

## License

Haven't decided yet. Probably MIT or GPL. Will update when I figure it out.

## Credits

Built with:
- PipeWire for the audio backend
- GTK4/Libadwaita for the UI
- Rust ecosystem for everything else

Thanks to everyone maintaining these projects.

---

Questions? Open an issue. Pull requests welcome.
