# publish_crates.ps1
# Publishes all publishable AetherDSP crates to crates.io in dependency order.
#
# USAGE:
#   .\scripts\publish_crates.ps1           # dry run (safe, no upload)
#   .\scripts\publish_crates.ps1 -Live     # real publish
#
# BEFORE RUNNING:
#   1. cargo login   (paste your crates.io API token)
#   2. Make sure you are on the main branch with a clean git status
#   3. Update the version in Cargo.toml [workspace.package] if needed

param(
    [switch]$Live   # Pass -Live to actually publish; default is dry-run
)

$ErrorActionPreference = "Stop"
$env:PATH = "C:\msys64\mingw64\bin;" + $env:PATH

$DryRun = if ($Live) { "" } else { "--dry-run" }
$Mode   = if ($Live) { "LIVE" } else { "DRY RUN" }

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  AetherDSP crates.io publish — $Mode" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Publish order — must respect dependency chain
$Crates = @(
    "crates/aether-core",        # no internal deps
    "crates/aether-ndk-macro",   # no internal deps (proc-macro)
    "crates/aether-nodes",       # depends on aether-core
    "crates/aether-ndk",         # depends on core + nodes + macro
    "crates/aether-midi",        # depends on aether-core
    "crates/aether-manifest",    # depends on core + ndk
    "crates/aether-registry",    # depends on ndk
    "crates/aether-sampler",     # depends on core + midi
    "crates/aether-timbre"       # depends on core + sampler
)

$Published = @()
$Failed    = @()

foreach ($Crate in $Crates) {
    $Name = Split-Path $Crate -Leaf
    Write-Host "→ Publishing $Name ..." -ForegroundColor Yellow

    Push-Location $Crate
    try {
        if ($Live) {
            # Real publish — wait between crates so crates.io index updates
            cargo publish
            Write-Host "  ✓ $Name published" -ForegroundColor Green
            $Published += $Name
            Write-Host "  Waiting 20s for crates.io index to update..." -ForegroundColor Gray
            Start-Sleep -Seconds 20
        } else {
            cargo publish --dry-run --allow-dirty
            Write-Host "  ✓ $Name dry-run OK" -ForegroundColor Green
            $Published += $Name
        }
    } catch {
        Write-Host "  ✗ $Name FAILED: $_" -ForegroundColor Red
        $Failed += $Name
        if ($Live) {
            Write-Host "  Stopping — fix the error above before continuing." -ForegroundColor Red
            Pop-Location
            exit 1
        }
    }
    Pop-Location
    Write-Host ""
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Summary ($Mode)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  OK:     $($Published -join ', ')" -ForegroundColor Green
if ($Failed.Count -gt 0) {
    Write-Host "  FAILED: $($Failed -join ', ')" -ForegroundColor Red
}
Write-Host ""

if (-not $Live) {
    Write-Host "Dry run complete. Run with -Live to publish for real:" -ForegroundColor Cyan
    Write-Host "  .\scripts\publish_crates.ps1 -Live" -ForegroundColor White
}
