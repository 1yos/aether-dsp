# AetherDSP v0.3 Release Notes

## 🎉 Major Release: Native GPU-Accelerated DAW

AetherDSP v0.3 marks a significant milestone with a complete, production-ready DAW built with Iced (GPU-accelerated native UI using wgpu). This release removes the React/Tauri prototype and delivers a professional music production environment.

---

## ✨ New Features

### 🎹 Complete DAW Interface

**Song View**

- Multi-track timeline with clip-based arrangement
- Drag, resize, split, and color-code clips
- Configurable grid snapping (1/4, 1/8, 1/16, 1/32 notes)
- Track controls: volume, pan, mute, solo, arm
- Add/remove/reorder tracks
- Inline track renaming

**Piano Roll**

- Full MIDI note editor with velocity lanes
- Draw, select, move, resize, and delete notes
- Quantization with configurable snap values
- Transposition (semitone and octave)
- Scale highlighting
- Keyboard input support (play notes with PC keyboard)

**Mixer**

- Per-track volume faders and pan controls
- VU meters with peak hold tracking
- Insert effects chain (EQ, Compressor, Reverb, Delay)
- Visual feedback for all parameters

**Transport Controls**

- Play, stop, record buttons
- Loop region support
- Metronome with downbeat accent (1200Hz downbeat, 800Hz beat)
- Dual time display: bars:beats + mm:ss.cs
- BPM control with tap tempo
- Real-time position tracking

### 💾 Project Management

**Save/Load System**

- Complete project serialization (tracks, clips, MIDI notes, mixer settings, effects)
- Atomic file writes prevent corruption
- `.aether-project` file format
- Preserves all DAW state including:
  - Track configuration and routing
  - MIDI note data with velocity
  - Mixer settings (volume, pan, mute, solo)
  - Effect chains and parameters
  - Transport settings (BPM, loop points)

### 🎵 Audio Export

**WAV Rendering**

- Offline rendering for high-quality export
- 48kHz sample rate, 16-bit stereo
- Progress tracking during export
- Export entire project or loop region
- Uses `hound` crate for WAV file writing

### 🎼 Metronome System

**DSP-Based Click Generator**

- Tempo-synced to project BPM
- Downbeat accent (1200Hz) vs regular beats (800Hz)
- Smooth envelope (5ms attack, 50ms release)
- Toggle on/off from transport bar

### 🎨 UI Polish

**Professional Time Display**

- Dual format: bars:beats + mm:ss.cs
- Real-time updates during playback
- Accurate to centisecond precision

**VU Meters**

- Peak hold with 2-second decay
- Color-coded levels (green → yellow → red)
- Per-track metering

---

## 🗑️ Removed

- **React/Tauri UI Prototype** - The `ui/` directory has been completely removed
  - 57 instrument presets (moved to `assets/instruments/`)
  - React components and hooks
  - Tauri desktop wrapper
  - WebGL node graph renderer
  - All TypeScript/JavaScript code

---

## 🔧 Technical Improvements

### Cross-Platform Build Support

**Windows (MSVC Recommended)**

```powershell
rustup default stable-x86_64-pc-windows-msvc
cargo build --release -p aether-ui
```

**macOS**

```bash
cargo build --release -p aether-ui
```

**Linux**

```bash
sudo apt install build-essential pkg-config libasound2-dev
cargo build --release -p aether-ui
```

### Known Issues

- **Windows MinGW**: Cannot build GUI applications due to linker command-line length limitations
  - **Solution**: Use MSVC toolchain (Visual Studio Build Tools)
  - See `BUILD_GUIDE.md` for detailed instructions

### Dependencies

- Added `hound = "3.5"` for WAV export
- Using `iced = "0.13"` with wgpu backend
- GPU acceleration via wgpu (Metal/Vulkan/DirectX 12)

---

## 📦 Crate Updates

| Crate               | Version | Status                                   |
| ------------------- | ------- | ---------------------------------------- |
| `aether-ui`         | 0.1.1   | **NEW** - Native GPU DAW (not published) |
| `aetherdsp-core`    | 0.1.1   | Stable                                   |
| `aetherdsp-nodes`   | 0.2.0   | Stable                                   |
| `aetherdsp-ndk`     | 0.1.1   | Stable                                   |
| `aetherdsp-midi`    | 0.1.1   | Stable                                   |
| `aetherdsp-sampler` | 0.2.0   | Stable                                   |

---

## 🚀 Getting Started

### Build and Run

```bash
# Clone the repository
git clone https://github.com/1yos/aether-dsp.git
cd aether-dsp

# Build the DAW
cargo build --release -p aether-ui

# Run the DAW
cargo run --release -p aether-ui
# Or directly:
./target/release/aether-studio
```

### Quick Test

1. Launch Aether Studio
2. Click "Add Track" to create a new track
3. Click in the timeline to create a clip
4. Double-click the clip to open Piano Roll
5. Draw some notes
6. Press Play to hear your music!

---

## 📚 Documentation

- **BUILD_GUIDE.md** - Comprehensive cross-platform build instructions
- **WINDOWS_BUILD_FIX.md** - Windows-specific troubleshooting
- **IMPLEMENTATION_COMPLETE.md** - Feature documentation
- **README.md** - Updated to reflect v0.3 changes

---

## 🎯 Roadmap

| Version | Milestone                                                       |
| ------- | --------------------------------------------------------------- |
| ✅ v0.1 | RT engine + WebSocket bridge + 9 crates on crates.io            |
| ✅ v0.2 | 15 DSP nodes, modulation matrix, sample library                 |
| ✅ v0.3 | **Native GPU DAW with Song view, Piano roll, Mixer, Transport** |
| 🔜 v0.4 | SIMD optimization, world music instruments, tuning systems      |
| 🔜 v1.0 | Stable public release                                           |

---

## 🙏 Acknowledgments

Built with:

- [Iced](https://github.com/iced-rs/iced) - GPU-accelerated UI framework
- [wgpu](https://github.com/gfx-rs/wgpu) - Cross-platform graphics API
- [CPAL](https://github.com/RustAudio/cpal) - Cross-platform audio I/O
- [hound](https://github.com/ruuda/hound) - WAV encoding/decoding

---

## 📄 License

MIT - see [LICENSE](LICENSE)

---

## 🔗 Links

- **GitHub**: https://github.com/1yos/aether-dsp
- **Crates.io**: https://crates.io/crates/aetherdsp-core
- **Documentation**: https://docs.rs/aether-core

---

**Commit**: `dd5189a` - feat: Complete DAW v0.3 with native Iced UI and remove React prototype

**Date**: May 7, 2026
