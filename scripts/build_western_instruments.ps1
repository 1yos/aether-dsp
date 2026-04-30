# build_western_instruments.ps1
# Builds .aether-instrument files from VSCO-2-CE WAV samples.
# Run AFTER the VSCO-2-CE clone completes.
#
# USAGE:
#   .\scripts\build_western_instruments.ps1
#
# Requires: D:\samples\VSCO-2-CE (cloned from https://github.com/sgossner/VSCO-2-CE)
# License: CC0 (public domain) — free for any use

$ErrorActionPreference = "Stop"
$VscoDir  = "D:\samples\VSCO-2-CE"
$OutDir   = "aether-dsp\ui\public\instruments"

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

# Note name to MIDI number
function Get-MidiNote($noteName) {
    $noteMap = @{
        "C"=0;"Cs"=1;"Db"=1;"D"=2;"Ds"=3;"Eb"=3;"E"=4;"F"=5;"Fs"=6;"Gb"=6;"G"=7;"Gs"=8;"Ab"=8;"A"=9;"As"=10;"Bb"=10;"B"=11
    }
    if ($noteName -match '^([A-Ga-g][sb]?)(\d)$') {
        $note = $Matches[1]
        # Normalize: s -> # equivalent
        $note = $note -replace 's$','s'
        $octave = [int]$Matches[2]
        if ($noteMap.ContainsKey($note)) {
            return ($octave + 1) * 12 + $noteMap[$note]
        }
    }
    return $null
}

# Build instrument JSON from a folder of WAV files
function Build-Instrument($name, $wavDir, $family, $region, $country, $tuning, $outFile) {
    $wavFiles = Get-ChildItem -Path $wavDir -Filter "*.wav" -ErrorAction SilentlyContinue |
                Sort-Object Name

    if ($wavFiles.Count -eq 0) {
        Write-Host "  ✗ No WAV files found in $wavDir" -ForegroundColor Red
        return
    }

    $zones = @()
    foreach ($file in $wavFiles) {
        # Try to extract MIDI note from filename
        # VSCO files: "Strings - Violin - Pizzicato - A3.wav" or "Violin_A3_v1.wav"
        $midiNote = $null
        if ($file.BaseName -match '([A-Ga-g][sb]?\d)') {
            $midiNote = Get-MidiNote $Matches[1]
        }
        if ($null -eq $midiNote) {
            # Try numeric
            if ($file.BaseName -match '(\d{2,3})$') {
                $midiNote = [int]$Matches[1]
            }
        }
        if ($null -eq $midiNote) { continue }

        $relPath = [System.IO.Path]::GetRelativePath(
            [System.IO.Path]::GetDirectoryName([System.IO.Path]::GetFullPath($outFile)),
            $file.FullName
        ) -replace '\\','/'

        $zones += [ordered]@{
            id            = "$($name.ToLower())-$midiNote"
            file_path     = $relPath
            root_note     = $midiNote
            note_low      = [Math]::Max(0, $midiNote - 2)
            note_high     = [Math]::Min(127, $midiNote + 2)
            velocity_low  = 0
            velocity_high = 127
            articulation  = "SustainLoop"
            volume_db     = 0.0
            tune_cents    = 0.0
            loop_start    = $null
            loop_end      = $null
            release_file  = $null
        }
    }

    if ($zones.Count -eq 0) {
        Write-Host "  ✗ Could not detect MIDI notes from filenames in $wavDir" -ForegroundColor Red
        return
    }

    $instrument = [ordered]@{
        name    = $name
        family  = $family
        region  = $region
        country = $country
        tuning  = @{ name = $tuning; frequencies = $null }
        adsr    = @{ attack = 0.01; decay = 0.1; sustain = 0.7; release = 0.3 }
        zones   = $zones
    }

    $instrument | ConvertTo-Json -Depth 10 | Set-Content -Path $outFile -Encoding UTF8
    Write-Host "  ✓ $name — $($zones.Count) zones → $outFile" -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Building Western Instruments (VSCO-2-CE)" -ForegroundColor Cyan
Write-Host "  License: CC0 — Public Domain" -ForegroundColor Gray
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path $VscoDir)) {
    Write-Host "ERROR: VSCO-2-CE not found at $VscoDir" -ForegroundColor Red
    Write-Host "Clone it first:" -ForegroundColor Yellow
    Write-Host "  git clone --depth=1 https://github.com/sgossner/VSCO-2-CE.git D:\samples\VSCO-2-CE" -ForegroundColor Gray
    exit 1
}

# Show what's available
Write-Host "Available VSCO folders:" -ForegroundColor Yellow
Get-ChildItem $VscoDir -Directory | ForEach-Object { Write-Host "  $($_.Name)" -ForegroundColor Gray }
Write-Host ""

# Map VSCO folders to instruments
$InstrumentMap = @(
    @{ Name="Violin";    Dir="$VscoDir\Strings\Violin - Sustain";    Family="bowed-string"; Region="Europe";  Country="Italy";   Tuning="12-tet" },
    @{ Name="Cello";     Dir="$VscoDir\Strings\Cello - Sustain";     Family="bowed-string"; Region="Europe";  Country="Italy";   Tuning="12-tet" },
    @{ Name="Viola";     Dir="$VscoDir\Strings\Viola - Sustain";     Family="bowed-string"; Region="Europe";  Country="Italy";   Tuning="12-tet" },
    @{ Name="Flute";     Dir="$VscoDir\Woodwinds\Flute";             Family="wind";         Region="Europe";  Country="Germany"; Tuning="12-tet" },
    @{ Name="Oboe";      Dir="$VscoDir\Woodwinds\Oboe";              Family="wind";         Region="Europe";  Country="Germany"; Tuning="12-tet" },
    @{ Name="Clarinet";  Dir="$VscoDir\Woodwinds\Clarinet";          Family="wind";         Region="Europe";  Country="Germany"; Tuning="12-tet" },
    @{ Name="Bassoon";   Dir="$VscoDir\Woodwinds\Bassoon";           Family="wind";         Region="Europe";  Country="Germany"; Tuning="12-tet" },
    @{ Name="Trumpet";   Dir="$VscoDir\Brass\Trumpet";               Family="wind";         Region="Europe";  Country="Germany"; Tuning="12-tet" },
    @{ Name="Trombone";  Dir="$VscoDir\Brass\Trombone";              Family="wind";         Region="Europe";  Country="Germany"; Tuning="12-tet" },
    @{ Name="French Horn";Dir="$VscoDir\Brass\French Horn";          Family="wind";         Region="Europe";  Country="Germany"; Tuning="12-tet" },
    @{ Name="Tuba";      Dir="$VscoDir\Brass\Tuba";                  Family="wind";         Region="Europe";  Country="Germany"; Tuning="12-tet" },
    @{ Name="Marimba";   Dir="$VscoDir\Percussion\Marimba";          Family="percussion";   Region="Americas";Country="USA";     Tuning="12-tet" },
    @{ Name="Xylophone"; Dir="$VscoDir\Percussion\Xylophone";        Family="percussion";   Region="Europe";  Country="Germany"; Tuning="12-tet" },
    @{ Name="Harp";      Dir="$VscoDir\Strings\Harp";                Family="plucked-string";Region="Europe"; Country="France";  Tuning="12-tet" }
)

$built = 0
foreach ($inst in $InstrumentMap) {
    Write-Host "→ $($inst.Name)..." -ForegroundColor Yellow

    # Try to find the directory (VSCO has varied folder structures)
    $dir = $inst.Dir
    if (-not (Test-Path $dir)) {
        # Try to find it by searching
        $found = Get-ChildItem $VscoDir -Recurse -Directory |
                 Where-Object { $_.Name -like "*$($inst.Name)*" } |
                 Select-Object -First 1
        if ($found) { $dir = $found.FullName }
        else {
            Write-Host "  ✗ Not found — skipping" -ForegroundColor DarkYellow
            continue
        }
    }

    $outFile = Join-Path $OutDir "$($inst.Name.ToLower() -replace ' ','-').aether-instrument"
    Build-Instrument $inst.Name $dir $inst.Family $inst.Region $inst.Country $inst.Tuning $outFile
    $built++
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Done! Built $built instruments" -ForegroundColor Green
Write-Host "  Output: $OutDir" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Green
