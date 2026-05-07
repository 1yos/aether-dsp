# Workaround Options for Building Aether UI on Windows

The Visual Studio Build Tools installation is having issues. Here are alternative approaches:

## Option 1: Download Pre-built Binary from GitHub Actions ⭐ EASIEST

Since your CI builds successfully on Windows with MSVC, you can download the pre-built binary:

### Steps:

1. Go to your GitHub repository
2. Click on **Actions** tab
3. Find a successful **CI** workflow run on Windows
4. Download the build artifacts
5. Extract and run `aether-studio.exe`

**Pros:** No installation needed, works immediately  
**Cons:** Need to rebuild on CI for every change

## Option 2: Use WSL2 (Windows Subsystem for Linux) ⭐ RECOMMENDED

Build on Linux inside Windows - no MSVC needed!

### Steps:

```powershell
# Install WSL2 (if not already installed)
wsl --install -d Ubuntu

# After restart, open Ubuntu terminal and run:
sudo apt update
sudo apt install build-essential pkg-config libasound2-dev curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Navigate to your project (Windows drives are mounted at /mnt/)
cd /mnt/d/Audio\ kernel/aether-dsp

# Build
cargo build --release -p aether-ui

# Binary will be at:
# /mnt/d/Audio kernel/aether-dsp/target/release/aether-studio
```

**Pros:** Fast, reliable, matches CI environment  
**Cons:** Requires WSL2 installation (~1 GB)

## Option 3: Use Docker

Build in a containerized Linux environment:

```powershell
# Create Dockerfile in project root
docker build -t aether-builder .
docker run -v "${PWD}:/workspace" aether-builder cargo build --release -p aether-ui
```

**Pros:** Isolated, reproducible builds  
**Cons:** Requires Docker Desktop

## Option 4: Manual VS Build Tools Installation (What We Tried)

If the automated installers fail, try manual installation:

### Method A: Visual Studio Installer

1. Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
2. Run `vs_BuildTools.exe`
3. Select **"Desktop development with C++"**
4. Install (~7 GB, 10-15 minutes)

### Method B: Visual Studio Community (Full IDE)

1. Download from: https://visualstudio.microsoft.com/vs/community/
2. During installation, select **"Desktop development with C++"**
3. Install (~10 GB, 15-20 minutes)

**After installation:**

```powershell
# Restart PowerShell, then:
cd "d:\Audio kernel\aether-dsp"
.\scripts\build_ui.ps1
```

## Option 5: Build Core Crates Only (No UI)

If you only need the DSP engine (not the GUI):

```powershell
# These build fine with MinGW
cargo build --release -p aether-core
cargo build --release -p aether-nodes
cargo build --release -p aether-host
cargo build --release -p aether-cli

# Run the WebSocket server (no GUI)
cargo run --release -p aether-host
```

Then connect to `ws://127.0.0.1:9001` from:

- Web browser with JavaScript
- Python script
- Any WebSocket client

**Pros:** Works with your current MinGW setup  
**Cons:** No native GUI

## Option 6: Use GitHub Codespaces

Build in the cloud:

1. Go to your GitHub repository
2. Click **Code** → **Codespaces** → **Create codespace**
3. Wait for environment to load
4. Run: `cargo build --release -p aether-ui`
5. Download the binary

**Pros:** No local setup needed  
**Cons:** Requires GitHub account, limited free hours

## Recommended Path Forward

**For immediate testing:**
→ **Option 1** (Download from CI) or **Option 2** (WSL2)

**For long-term development:**
→ **Option 2** (WSL2) - Best balance of convenience and performance

**If you must use native Windows:**
→ **Option 4** (Manual VS installation) - Keep trying different installers

## Why This Is Happening

The Iced UI framework has 500+ dependencies. When linking, the command line becomes extremely long:

- **MinGW's `ld` linker:** Has a command-line length limit → fails with exit code 53
- **MSVC's `link.exe`:** Handles long command lines correctly → works

This is a known limitation of MinGW on Windows for large Rust GUI projects.

## Current Project Status

✅ Code is correct (CI passes on all platforms)  
✅ Rust toolchain switched to MSVC  
✅ `.cargo/config.toml` updated  
✅ Build artifacts cleaned  
❌ Visual Studio Build Tools installation failed

The only blocker is getting `link.exe` (MSVC linker) installed on your system.

## Need Help?

If you choose Option 2 (WSL2), I can help you set it up step-by-step.  
If you choose Option 4 (Manual installation), let me know if you encounter specific errors.

---

**Quick Decision Matrix:**

| Need                   | Best Option                  |
| ---------------------- | ---------------------------- |
| Test UI now            | Option 1 (Download from CI)  |
| Develop regularly      | Option 2 (WSL2)              |
| Only need DSP engine   | Option 5 (Core crates only)  |
| Have slow internet     | Option 4 (Manual VS install) |
| Want cloud development | Option 6 (Codespaces)        |
