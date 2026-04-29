#!/usr/bin/env pwsh
# FORCE RESTART - Kill all node processes and start fresh

Write-Host "=" -NoNewline -ForegroundColor Cyan
Write-Host "=".PadRight(70, '=') -ForegroundColor Cyan
Write-Host "  FORCE RESTART DEV SERVER" -ForegroundColor Yellow
Write-Host "=".PadRight(71, '=') -ForegroundColor Cyan
Write-Host ""

# Kill any existing node/vite processes
Write-Host "🛑 Killing existing Node processes..." -ForegroundColor Red
Get-Process node -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# Clear all caches
Write-Host "🧹 Clearing caches..." -ForegroundColor Cyan
Remove-Item -Recurse -Force node_modules\.vite -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force dist -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force .vite -ErrorAction SilentlyContinue

Write-Host "✅ Caches cleared" -ForegroundColor Green
Write-Host ""

# Verify catalog files exist
Write-Host "📁 Checking catalog files..." -ForegroundColor Cyan
$catalogFiles = @(
    "src\catalog\types.ts",
    "src\catalog\catalogData.ts",
    "src\catalog\InstrumentBrowser.tsx",
    "src\catalog\useCatalog.ts"
)

$allExist = $true
foreach ($file in $catalogFiles) {
    if (Test-Path $file) {
        Write-Host "  ✓ $file" -ForegroundColor Green
    } else {
        Write-Host "  ✗ MISSING: $file" -ForegroundColor Red
        $allExist = $false
    }
}

if (-not $allExist) {
    Write-Host ""
    Write-Host "❌ ERROR: Some catalog files are missing!" -ForegroundColor Red
    Write-Host "   The catalog may not work properly." -ForegroundColor Yellow
    Write-Host ""
    Read-Host "Press Enter to continue anyway"
}

Write-Host ""
Write-Host "🚀 Starting dev server..." -ForegroundColor Cyan
Write-Host ""
Write-Host "   After the server starts:" -ForegroundColor Yellow
Write-Host "   1. Open http://localhost:5173 in your browser" -ForegroundColor White
Write-Host "   2. Press Ctrl+Shift+R to hard refresh" -ForegroundColor White
Write-Host "   3. Look for 'v2.1-catalog' next to 'AetherStudio' in top-left" -ForegroundColor White
Write-Host "   4. The Catalog button (🌍 Catalog 60) should be visible" -ForegroundColor White
Write-Host "      after 'Instrument Maker' in the top bar" -ForegroundColor White
Write-Host ""
Write-Host "=" -NoNewline -ForegroundColor Cyan
Write-Host "=".PadRight(70, '=') -ForegroundColor Cyan
Write-Host ""

npm run dev
