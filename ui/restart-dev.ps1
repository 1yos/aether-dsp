#!/usr/bin/env pwsh
# Force-restart dev server with clean cache

Write-Host "🧹 Clearing Vite cache..." -ForegroundColor Cyan
Remove-Item -Recurse -Force node_modules\.vite -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force dist -ErrorAction SilentlyContinue

Write-Host "✅ Cache cleared" -ForegroundColor Green
Write-Host ""
Write-Host "🚀 Starting dev server..." -ForegroundColor Cyan
Write-Host "   The Catalog button (🌍) should appear in the top bar" -ForegroundColor Yellow
Write-Host "   after 'Instrument Maker'" -ForegroundColor Yellow
Write-Host ""

npm run dev
