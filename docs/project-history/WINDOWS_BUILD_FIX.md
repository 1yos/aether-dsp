# Windows Build Fix Guide

## Problem

The application fails to link on Windows with MinGW due to command line length limitations and library conflicts.

## Solutions (Choose One)

### ✅ Solution 1: Install Visual Studio Build Tools (RECOMMENDED)

This is the **official and recommended** way to build Rust applications on Windows.

1. **Download Visual Studio Build Tools**:
   - Go to: https://visualstudio.microsoft.com/downloads/
   - Scroll down to "Tools for Visual Studio"
   - Download "Build Tools for Visual Studio 2022"

2. **Install with C++ Support**:
   - Run the installer
   - Select "Desktop development with C++"
   - Click Install (requires ~7GB)

3. **Switch Rust Toolchain**:

   ```powershell
   rustup default stable-msvc
   ```

4. **Build**:
   ```powershell
   cargo clean
   cargo run --package aether-ui --bin aether-studio --release
   ```

### ✅ Solution 2: Use Pre-built Binary

If you have a working build from another machine:

1. Copy the `target/release/aether-studio.exe` file
2. Copy any required DLLs from MinGW (if needed)
3. Run directly

### ✅ Solution 3: Build on Linux/WSL

Windows Subsystem for Linux provides a clean Linux environment:

1. **Install WSL2**:

   ```powershell
   wsl --install
   ```

2. **Inside WSL, install dependencies**:

   ```bash
   sudo apt update
   sudo apt install -y build-essential pkg-config libasound2-dev \
       libx11-dev libxcursor-dev libxrandr-dev libxi-dev \
       libgl1-mesa-dev libglu1-mesa-dev

   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

3. **Build**:

   ```bash
   cd /mnt/d/Audio\ kernel/aether-dsp
   cargo build --release --package aether-ui --bin aether-studio
   ```

4. **Run with X Server** (install VcXsrv or X410 on Windows):
   ```bash
   export DISPLAY=:0
   ./target/release/aether-studio
   ```

### ⚠️ Solution 4: Fix MinGW Linking (Advanced)

If you must use MinGW, try these workarounds:

#### Option A: Reduce Dependencies

Edit `Cargo.toml` to use fewer features:

```toml
[dependencies]
iced = { version = "0.13", features = ["canvas", "wgpu"], default-features = false }
```

#### Option B: Use LLD Linker

```powershell
# Install LLVM
# Download from: https://releases.llvm.org/

# Add to .cargo/config.toml
[target.x86_64-pc-windows-gnu]
linker = "lld-link.exe"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

#### Option C: Increase Command Line Buffer

Edit `.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-gnu]
rustflags = [
    "-C", "link-arg=-Wl,--no-as-needed",
    "-C", "link-arg=-Wl,--gc-sections",
]
```

## Why This Happens

MinGW on Windows has several limitations:

1. **Command Line Length**: Windows has a 32KB command line limit, and MinGW's linker can exceed this with many libraries
2. **Library Conflicts**: MinGW and Windows system libraries can conflict
3. **Path Issues**: Spaces in paths (like "D:\Audio kernel\") cause problems
4. **DLL Hell**: MinGW requires specific runtime DLLs that may conflict

MSVC doesn't have these issues because it's the native Windows toolchain.

## Cross-Platform Development

For true cross-platform development:

- **Windows**: Use MSVC toolchain
- **macOS**: Use default toolchain (works out of the box)
- **Linux**: Use default toolchain (works out of the box)

All three will build the same codebase without issues.

## CI/CD Recommendation

For automated builds:

```yaml
# GitHub Actions example
jobs:
  build:
    strategy:
      matrix:
        os: [windows-latest, macos-latest, ubuntu-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          # Windows will automatically use MSVC
      - run: cargo build --release
```

## Quick Start for Impatient Users

**Just want it to work?**

```powershell
# Install Visual Studio Build Tools (one-time, ~10 minutes)
# Download from: https://visualstudio.microsoft.com/downloads/

# Then:
rustup default stable-msvc
cargo clean
cargo run --release --package aether-ui --bin aether-studio
```

That's it! The application will build and run.

---

**TL;DR**: Install Visual Studio Build Tools and use MSVC. It's the standard way to build Rust on Windows and avoids all MinGW issues.
