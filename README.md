# Auralis Audio - Phase 0, 1, 2 Complete

The 1-Click Audio Mesh for Linux

## Features
- ✅ Visual drag-and-drop device clustering
- ✅ Multi-device synchronized playback
- ✅ Native PipeWire integration  
- ✅ Beautiful GTK4/Libadwaita UI
- ✅ Stable shutdown (Phase 0 complete)
- ✅ Clean codebase (Phase 1 complete)
- ✅ Test infrastructure (Phase 2 complete)

## Project Status

**Completion:** ~65%  
**Phases Complete:** 3/8  
**Tests:** 15 passing  
**Build Status:** ✅ 0 warnings

### ✅ Completed
- Phase 0: Critical segfault fix
- Phase 1: Code cleanup (0 warnings)
- Phase 2: Testing infrastructure (15 tests)

### 🔄 In Progress
- Phase 3-8: See PHASE_TRACKER.md

## Building

```bash
# Build release
cargo build --release

# Run application
cargo run --release -p auralis-ui

# Run tests
cargo test
```

## Documentation

- `PHASE_TRACKER.md` - Overall progress tracking
- `TESTING.md` - Testing guide
- `Prompt.txt` - Original design specification
- `PHASE_0_COMPLETE.md` - Segfault fix details
- `PHASE_1_COMPLETE.md` - Code cleanup summary
- `PHASE_2_COMPLETE.md` - Testing infrastructure

## Requirements

- Fedora Workstation (or compatible Linux)
- PipeWire audio system
- GTK4 / Libadwaita
- Rust toolchain

## License

[Add your license]

## Development

See individual phase completion docs for detailed development notes and decisions.
