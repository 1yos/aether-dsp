# Build script for Aether UI with MSVC toolchain
# This script ensures the correct environment is set up before building

Write-Host "=== Aether UI Build Script ===" -ForegroundColor Cyan
Write-Host ""

# 1. Check toolchain
Write-Host "Checking Rust toolchain..." -ForegroundColor Yellow
$toolchain = rustup show active-toolchain
if ($toolchain -notmatch "msvc") {
    Write-Host "❌ Wrong toolchain: $toolchain" -ForegroundColor Red
    Write-Host "Switching to MSVC..." -ForegroundColor Yellow
    rustup default stable-x86_64-pc-windows-msvc
    Write-Host "✅ Switched to MSVC toolchain" -ForegroundColor Green
} else {
    Write-Host "✅ MSVC toolchain active" -ForegroundColor Green
}
Write-Host ""

# 2. Check for MSVC compiler
Write-Host "Checking for MSVC compiler..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($vsPath) {
        Write-Host "✅ Visual Studio Build Tools found at: $vsPath" -ForegroundColor Green
    } else {
        Write-Host "❌ Visual Studio Build Tools not found" -ForegroundColor Red
        Write-Host "Please install Visual Studio Build Tools with C++ workload" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "⚠️  Cannot verify VS installation (vswhere.exe not found)" -ForegroundColor Yellow
    Write-Host "Attempting build anyway..." -ForegroundColor Yellow
}
Write-Host ""

# 3. Clean build (optional - uncomment if needed)
# Write-Host "Cleaning previous build artifacts..." -ForegroundColor Yellow
# cargo clean
# Write-Host "✅ Clean complete" -ForegroundColor Green
# Write-Host ""

# 4. Build
Write-Host "Building aether-ui..." -ForegroundColor Yellow
Write-Host "This will take 5-10 minutes on first build..." -ForegroundColor Cyan
Write-Host ""

$buildStart = Get-Date
cargo build --release -p aether-ui

if ($LASTEXITCODE -eq 0) {
    $buildTime = (Get-Date) - $buildStart
    Write-Host ""
    Write-Host "✅ Build successful!" -ForegroundColor Green
    Write-Host "Build time: $($buildTime.ToString('mm\:ss'))" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Binary location:" -ForegroundColor Yellow
    Write-Host "  C:\aether-target\release\aether-studio.exe" -ForegroundColor White
    Write-Host ""
    Write-Host "To run:" -ForegroundColor Yellow
    Write-Host "  C:\aether-target\release\aether-studio.exe" -ForegroundColor White
    Write-Host "  or" -ForegroundColor Gray
    Write-Host "  cargo run --release -p aether-ui" -ForegroundColor White
} else {
    Write-Host ""
    Write-Host "❌ Build failed" -ForegroundColor Red
    Write-Host ""
    Write-Host "Common fixes:" -ForegroundColor Yellow
    Write-Host "  1. Ensure Visual Studio Build Tools are installed" -ForegroundColor White
    Write-Host "  2. Run 'cargo clean' and try again" -ForegroundColor White
    Write-Host "  3. Restart your terminal/IDE" -ForegroundColor White
    Write-Host "  4. Check that MSVC toolchain is active: rustup show" -ForegroundColor White
    exit 1
}
