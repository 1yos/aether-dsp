# Download VSCO 2 CE Samples
# Automatically downloads and extracts VSCO 2 Community Edition

param(
    [Parameter(Mandatory=$false)]
    [string]$TempPath = "$env:TEMP\vsco2ce",
    
    [Parameter(Mandatory=$false)]
    [switch]$SkipDownload
)

$ErrorActionPreference = "Stop"

Write-Host "=== VSCO 2 CE Downloader ===" -ForegroundColor Cyan
Write-Host ""

# VSCO 2 CE download URL (check GitHub releases for latest)
$vscoUrl = "https://github.com/sgossner/VSCO-2-CE/archive/refs/heads/master.zip"
$zipFile = Join-Path $TempPath "vsco2ce.zip"
$extractPath = Join-Path $TempPath "extracted"

# Create temp directory
if (-not (Test-Path $TempPath)) {
    New-Item -ItemType Directory -Path $TempPath -Force | Out-Null
}

if (-not $SkipDownload) {
    Write-Host "Downloading VSCO 2 CE from GitHub..." -ForegroundColor Yellow
    Write-Host "URL: $vscoUrl" -ForegroundColor Gray
    Write-Host "This may take several minutes (~1GB download)..." -ForegroundColor Gray
    Write-Host ""
    
    try {
        # Download with progress
        $ProgressPreference = 'SilentlyContinue'
        Invoke-WebRequest -Uri $vscoUrl -OutFile $zipFile -UseBasicParsing
        $ProgressPreference = 'Continue'
        
        $fileSize = (Get-Item $zipFile).Length / 1MB
        Write-Host "Downloaded: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Green
    } catch {
        Write-Host "ERROR: Failed to download VSCO 2 CE" -ForegroundColor Red
        Write-Host $_.Exception.Message -ForegroundColor Red
        Write-Host ""
        Write-Host "Please download manually from:" -ForegroundColor Yellow
        Write-Host "  https://github.com/sgossner/VSCO-2-CE/releases" -ForegroundColor Gray
        exit 1
    }
    
    Write-Host ""
    Write-Host "Extracting archive..." -ForegroundColor Yellow
    
    try {
        Expand-Archive -Path $zipFile -DestinationPath $extractPath -Force
        Write-Host "Extracted successfully" -ForegroundColor Green
    } catch {
        Write-Host "ERROR: Failed to extract archive" -ForegroundColor Red
        Write-Host $_.Exception.Message -ForegroundColor Red
        exit 1
    }
}

Write-Host ""
Write-Host "VSCO 2 CE is ready at: $extractPath" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Run the setup script:" -ForegroundColor White
Write-Host "   .\scripts\setup_samples.ps1 -VscoPath '$extractPath\VSCO-2-CE-master'" -ForegroundColor Gray
Write-Host ""
Write-Host "2. Or manually copy samples to:" -ForegroundColor White
Write-Host "   assets\samples\drums-studio\" -ForegroundColor Gray
Write-Host ""

# Try to find the actual VSCO folder
$vscoFolder = Get-ChildItem -Path $extractPath -Directory -Filter "VSCO*" -ErrorAction SilentlyContinue | Select-Object -First 1

if ($vscoFolder) {
    Write-Host "Found VSCO folder: $($vscoFolder.FullName)" -ForegroundColor Green
    Write-Host ""
    Write-Host "Quick setup command:" -ForegroundColor Cyan
    Write-Host "  .\scripts\setup_samples.ps1 -VscoPath '$($vscoFolder.FullName)'" -ForegroundColor Gray
} else {
    Write-Host "Could not locate VSCO folder automatically." -ForegroundColor Yellow
    Write-Host "Please check: $extractPath" -ForegroundColor Gray
}

Write-Host ""
