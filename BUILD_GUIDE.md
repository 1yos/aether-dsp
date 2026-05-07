# Cross-Platform Build Guide for Aether DSP

This guide explains how to build and run Aether DSP on Windows, macOS, and Linux.

---

## 🪟 Windows

### Prerequisites

- **Rust** (with MSVC toolchain recommended)
- **Visual Studio Build Tools** or **Visual Studio** with C++ support

### Setup

1. **Install Rust with MSVC toolchain** (recommended):

```powershell
# If you already have Rust, add MSVC toolchain
rustup toolchain install stable-msvc

# Set MSVC as default
rustup default stable-msvc
```

2. **Install Visual Studio Build Tools**:
   - Download from: https://visualstudio.microsoft.com/downloads/
   - Select "Desktop development with C++"
   - Or install full Visual Studio with C++ workload

### Build and Run

```powershell
# Clone the repository
git clone https://github.com/yourusername/aether-dsp.git
cd aether-dsp

# Build and run (debug)
cargo run --package aether-ui --bin aether-studio

# Build and run (release - faster)
cargo run --package aether-ui --bin aether-studio --release

# Build only
cargo build --release --package aether-ui --bin aether-studio

# Run the built binary
.\target\release\aether-studio.exe
```

### Alternative: MinGW (Not Recommended)

MinGW has known linker issues with some dependencies. If you must use it:

```powershell
# Install MinGW toolchain
rustup toolchain install stable-gnu
rustup default stable-gnu

# Install MSYS2 from https://www.msys2.org/
# Then install MinGW-w64:
pacman -S mingw-w64-x86_64-gcc

# Build (may fail with linker errors)
cargo build --release
```

---

## 🍎 macOS

### Prerequisites

- **Rust**
- **Xcode Command Line Tools**

### Setup

1. **Install Xcode Command Line Tools**:

```bash
xcode-select --install
```

2. **Install Rust**:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Build and Run

```bash
# Clone the repository
git clone https://github.com/yourusername/aether-dsp.git
cd aether-dsp

# Build and run (debug)
cargo run --package aether-ui --bin aether-studio

# Build and run (release - faster)
cargo run --package aether-ui --bin aether-studio --release

# Build only
cargo build --release --package aether-ui --bin aether-studio

# Run the built binary
./target/release/aether-studio
```

### Apple Silicon (M1/M2/M3) Notes

The build will automatically use the `aarch64-apple-darwin` target. Native performance optimizations are enabled via `target-cpu=native` in `.cargo/config.toml`.

---

## 🐧 Linux

### Prerequisites

- **Rust**
- **Build essentials** (gcc, make, etc.)
- **ALSA development libraries** (for audio)
- **X11/Wayland development libraries** (for GUI)

### Setup

#### Ubuntu/Debian

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install dependencies
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libasound2-dev \
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libgl1-mesa-dev \
    libglu1-mesa-dev
```

#### Fedora/RHEL

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install dependencies
sudo dnf install -y \
    gcc \
    gcc-c++ \
    make \
    pkg-config \
    alsa-lib-devel \
    libX11-devel \
    libXcursor-devel \
    libXrandr-devel \
    libXi-devel \
    mesa-libGL-devel \
    mesa-libGLU-devel
```

#### Arch Linux

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install dependencies
sudo pacman -S \
    base-devel \
    alsa-lib \
    libx11 \
    libxcursor \
    libxrandr \
    libxi \
    mesa
```

### Build and Run

```bash
# Clone the repository
git clone https://github.com/yourusername/aether-dsp.git
cd aether-dsp

# Build and run (debug)
cargo run --package aether-ui --bin aether-studio

# Build and run (release - faster)
cargo run --package aether-ui --bin aether-studio --release

# Build only
cargo build --release --package aether-ui --bin aether-studio

# Run the built binary
./target/release/aether-studio
```

---

## 🚀 Performance Optimizations

### CPU-Specific Optimizations

The `.cargo/config.toml` file enables `target-cpu=native` for all platforms, which optimizes the binary for your specific CPU. This provides:

- **AVX2/FMA** instructions on modern x86_64 CPUs
- **NEON** instructions on ARM CPUs (Apple Silicon, ARM Linux)
- Better SIMD performance for audio processing

### Release vs Debug Builds

- **Debug builds** (`cargo run`): Slower, but faster to compile
- **Release builds** (`cargo run --release`): Much faster runtime, but slower to compile

For audio work, **always use release builds** to avoid audio dropouts.

---

## 🔧 Troubleshooting

### Windows: "linker 'link.exe' not found"

**Solution**: Install Visual Studio Build Tools with C++ support.

```powershell
# Or switch to MSVC toolchain if you have it
rustup default stable-msvc
```

### Windows: MinGW linker errors

**Solution**: Switch to MSVC toolchain (recommended):

```powershell
rustup default stable-msvc
cargo clean
cargo build --release
```

### macOS: "xcrun: error: invalid active developer path"

**Solution**: Install Xcode Command Line Tools:

```bash
xcode-select --install
```

### Linux: "error: failed to run custom build command for `alsa-sys`"

**Solution**: Install ALSA development libraries:

```bash
# Ubuntu/Debian
sudo apt install libasound2-dev

# Fedora
sudo dnf install alsa-lib-devel

# Arch
sudo pacman -S alsa-lib
```

### Linux: "error: failed to run custom build command for `x11`"

**Solution**: Install X11 development libraries:

```bash
# Ubuntu/Debian
sudo apt install libx11-dev libxcursor-dev libxrandr-dev libxi-dev

# Fedora
sudo dnf install libX11-devel libXcursor-devel libXrandr-devel libXi-devel

# Arch
sudo pacman -S libx11 libxcursor libxrandr libxi
```

### All Platforms: Out of memory during compilation

**Solution**: Build with fewer parallel jobs:

```bash
cargo build --release -j 2
```

Or increase system swap space.

---

## 📦 Distribution

### Creating Distributable Binaries

#### Windows

```powershell
# Build release binary
cargo build --release --package aether-ui --bin aether-studio

# Binary location
# target\release\aether-studio.exe

# Package with dependencies (if needed)
# Copy any required DLLs to the same directory
```

#### macOS

```bash
# Build release binary
cargo build --release --package aether-ui --bin aether-studio

# Binary location
# target/release/aether-studio

# Create app bundle (optional)
# Use cargo-bundle or manually create .app structure
```

#### Linux

```bash
# Build release binary
cargo build --release --package aether-ui --bin aether-studio

# Binary location
# target/release/aether-studio

# Create AppImage or .deb/.rpm package
# Use cargo-deb, cargo-generate-rpm, or AppImage tools
```

---

## 🧪 Testing

### Run Tests

```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test --package aether-core
cargo test --package aether-ui

# Run with output
cargo test -- --nocapture
```

### Run Benchmarks

```bash
# Run benchmarks
cargo bench --package aether-core
```

---

## 📝 Development Tips

### Fast Iteration

```bash
# Use cargo-watch for auto-rebuild on file changes
cargo install cargo-watch
cargo watch -x 'run --package aether-ui --bin aether-studio'
```

### Faster Compilation

Add to `~/.cargo/config.toml` (or `%USERPROFILE%\.cargo\config.toml` on Windows):

```toml
[build]
# Use all CPU cores
jobs = 8  # Adjust to your CPU core count

# Use faster linker (Unix only)
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### IDE Setup

- **VS Code**: Install `rust-analyzer` extension
- **CLion/IntelliJ**: Install Rust plugin
- **Vim/Neovim**: Use `rust-analyzer` with LSP

---

## 🌐 Cross-Compilation

### Windows → Linux

```powershell
# Install cross-compilation toolchain
rustup target add x86_64-unknown-linux-gnu

# Install cross (requires Docker)
cargo install cross

# Build for Linux
cross build --target x86_64-unknown-linux-gnu --release
```

### macOS → Windows

```bash
# Install cross-compilation toolchain
rustup target add x86_64-pc-windows-msvc

# Note: Requires Windows SDK (complex setup)
# Easier to build on actual Windows machine or CI
```

---

## 📚 Additional Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **Cargo Book**: https://doc.rust-lang.org/cargo/
- **Iced GUI Framework**: https://iced.rs/
- **CPAL Audio Library**: https://github.com/RustAudio/cpal

---

## 🆘 Getting Help

If you encounter issues:

1. Check this guide's troubleshooting section
2. Search existing GitHub issues
3. Create a new issue with:
   - Your OS and version
   - Rust version (`rustc --version`)
   - Full error message
   - Steps to reproduce

---

**Last Updated**: May 7, 2026  
**Tested Platforms**: Windows 11 (MSVC), macOS 14 (Apple Silicon), Ubuntu 22.04 LTS
