# Setup WSL2 for building Aether UI
# This is MUCH easier than installing Visual Studio Build Tools!

Write-Host "=== WSL2 Setup for Aether UI Build ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "This will:" -ForegroundColor Yellow
Write-Host "  1. Install Ubuntu on WSL2 (~500 MB download)"
Write-Host "  2. Install Rust and build dependencies in Ubuntu"
Write-Host "  3. Build the UI in Linux (no MSVC needed!)"
Write-Host ""
Write-Host "Total time: ~10 minutes"
Write-Host "Disk space: ~2 GB"
Write-Host ""

$response = Read-Host "Continue? (y/n)"
if ($response -ne 'y') {
    Write-Host "Cancelled." -ForegroundColor Red
    exit
}

Write-Host ""
Write-Host "Step 1: Installing Ubuntu on WSL2..." -ForegroundColor Green
Write-Host "This will open a new window. Please wait for it to complete."
Write-Host ""

wsl --install -d Ubuntu

Write-Host ""
Write-Host "✅ Ubuntu installed!" -ForegroundColor Green
Write-Host ""
Write-Host "Step 2: Setup build environment in Ubuntu" -ForegroundColor Green
Write-Host "Copy and paste these commands into the Ubuntu terminal:" -ForegroundColor Yellow
Write-Host ""
Write-Host @"
# Update package list
sudo apt update

# Install build dependencies
sudo apt install -y build-essential pkg-config libasound2-dev curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Navigate to project
cd /mnt/d/Audio\ kernel/aether-dsp

# Build the UI
cargo build --release -p aether-ui

# Done! Binary is at:
# /mnt/d/Audio kernel/aether-dsp/target/release/aether-studio
"@ -ForegroundColor White

Write-Host ""
Write-Host "After building, you can run it from Windows:" -ForegroundColor Cyan
Write-Host '  d:\Audio kernel\aether-dsp\target\release\aether-studio.exe' -ForegroundColor White
Write-Host ""
Write-Host "Or create a shortcut to it on your desktop!" -ForegroundColor Green
