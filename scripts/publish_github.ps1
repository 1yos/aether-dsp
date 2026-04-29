#!/usr/bin/env pwsh
# Push AetherDSP to GitHub and create a release tag
# Usage: .\scripts\publish_github.ps1 -Repo "YOUR_USERNAME/aether-dsp"

param(
    [Parameter(Mandatory)][string]$Repo,
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"
Set-Location "D:\Audio kernel\aether-dsp"

Write-Host "=== Publishing AetherDSP v$Version to $Repo ===" -ForegroundColor Cyan

# Init if needed
if (-not (Test-Path ".git")) {
    git init
    Write-Host "  git init: OK" -ForegroundColor Green
}

# Stage and commit
git add .
git commit -m "AetherDSP v$Version

- Hard real-time DSP runtime in Rust
- Lock-free SPSC command ring
- Generational arena allocator
- Topological DAG execution
- React Flow visual node editor
- WebSocket bridge (tokio-tungstenite)
- Criterion benchmark suite (51.7 ns param fill)
- CLAP plugin template (NIH-plug)
- IEEE LaTeX paper draft
- Investor pitch deck" 2>&1

git branch -M main
git remote remove origin 2>$null
git remote add origin "https://github.com/$Repo.git"
git push -u origin main --force

# Tag
git tag -a "v$Version" -m "AetherDSP v$Version Initial Release"
git push origin "v$Version"

Write-Host "`n=== Published ===" -ForegroundColor Cyan
Write-Host "  https://github.com/$Repo" -ForegroundColor Gray
Write-Host "  https://github.com/$Repo/releases/tag/v$Version" -ForegroundColor Gray
Write-Host "`nNext: create a GitHub Release and attach:" -ForegroundColor Yellow
Write-Host "  - AetherDSP-v$Version-windows-x64.zip" -ForegroundColor Gray
Write-Host "  - docs/architecture.svg" -ForegroundColor Gray
Write-Host "  - docs/paper_draft.md" -ForegroundColor Gray
