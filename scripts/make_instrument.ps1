# make_instrument.ps1
# Converts a folder of WAV files into an .aether-instrument JSON file.
#
# USAGE:
#   .\scripts\make_instrument.ps1 -Name "Krar" -WavDir "D:\samples\krar" -Output "ui\public\instruments\krar.aether-instrument"
#
# WAV FILE NAMING CONVENTION:
#   Files should be named with the MIDI note number or note name:
#     krar_C3.wav   or   krar_48.wav
#     krar_E3.wav   or   krar_52.wav
#   The script will auto-detect the root note from the filename.
#
# PARAMETERS:
#   -Name       Instrument display name (e.g. "Krar")
#   -WavDir     Directory containing WAV files
#   -Output     Output .aether-instrument file path
#   -Family     Instrument family (default: "plucked-string")
#   -Region     Region (default: "East Africa")
#   -Country    Country (default: "Ethiopia")
#   -Tuning     Tuning system (default: "ethiopian-tizita")
#   -NoteRange  How many semitones each sample covers (default: 3)

param(
    [Parameter(Mandatory=$true)]
    [string]$Name,

    [Parameter(Mandatory=$true)]
    [string]$WavDir,

    [Parameter(Mandatory=$true)]
    [string]$Output,

    [string]$Family = "plucked-string",
    [string]$Region = "East Africa",
    [string]$Country = "Ethiopia",
    [string]$Tuning = "ethiopian-tizita",
    [int]$NoteRange = 3,
    [float]$Attack = 0.02,
    [float]$Decay = 0.15,
    [float]$Sustain = 0.6,
    [float]$Release = 0.4
)

$ErrorActionPreference = "Stop"

# Note name → MIDI number mapping
$NoteMap = @{
    "C"  = 0;  "C#" = 1;  "Db" = 1;
    "D"  = 2;  "D#" = 3;  "Eb" = 3;
    "E"  = 4;
    "F"  = 5;  "F#" = 6;  "Gb" = 6;
    "G"  = 7;  "G#" = 8;  "Ab" = 8;
    "A"  = 9;  "A#" = 10; "Bb" = 10;
    "B"  = 11
}

function Get-MidiNote($filename) {
    $base = [System.IO.Path]::GetFileNameWithoutExtension($filename)

    # Try direct MIDI number: "48", "krar_48", "note_60"
    if ($base -match '(\d{2,3})$') {
        $num = [int]$Matches[1]
        if ($num -ge 0 -and $num -le 127) { return $num }
    }

    # Try note name + octave: "C3", "krar_C3", "E4", "F#3"
    if ($base -match '([A-Ga-g][#b]?)(\d)') {
        $noteName = $Matches[1].ToUpper()
        $octave   = [int]$Matches[2]
        if ($NoteMap.ContainsKey($noteName)) {
            return ($octave + 1) * 12 + $NoteMap[$noteName]
        }
    }

    return $null
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Aether Instrument Builder" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Find WAV files
$wavFiles = Get-ChildItem -Path $WavDir -Filter "*.wav" | Sort-Object Name
if ($wavFiles.Count -eq 0) {
    Write-Host "ERROR: No WAV files found in $WavDir" -ForegroundColor Red
    exit 1
}

Write-Host "Found $($wavFiles.Count) WAV files:" -ForegroundColor Yellow
$zones = @()

foreach ($file in $wavFiles) {
    $midiNote = Get-MidiNote $file.Name

    if ($null -eq $midiNote) {
        Write-Host "  ⚠ Could not detect MIDI note from: $($file.Name) — skipping" -ForegroundColor Yellow
        continue
    }

    $lowNote  = [Math]::Max(0,   $midiNote - [Math]::Floor($NoteRange / 2))
    $highNote = [Math]::Min(127, $midiNote + [Math]::Floor($NoteRange / 2))

    # Use relative path from the output file's directory
    $outputDir = [System.IO.Path]::GetDirectoryName([System.IO.Path]::GetFullPath($Output))
    $relPath   = [System.IO.Path]::GetRelativePath($outputDir, $file.FullName) -replace '\\', '/'

    $zone = [ordered]@{
        id            = "$($Name.ToLower())-$midiNote"
        file_path     = $relPath
        root_note     = $midiNote
        note_low      = $lowNote
        note_high     = $highNote
        velocity_low  = 0
        velocity_high = 127
        articulation  = "SustainLoop"
        volume_db     = 0.0
        tune_cents    = 0.0
        loop_start    = $null
        loop_end      = $null
        release_file  = $null
    }

    $zones += $zone
    Write-Host "  ✓ $($file.Name) → MIDI $midiNote (range $lowNote–$highNote)" -ForegroundColor Green
}

if ($zones.Count -eq 0) {
    Write-Host "ERROR: No zones created — check file naming convention" -ForegroundColor Red
    exit 1
}

# Build the instrument JSON
$instrument = [ordered]@{
    name        = $Name
    family      = $Family
    region      = $Region
    country     = $Country
    tuning      = @{
        name        = $Tuning
        frequencies = $null  # null = use default 12-TET; replace with custom array for microtonal
    }
    adsr        = @{
        attack  = $Attack
        decay   = $Decay
        sustain = $Sustain
        release = $Release
    }
    zones       = $zones
}

# Write output
$outputDir = [System.IO.Path]::GetDirectoryName([System.IO.Path]::GetFullPath($Output))
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
}

$json = $instrument | ConvertTo-Json -Depth 10
Set-Content -Path $Output -Value $json -Encoding UTF8

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Done!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Instrument: $Name" -ForegroundColor White
Write-Host "  Zones:      $($zones.Count)" -ForegroundColor White
Write-Host "  Output:     $Output" -ForegroundColor White
Write-Host ""
Write-Host "To use in Aether Studio:" -ForegroundColor Cyan
Write-Host "  1. Start aether-host" -ForegroundColor White
Write-Host "  2. Open Aether Studio in browser" -ForegroundColor White
Write-Host "  3. Go to Create mode → Add SamplerNode" -ForegroundColor White
Write-Host "  4. Open Instrument Rack → Load Instrument → select $Output" -ForegroundColor White
