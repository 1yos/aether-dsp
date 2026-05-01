# build_tauri.ps1
# Builds the complete Aether Studio standalone app using Tauri.
#
# USAGE:
#   .\scripts\build_tauri.ps1
#
# OUTPUT:
#   ui/src-tauri/target/release/bundle/
#     windows: AetherStudio_0.2.0_x64-setup.exe (installer)
#              AetherStudio_0.2.0_x64.msi
#     macOS:   AetherStudio_0.2.0_x64.dmg
#     linux:   aether-studio_0.2.0_amd64.deb
#              aether-studio_0.2.0_amd64.AppImage

$ErrorActionPreference = "Stop"
$env:PATH = "C:\msys64\mingw64\bin;" + $env:PATH

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Aether Studio — Tauri Build" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build aether-host in release mode
Write-Host "→ Building aether-host..." -ForegroundColor Yellow
cargo build -p aether-host --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ✗ aether-host build failed" -ForegroundColor Red
    exit 1
}
Write-Host "  ✓ aether-host built" -ForegroundColor Green

# Step 2: Copy aether-host binary to Tauri's sidecar location
# Tauri expects: ui/src-tauri/binaries/aether-host-{target-triple}
$target = "x86_64-pc-windows-gnu"
$binDir = "ui\src-tauri\binaries"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

$srcBin = "target\release\aether-host.exe"
$dstBin = "$binDir\aether-host-$target.exe"

if (Test-Path $srcBin) {
    Copy-Item $srcBin $dstBin -Force
    Write-Host "  ✓ Copied aether-host to $dstBin" -ForegroundColor Green
} else {
    Write-Host "  ✗ aether-host binary not found at $srcBin" -ForegroundColor Red
    exit 1
}

# Step 3: Build the Tauri app
Write-Host ""
Write-Host "→ Building Tauri app..." -ForegroundColor Yellow
Set-Location ui
npm run tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ✗ Tauri build failed" -ForegroundColor Red
    Set-Location ..
    exit 1
}
Set-Location ..

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Build complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Installer: ui\src-tauri\target\release\bundle\" -ForegroundColor White
