#!/usr/bin/env pwsh
# AetherDSP v0.3 Release Build Script
# Run from the workspace root: .\scripts\build_release.ps1

param(
    [string]$Version = "0.3.0",
    [string]$OutDir = ".\release"
)

$ErrorActionPreference = "Stop"

Write-Host "=== AetherDSP v$Version Release Build ===" -ForegroundColor Cyan

# 1. Check toolchain
Write-Host "`n[1/6] Verifying toolchain..." -ForegroundColor Yellow
rustc --version
cargo --version
Write-Host "  Toolchain: OK" -ForegroundColor Green

# 2. Format check
Write-Host "`n[2/6] Checking formatting..." -ForegroundColor Yellow
cargo fmt --all -- --check
Write-Host "  Format: OK" -ForegroundColor Green

# 3. Clippy
Write-Host "`n[3/6] Running clippy..." -ForegroundColor Yellow
cargo clippy --workspace -- -D warnings -A dead_code
Write-Host "  Clippy: OK" -ForegroundColor Green

# 4. Tests
Write-Host "`n[4/6] Running tests..." -ForegroundColor Yellow
cargo test --lib -p aetherdsp-core -p aetherdsp-nodes -p aetherdsp-ndk -p aetherdsp-midi
Write-Host "  Tests: OK" -ForegroundColor Green

# 5. Release build
Write-Host "`n[5/6] Building release binaries..." -ForegroundColor Yellow
cargo build --release -p aether-ui
cargo build --release -p aether-host
cargo build --release -p aether-cli
Write-Host "  Build: OK" -ForegroundColor Green

# 6. Package
Write-Host "`n[6/6] Packaging..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Copy-Item "target\release\aether-studio.exe" "$OutDir\" -ErrorAction SilentlyContinue
Copy-Item "target\release\aether-host.exe" "$OutDir\" -ErrorAction SilentlyContinue
Copy-Item "target\release\aether-cli.exe" "$OutDir\" -ErrorAction SilentlyContinue
Copy-Item "README.md" "$OutDir\" -ErrorAction SilentlyContinue
Copy-Item "LICENSE" "$OutDir\" -ErrorAction SilentlyContinue

$zipName = "AetherDSP-v$Version-windows-x64.zip"
Compress-Archive -Path "$OutDir\*" -DestinationPath $zipName -Force
Write-Host "  Package: $zipName" -ForegroundColor Green

Write-Host "`n=== Release v$Version complete ===" -ForegroundColor Cyan
Write-Host "Artifacts:" -ForegroundColor White
Write-Host "  $zipName" -ForegroundColor Gray
Write-Host "  $OutDir\aether-studio.exe (Native DAW)" -ForegroundColor Gray
Write-Host "  $OutDir\aether-host.exe (Audio engine)" -ForegroundColor Gray
Write-Host "  $OutDir\aether-cli.exe (CLI tools)" -ForegroundColor Gray
