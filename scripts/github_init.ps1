# github_init.ps1
# One-time script to initialize git and push to GitHub.
#
# USAGE:
#   1. Create an empty repo on GitHub named "aether-dsp" (no README, no .gitignore)
#   2. Edit the $RepoUrl below to match your GitHub username
#   3. Run: .\scripts\github_init.ps1

param(
    [string]$RepoUrl = "https://github.com/YOUR_USERNAME/aether-dsp.git"
)

$ErrorActionPreference = "Stop"

if ($RepoUrl -like "*YOUR_USERNAME*") {
    Write-Host "ERROR: Edit the RepoUrl in this script first." -ForegroundColor Red
    Write-Host "  Replace YOUR_USERNAME with your actual GitHub username." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  AetherDSP — GitHub init" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Initialize git if not already done
if (-not (Test-Path ".git")) {
    Write-Host "→ Initializing git repository..." -ForegroundColor Yellow
    git init
    Write-Host "  ✓ git init" -ForegroundColor Green
} else {
    Write-Host "  ✓ git already initialized" -ForegroundColor Green
}

# Stage everything
Write-Host "→ Staging files..." -ForegroundColor Yellow
git add .
Write-Host "  ✓ git add ." -ForegroundColor Green

# Initial commit
Write-Host "→ Creating initial commit..." -ForegroundColor Yellow
git commit -m "feat: initial release of AetherDSP v0.1.0

- aether-core: hard RT DSP engine (arena, graph, scheduler, buffer pool)
- aether-nodes: built-in DSP nodes (oscillator, SVF filter, ADSR, delay, gain, mixer)
- aether-ndk: node development kit with #[aether_node] proc macro
- aether-ndk-macro: procedural macro crate
- aether-midi: MIDI engine with microtonal tuning support
- aether-manifest: node package manifest format
- aether-registry: runtime node type registry
- aether-sampler: polyphonic sampler engine
- aether-timbre: spectral timbre analysis and transfer
- aether-host: CPAL audio host with WebSocket bridge
- ui: React + ReactFlow studio interface"
Write-Host "  ✓ initial commit" -ForegroundColor Green

# Set main branch
Write-Host "→ Setting branch to main..." -ForegroundColor Yellow
git branch -M main
Write-Host "  ✓ branch: main" -ForegroundColor Green

# Add remote
Write-Host "→ Adding remote origin..." -ForegroundColor Yellow
git remote add origin $RepoUrl
Write-Host "  ✓ remote: $RepoUrl" -ForegroundColor Green

# Push
Write-Host "→ Pushing to GitHub..." -ForegroundColor Yellow
git push -u origin main
Write-Host "  ✓ pushed to GitHub" -ForegroundColor Green

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Done! Your repo is live at:" -ForegroundColor Green
Write-Host "  $($RepoUrl -replace '\.git$', '')" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Update repository URL in workspace Cargo.toml" -ForegroundColor White
Write-Host "     repository = `"$($RepoUrl -replace '\.git$', '')`"" -ForegroundColor Gray
Write-Host "  2. Run dry-run publish check:" -ForegroundColor White
Write-Host "     .\scripts\publish_crates.ps1" -ForegroundColor Gray
Write-Host "  3. When ready, publish for real:" -ForegroundColor White
Write-Host "     .\scripts\publish_crates.ps1 -Live" -ForegroundColor Gray
