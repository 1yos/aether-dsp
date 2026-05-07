# Download Free CC0 Drum Samples
# Uses samples from freesound.org and other CC0 sources

param(
    [Parameter(Mandatory=$false)]
    [string]$OutputPath = "assets\samples\drums-studio"
)

Write-Host "=== Downloading Free Drum Samples (CC0) ===" -ForegroundColor Cyan
Write-Host ""

# Create directory structure
$basePath = Join-Path $PSScriptRoot "..\$OutputPath"
$subdirs = @("kick", "snare", "hihat", "toms", "cymbals")

foreach ($subdir in $subdirs) {
    $path = Join-Path $basePath $subdir
    if (-not (Test-Path $path)) {
        New-Item -ItemType Directory -Path $path -Force | Out-Null
        Write-Host "Created: $subdir\" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "NOTE: This script requires manual download due to freesound.org API requirements." -ForegroundColor Yellow
Write-Host ""
Write-Host "Please download samples manually from these CC0 sources:" -ForegroundColor Cyan
Write-Host ""

# Provide direct links to CC0 drum samples
$samples = @(
    @{
        Name = "Kick Drum"
        URL = "https://freesound.org/people/DWSD/sounds/171104/"
        File = "kick-v1.wav"
        Dest = "kick"
    },
    @{
        Name = "Snare Drum"
        URL = "https://freesound.org/people/DWSD/sounds/171106/"
        File = "snare-v1.wav"
        Dest = "snare"
    },
    @{
        Name = "Snare Rim"
        URL = "https://freesound.org/people/DWSD/sounds/171107/"
        File = "snare-rim-v1.wav"
        Dest = "snare"
    },
    @{
        Name = "Hi-Hat Closed"
        URL = "https://freesound.org/people/DWSD/sounds/171099/"
        File = "hihat-closed-v1.wav"
        Dest = "hihat"
    },
    @{
        Name = "Hi-Hat Open"
        URL = "https://freesound.org/people/DWSD/sounds/171100/"
        File = "hihat-open-v1.wav"
        Dest = "hihat"
    },
    @{
        Name = "Hi-Hat Pedal"
        URL = "https://freesound.org/people/DWSD/sounds/171101/"
        File = "hihat-pedal-v1.wav"
        Dest = "hihat"
    },
    @{
        Name = "Tom High"
        URL = "https://freesound.org/people/DWSD/sounds/171108/"
        File = "tom-high-v1.wav"
        Dest = "toms"
    },
    @{
        Name = "Tom Mid"
        URL = "https://freesound.org/people/DWSD/sounds/171109/"
        File = "tom-mid-v1.wav"
        Dest = "toms"
    },
    @{
        Name = "Tom Floor"
        URL = "https://freesound.org/people/DWSD/sounds/171110/"
        File = "tom-floor-v1.wav"
        Dest = "toms"
    },
    @{
        Name = "Crash Cymbal"
        URL = "https://freesound.org/people/DWSD/sounds/171097/"
        File = "crash-v1.wav"
        Dest = "cymbals"
    },
    @{
        Name = "Ride Cymbal"
        URL = "https://freesound.org/people/DWSD/sounds/171105/"
        File = "ride-v1.wav"
        Dest = "cymbals"
    },
    @{
        Name = "Ride Bell"
        URL = "https://freesound.org/people/DWSD/sounds/171105/"
        File = "ride-bell-v1.wav"
        Dest = "cymbals"
    }
)

Write-Host "Download Instructions:" -ForegroundColor Yellow
Write-Host "1. Visit each URL below" -ForegroundColor White
Write-Host "2. Click 'Download' button" -ForegroundColor White
Write-Host "3. Save to the specified location" -ForegroundColor White
Write-Host "4. Rename to the specified filename" -ForegroundColor White
Write-Host ""

foreach ($sample in $samples) {
    $destPath = Join-Path $basePath "$($sample.Dest)\$($sample.File)"
    Write-Host "$($sample.Name):" -ForegroundColor Cyan
    Write-Host "  URL:  $($sample.URL)" -ForegroundColor Gray
    Write-Host "  Save: $destPath" -ForegroundColor Gray
    Write-Host ""
}

Write-Host "=== Alternative: Use VSCO 2 CE ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "For better quality, download VSCO 2 CE (~1GB):" -ForegroundColor White
Write-Host "  https://github.com/sgossner/VSCO-2-CE/releases" -ForegroundColor Gray
Write-Host ""
Write-Host "Then run:" -ForegroundColor White
Write-Host "  .\scripts\setup_samples.ps1 -VscoPath 'C:\Path\To\VSCO2-CE'" -ForegroundColor Gray
Write-Host ""

Write-Host "After downloading samples, verify with:" -ForegroundColor Cyan
Write-Host "  Get-ChildItem -Recurse $basePath" -ForegroundColor Gray
