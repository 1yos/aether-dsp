#!/usr/bin/env pwsh
# AetherDSP v0.1 Release Build Script
# Run from the workspace root: .\scripts\build_release.ps1

param(
    [string]$Version = "0.1.0",
    [string]$OutDir = ".\release"
)

$ErrorActionPreference = "Stop"
$env:PATH = "C:\msys64\mingw64\bin;" + $env:PATH

Write-Host "=== AetherDSP v$Version Release Build ===" -ForegroundColor Cyan

# 1. Check toolchain
Write-Host "`n[1/7] Verifying toolchain..." -ForegroundColor Yellow
rustc --version
cargo --version

# 2. Format check
Write-Host "`n[2/7] Checking formatting..." -ForegroundColor Yellow
Set-Location "C:\aether-dsp"
cargo fmt --all -- --check
Write-Host "  Format: OK" -ForegroundColor Green

# 3. Clippy
Write-Host "`n[3/7] Running clippy..." -ForegroundColor Yellow
cargo clippy --workspace -- -D warnings
Write-Host "  Clippy: OK" -ForegroundColor Green

# 4. Tests
Write-Host "`n[4/7] Running tests..." -ForegroundColor Yellow
cargo test --workspace 2>&1 | Select-String -Pattern "test result"
Write-Host "  Tests: OK" -ForegroundColor Green

# 5. Release build
Write-Host "`n[5/7] Building release..." -ForegroundColor Yellow
cargo build --release
Write-Host "  Build: OK" -ForegroundColor Green

# 6. UI build
Write-Host "`n[6/7] Building UI..." -ForegroundColor Yellow
Set-Location "D:\Audio kernel\aether-dsp\ui"
npm run build
Write-Host "  UI: OK" -ForegroundColor Green

# 7. Tauri desktop app
Write-Host "`n[7/8] Building Tauri desktop app..." -ForegroundColor Yellow
Set-Location "$PSScriptRoot/../ui"
npm run tauri build -- --target x86_64-pc-windows-gnu
Write-Host "  Tauri: OK" -ForegroundColor Green

# 8. Package
Write-Host "`n[8/8] Packaging..." -ForegroundColor Yellow
Set-Location "C:\"
$zipName = "AetherDSP-v$Version-windows-x64.zip"
Compress-Archive -Path "aether-dsp" -DestinationPath $zipName -Force
Write-Host "  Package: $zipName" -ForegroundColor Green

Write-Host "`n=== Release v$Version complete ===" -ForegroundColor Cyan
Write-Host "Artifacts:" -ForegroundColor White
Write-Host "  C:\$zipName" -ForegroundColor Gray
Write-Host "  C:\aether-dsp\target\release\aether-host.exe" -ForegroundColor Gray
Write-Host "  D:\Audio kernel\aether-dsp\ui\dist\" -ForegroundColor Gray
