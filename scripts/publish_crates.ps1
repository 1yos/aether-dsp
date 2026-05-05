#!/usr/bin/env pwsh
# Publish updated AetherDSP crates to crates.io
# Usage: .\scripts\publish_crates.ps1
#
# Prerequisites:
# - cargo login (run once to store your crates.io API token)
# - All changes committed to git
# - Versions bumped in Cargo.toml files

param(
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
Set-Location "D:\Audio kernel\aether-dsp"

# Ensure MinGW64 GCC is in PATH
$env:PATH = "C:\msys64\mingw64\bin;" + $env:PATH

Write-Host "=== Publishing AetherDSP crates to crates.io ===" -ForegroundColor Cyan

# Publish order matters — dependencies must be published first
$crates = @(
    "aetherdsp-ndk-macro",   # No deps
    "aetherdsp-manifest",    # Depends on core, ndk
    "aetherdsp-registry",    # Depends on ndk
    "aetherdsp-nodes"        # Depends on core (updated description)
)

foreach ($crate in $crates) {
    Write-Host "`n--- Publishing $crate ---" -ForegroundColor Yellow
    
    if ($DryRun) {
        cargo publish --dry-run -p $crate
    } else {
        cargo publish -p $crate
        if ($LASTEXITCODE -ne 0) {
            Write-Host "ERROR: Failed to publish $crate" -ForegroundColor Red
            exit 1
        }
        Write-Host "  $crate published successfully" -ForegroundColor Green
        # Wait 10 seconds for crates.io to index before publishing next crate
        Start-Sleep -Seconds 10
    }
}

Write-Host "`n=== All crates published ===" -ForegroundColor Cyan
Write-Host "Check: https://crates.io/users/AetherDSP" -ForegroundColor Gray
