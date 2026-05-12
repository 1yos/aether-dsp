#!/usr/bin/env pwsh
# Publish Phase 1 improvements (CHANGELOG + Examples) to crates.io
# Usage: .\scripts\publish_phase1.ps1 [-DryRun]
#
# Prerequisites:
# - cargo login (run once to store your crates.io API token)
# - All changes committed to git
#
# This script publishes crates in dependency order with proper delays

param(
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
Set-Location "D:\Audio kernel\aether-dsp"

# Ensure MinGW64 GCC is in PATH (for Windows builds)
if (Test-Path "C:\msys64\mingw64\bin") {
    $env:PATH = "C:\msys64\mingw64\bin;" + $env:PATH
}

Write-Host "=== Publishing AetherDSP Phase 1 Improvements to crates.io ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Phase 1 includes:" -ForegroundColor Yellow
Write-Host "  ✅ CHANGELOG.md files (9 crates)" -ForegroundColor Green
Write-Host "  ✅ Working examples (6 new examples)" -ForegroundColor Green
Write-Host ""

if ($DryRun) {
    Write-Host "DRY RUN MODE - No actual publishing" -ForegroundColor Magenta
    Write-Host ""
}

# Publish order matters — dependencies must be published first
# Only publishing crates that have CHANGELOG.md and/or examples
$crates = @(
    @{Name="aetherdsp-ndk-macro"; HasExamples=$false; Reason="Macro crate (no deps)"},
    @{Name="aetherdsp-core"; HasExamples=$true; Reason="Core engine (3 new examples)"},
    @{Name="aetherdsp-manifest"; HasExamples=$false; Reason="Manifest format"},
    @{Name="aetherdsp-nodes"; HasExamples=$false; Reason="DSP nodes library"},
    @{Name="aetherdsp-ndk"; HasExamples=$true; Reason="Node Development Kit (1 new example)"},
    @{Name="aetherdsp-registry"; HasExamples=$false; Reason="Node registry"},
    @{Name="aetherdsp-midi"; HasExamples=$true; Reason="MIDI engine (1 new example)"},
    @{Name="aetherdsp-sampler"; HasExamples=$false; Reason="Sampler engine"},
    @{Name="aetherdsp-timbre"; HasExamples=$false; Reason="Spectral analysis"}
)

$successCount = 0
$failCount = 0

foreach ($crate in $crates) {
    $crateName = $crate.Name
    $reason = $crate.Reason
    
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "Publishing: $crateName" -ForegroundColor Yellow
    Write-Host "Reason: $reason" -ForegroundColor Gray
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    
    try {
        if ($DryRun) {
            Write-Host "  Running dry-run check..." -ForegroundColor Magenta
            cargo publish --dry-run -p $crateName
            if ($LASTEXITCODE -ne 0) {
                throw "Dry-run failed for $crateName"
            }
            Write-Host "  ✅ Dry-run passed for $crateName" -ForegroundColor Green
        } else {
            Write-Host "  Publishing to crates.io..." -ForegroundColor Yellow
            cargo publish -p $crateName
            if ($LASTEXITCODE -ne 0) {
                throw "Publish failed for $crateName"
            }
            Write-Host "  ✅ $crateName published successfully" -ForegroundColor Green
            
            # Wait for crates.io to index before publishing next crate
            Write-Host "  ⏳ Waiting 15 seconds for crates.io indexing..." -ForegroundColor Gray
            Start-Sleep -Seconds 15
        }
        $successCount++
    }
    catch {
        Write-Host "  ❌ ERROR: $_" -ForegroundColor Red
        $failCount++
        
        # Ask user if they want to continue
        if (-not $DryRun) {
            $response = Read-Host "Continue with remaining crates? (y/n)"
            if ($response -ne "y") {
                Write-Host ""
                Write-Host "Publishing aborted by user" -ForegroundColor Red
                exit 1
            }
        }
    }
}

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "PUBLISHING COMPLETE" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""
Write-Host "Results:" -ForegroundColor Yellow
Write-Host "  ✅ Success: $successCount crates" -ForegroundColor Green
if ($failCount -gt 0) {
    Write-Host "  ❌ Failed: $failCount crates" -ForegroundColor Red
}
Write-Host ""

if (-not $DryRun -and $successCount -gt 0) {
    Write-Host "Next Steps:" -ForegroundColor Yellow
    Write-Host "  1. Verify on crates.io: https://crates.io/users/AetherDSP" -ForegroundColor Gray
    Write-Host "  2. Check docs.rs: https://docs.rs/aetherdsp-core" -ForegroundColor Gray
    Write-Host "  3. Announce on Reddit r/rust" -ForegroundColor Gray
    Write-Host "  4. Post on Rust Users Forum" -ForegroundColor Gray
    Write-Host "  5. Monitor downloads weekly" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Expected Impact (2 weeks):" -ForegroundColor Yellow
    Write-Host "  📈 2-3× increase in downloads" -ForegroundColor Green
    Write-Host "  ⭐ +20-30 GitHub stars" -ForegroundColor Green
    Write-Host "  📚 Better docs.rs ranking" -ForegroundColor Green
}

if ($DryRun) {
    Write-Host ""
    Write-Host "Dry-run complete! Run without -DryRun to publish for real." -ForegroundColor Magenta
}
