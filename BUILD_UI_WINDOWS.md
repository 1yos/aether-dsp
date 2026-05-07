# Building Aether UI on Windows

## Quick Start (After Installing Build Tools)

```powershell
# Use the build script (recommended)
.\scripts\build_ui.ps1

# Or manually
cargo build --release -p aether-ui
```

## Prerequisites

### 1. MSVC Toolchain (✅ Already configured)

Your Rust is now set to use MSVC:

```powershell
rustup default stable-x86_64-pc-windows-msvc
```

### 2. Visual Studio Build Tools (⏳ Installing now)

**What you need to install:**

- ✅ **Desktop development with C++** workload

**This includes:**

- MSVC v143 - VS 2022 C++ x64/x86 build tools
- Windows 11 SDK (10.0.22621.0 or later)
- C++ CMake tools for Windows
- Testing tools core features

**Installation size:** ~7 GB  
**Time:** 10-15 minutes

### 3. After Installation

**Restart your terminal/PowerShell** to pick up new environment variables.

Then verify:

```powershell
# Check toolchain
rustup show
# Should show: stable-x86_64-pc-windows-msvc (default)

# Check MSVC compiler (should work after restart)
where.exe cl.exe
# Should show: C:\Program Files\Microsoft Visual Studio\...\cl.exe
```

## Building

### Option 1: Use the build script (recommended)

```powershell
.\scripts\build_ui.ps1
```

This script:

- Verifies toolchain is MSVC
- Checks for Visual Studio Build Tools
- Builds the UI
- Shows build time and binary location

### Option 2: Manual build

```powershell
# Clean build (if switching from MinGW)
cargo clean

# Build release
cargo build --release -p aether-ui

# Run
cargo run --release -p aether-ui
```

## Binary Location

After successful build:

```
C:\aether-target\release\aether-studio.exe
```

## Troubleshooting

### Error: `link.exe` not found

**Cause:** Visual Studio Build Tools not installed or not in PATH

**Fix:**

1. Complete the Build Tools installation
2. Restart your terminal
3. Run `where.exe cl.exe` to verify

### Error: Mixed toolchain artifacts

**Cause:** Built with MinGW, now using MSVC (or vice versa)

**Fix:**

```powershell
cargo clean
cargo build --release -p aether-ui
```

### Error: Still using GNU toolchain

**Cause:** Toolchain not switched

**Fix:**

```powershell
rustup default stable-x86_64-pc-windows-msvc
rustup show  # Verify
```

### Build is very slow

**Normal:** First build takes 5-10 minutes (compiling 500+ dependencies)  
**Subsequent builds:** 30 seconds - 2 minutes (incremental compilation)

## Why MSVC Instead of MinGW?

The Iced UI framework has 500+ dependencies. MinGW's linker (`ld`) has a command-line length limit that causes it to fail with:

```
error: ld returned 53 exit status
```

MSVC's linker (`link.exe`) handles this correctly. This is a known limitation of MinGW on Windows for large Rust projects.

## Configuration Changes Made

### `.cargo/config.toml`

- ✅ MSVC target enabled
- ❌ MinGW target commented out (to prevent conflicts)

### Rust toolchain

- ✅ Default: `stable-x86_64-pc-windows-msvc`
- ❌ Previous: `stable-x86_64-pc-windows-gnu`

## CI/CD

GitHub Actions uses MSVC on Windows and builds successfully. Your local setup now matches CI.

## Next Steps

1. ✅ Wait for Visual Studio Build Tools installation to complete
2. ✅ Restart your terminal/PowerShell
3. ✅ Run `.\scripts\build_ui.ps1`
4. ✅ Launch `C:\aether-target\release\aether-studio.exe`

---

**Need help?** Check the installer window is still running. The installation can take 10-15 minutes.
