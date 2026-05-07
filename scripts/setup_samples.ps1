# Setup VSCO 2 CE Samples for Aether DSP
# This script helps organize VSCO 2 CE samples into the correct directory structure

param(
    [Parameter(Mandatory=$true)]
    [string]$VscoPath,  # Path to extracted VSCO 2 CE folder
    
    [Parameter(Mandatory=$false)]
    [string]$OutputPath = "assets\samples"
)

Write-Host "=== Aether DSP Sample Setup ===" -ForegroundColor Cyan
Write-Host ""

# Check if VSCO path exists
if (-not (Test-Path $VscoPath)) {
    Write-Host "ERROR: VSCO 2 CE path not found: $VscoPath" -ForegroundColor Red
    Write-Host "Please download VSCO 2 CE from: https://github.com/sgossner/VSCO-2-CE/releases" -ForegroundColor Yellow
    exit 1
}

# Create output directory
$OutputPath = Join-Path $PSScriptRoot "..\$OutputPath"
if (-not (Test-Path $OutputPath)) {
    New-Item -ItemType Directory -Path $OutputPath -Force | Out-Null
    Write-Host "Created samples directory: $OutputPath" -ForegroundColor Green
}

# Create drums-studio subdirectory structure
$drumsPath = Join-Path $OutputPath "drums-studio"
$subdirs = @("kick", "snare", "hihat", "toms", "cymbals")

foreach ($subdir in $subdirs) {
    $path = Join-Path $drumsPath $subdir
    if (-not (Test-Path $path)) {
        New-Item -ItemType Directory -Path $path -Force | Out-Null
        Write-Host "Created: $subdir\" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "=== Copying Drum Samples ===" -ForegroundColor Cyan

# Map VSCO 2 CE drum samples to our structure
# Note: Adjust these paths based on actual VSCO 2 CE structure
$sampleMap = @{
    # Kick drums
    "Percussion\Bass Drum\*.wav" = "drums-studio\kick"
    
    # Snare drums
    "Percussion\Snare\*.wav" = "drums-studio\snare"
    
    # Hi-hats
    "Percussion\Hi-Hat\*.wav" = "drums-studio\hihat"
    
    # Toms
    "Percussion\Tom\*.wav" = "drums-studio\toms"
    
    # Cymbals
    "Percussion\Crash\*.wav" = "drums-studio\cymbals"
    "Percussion\Ride\*.wav" = "drums-studio\cymbals"
}

$copiedCount = 0
$missingCount = 0

foreach ($pattern in $sampleMap.Keys) {
    $sourcePath = Join-Path $VscoPath $pattern
    $destPath = Join-Path $OutputPath $sampleMap[$pattern]
    
    $files = Get-ChildItem -Path $sourcePath -ErrorAction SilentlyContinue
    
    if ($files) {
        foreach ($file in $files) {
            $destFile = Join-Path $destPath $file.Name
            Copy-Item -Path $file.FullName -Destination $destFile -Force
            Write-Host "  Copied: $($file.Name)" -ForegroundColor Gray
            $copiedCount++
        }
    } else {
        Write-Host "  WARNING: No files found matching: $pattern" -ForegroundColor Yellow
        $missingCount++
    }
}

Write-Host ""
Write-Host "=== Summary ===" -ForegroundColor Cyan
Write-Host "Copied: $copiedCount files" -ForegroundColor Green
if ($missingCount -gt 0) {
    Write-Host "Missing: $missingCount patterns" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "NOTE: VSCO 2 CE structure may vary. You may need to manually copy samples." -ForegroundColor Yellow
    Write-Host "Expected structure:" -ForegroundColor Yellow
    Write-Host "  assets\samples\drums-studio\kick\kick-v1.wav" -ForegroundColor Gray
    Write-Host "  assets\samples\drums-studio\snare\snare-v1.wav" -ForegroundColor Gray
    Write-Host "  assets\samples\drums-studio\hihat\hihat-closed-v1.wav" -ForegroundColor Gray
    Write-Host "  etc..." -ForegroundColor Gray
}

Write-Host ""
Write-Host "Done! Samples are ready in: $OutputPath" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Verify samples are in correct locations" -ForegroundColor White
Write-Host "2. Rename files to match drums-studio.json (kick-v1.wav, snare-v1.wav, etc.)" -ForegroundColor White
Write-Host "3. Run: cargo build -p aether-ui" -ForegroundColor White
Write-Host "4. Test drum playback in the UI" -ForegroundColor White
