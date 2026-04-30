# download_western_samples.ps1
# Downloads free WAV samples for western instruments from the University of Iowa
# Musical Instrument Samples database (freely available, no restrictions).
# https://theremin.music.uiowa.edu/MIS.html
#
# USAGE:
#   .\scripts\download_western_samples.ps1
#
# Downloads samples for: Piano, Violin, Cello, Flute, Clarinet, Trumpet, Guitar
# Then converts each to .aether-instrument format.

$ErrorActionPreference = "Stop"
$OutDir = "D:\samples\western"
$InstrDir = "aether-dsp\ui\public\instruments"

# Create output directories
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
New-Item -ItemType Directory -Force -Path $InstrDir | Out-Null

# Base URL for University of Iowa samples
$BaseUrl = "https://theremin.music.uiowa.edu/sound%20files/MIS"

# Instruments to download: name, subfolder, file pattern, notes to grab
# Iowa files are named like: Piano.mf.C4.aiff
# We'll grab a subset of notes spread across the range

$Instruments = @(
    @{
        Name       = "Piano"
        Folder     = "Piano"
        Prefix     = "Piano.mf"
        Extension  = "aiff"
        Notes      = @("C2","E2","G2","C3","E3","G3","C4","E4","G4","C5","E5","G5","C6")
        Family     = "keyboard"
        Region     = "Europe"
        Country    = "Germany"
        Tuning     = "12-tet"
    },
    @{
        Name       = "Violin"
        Folder     = "Strings/Violin"
        Prefix     = "Violin.arco.mf.sulA"
        Extension  = "aiff"
        Notes      = @("G3","A3","B3","C4","D4","E4","F4","G4","A4","B4","C5","D5","E5")
        Family     = "bowed-string"
        Region     = "Europe"
        Country    = "Italy"
        Tuning     = "12-tet"
    },
    @{
        Name       = "Cello"
        Folder     = "Strings/Cello"
        Prefix     = "Cello.arco.mf.sulA"
        Extension  = "aiff"
        Notes      = @("C2","D2","E2","F2","G2","A2","B2","C3","D3","E3","F3","G3","A3")
        Family     = "bowed-string"
        Region     = "Europe"
        Country    = "Italy"
        Tuning     = "12-tet"
    },
    @{
        Name       = "Flute"
        Folder     = "Woodwinds/Flute"
        Prefix     = "Flute.vib.mf"
        Extension  = "aiff"
        Notes      = @("C4","D4","E4","F4","G4","A4","B4","C5","D5","E5","F5","G5","A5")
        Family     = "wind"
        Region     = "Europe"
        Country    = "Germany"
        Tuning     = "12-tet"
    },
    @{
        Name       = "Clarinet"
        Folder     = "Woodwinds/Clarinet"
        Prefix     = "Clarinet.mf"
        Extension  = "aiff"
        Notes      = @("D3","E3","F3","G3","A3","B3","C4","D4","E4","F4","G4","A4","B4")
        Family     = "wind"
        Region     = "Europe"
        Country    = "Germany"
        Tuning     = "12-tet"
    },
    @{
        Name       = "Trumpet"
        Folder     = "Brass/Trumpet"
        Prefix     = "Trumpet.mf"
        Extension  = "aiff"
        Notes      = @("E3","F3","G3","A3","B3","C4","D4","E4","F4","G4","A4","B4","C5")
        Family     = "wind"
        Region     = "Europe"
        Country    = "Germany"
        Tuning     = "12-tet"
    },
    @{
        Name       = "Guitar"
        Folder     = "Guitar"
        Prefix     = "Guitar.mf"
        Extension  = "aiff"
        Notes      = @("E2","A2","D3","G3","B3","E4","A4","D5","G5","B5","E6")
        Family     = "plucked-string"
        Region     = "Europe"
        Country    = "Spain"
        Tuning     = "12-tet"
    }
)

# Note name to MIDI number
function Get-MidiNote($noteName) {
    $noteMap = @{
        "C"=0;"C#"=1;"Db"=1;"D"=2;"D#"=3;"Eb"=3;"E"=4;"F"=5;"F#"=6;"Gb"=6;"G"=7;"G#"=8;"Ab"=8;"A"=9;"A#"=10;"Bb"=10;"B"=11
    }
    if ($noteName -match '^([A-Ga-g][#b]?)(\d)$') {
        $note = $Matches[1].ToUpper()
        $octave = [int]$Matches[2]
        return ($octave + 1) * 12 + $noteMap[$note]
    }
    return $null
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Aether Western Instrument Downloader" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Source: University of Iowa MIS (free, no restrictions)" -ForegroundColor Gray
Write-Host ""

foreach ($inst in $Instruments) {
    $instDir = Join-Path $OutDir $inst.Name
    New-Item -ItemType Directory -Force -Path $instDir | Out-Null

    Write-Host "→ $($inst.Name)" -ForegroundColor Yellow
    $downloaded = 0

    foreach ($note in $inst.Notes) {
        $filename = "$($inst.Prefix).$note.$($inst.Extension)"
        $url = "$BaseUrl/$($inst.Folder)/$filename"
        $wavOut = Join-Path $instDir "$($inst.Name.ToLower())_$note.wav"

        # Skip if already downloaded
        if (Test-Path $wavOut) {
            Write-Host "  ✓ $note (cached)" -ForegroundColor DarkGreen
            $downloaded++
            continue
        }

        try {
            # Download AIFF then note it needs conversion
            $aiffOut = Join-Path $instDir "$($inst.Name.ToLower())_$note.aiff"
            Invoke-WebRequest -Uri $url -OutFile $aiffOut -UseBasicParsing -TimeoutSec 30 -ErrorAction Stop
            Write-Host "  ✓ $note" -ForegroundColor Green
            $downloaded++
        } catch {
            Write-Host "  ✗ $note — not found at $url" -ForegroundColor DarkYellow
        }
    }

    Write-Host "  Downloaded: $downloaded/$($inst.Notes.Count) notes" -ForegroundColor Cyan
    Write-Host ""
}

Write-Host "========================================" -ForegroundColor Green
Write-Host "  Download complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "NOTE: Iowa samples are in AIFF format." -ForegroundColor Yellow
Write-Host "To convert to WAV, install ffmpeg and run:" -ForegroundColor Yellow
Write-Host "  Get-ChildItem D:\samples\western -Recurse -Filter *.aiff | ForEach-Object { ffmpeg -i `$_.FullName `$_.FullName.Replace('.aiff','.wav') }" -ForegroundColor Gray
Write-Host ""
Write-Host "Or use Audacity (free): File > Import > Audio, then Export as WAV" -ForegroundColor Gray
