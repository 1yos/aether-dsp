# Sample Setup Guide

## Quick Start: Get Drums Working

### Step 1: Download VSCO 2 CE

1. Go to: https://github.com/sgossner/VSCO-2-CE/releases
2. Download the latest release (vsco2-ce-\*.zip, ~1GB)
3. Extract to a temporary folder (e.g., `C:\Temp\VSCO2-CE\`)

### Step 2: Create Sample Directory Structure

Create these folders in your project:

```
assets/
└── samples/
    └── drums-studio/
        ├── kick/
        ├── snare/
        ├── hihat/
        ├── toms/
        └── cymbals/
```

### Step 3: Copy Drum Samples

From the extracted VSCO 2 CE folder, find and copy drum samples:

**Required files** (rename to match these exact names):

```
assets/samples/drums-studio/kick/kick-v1.wav
assets/samples/drums-studio/snare/snare-v1.wav
assets/samples/drums-studio/snare/snare-rim-v1.wav
assets/samples/drums-studio/hihat/hihat-closed-v1.wav
assets/samples/drums-studio/hihat/hihat-open-v1.wav
assets/samples/drums-studio/hihat/hihat-pedal-v1.wav
assets/samples/drums-studio/toms/tom-high-v1.wav
assets/samples/drums-studio/toms/tom-mid-v1.wav
assets/samples/drums-studio/toms/tom-floor-v1.wav
assets/samples/drums-studio/cymbals/crash-v1.wav
assets/samples/drums-studio/cymbals/ride-v1.wav
assets/samples/drums-studio/cymbals/ride-bell-v1.wav
```

### Step 4: Verify Files

Run this PowerShell command to check if all files exist:

```powershell
$files = @(
    "kick/kick-v1.wav",
    "snare/snare-v1.wav",
    "snare/snare-rim-v1.wav",
    "hihat/hihat-closed-v1.wav",
    "hihat/hihat-open-v1.wav",
    "hihat/hihat-pedal-v1.wav",
    "toms/tom-high-v1.wav",
    "toms/tom-mid-v1.wav",
    "toms/tom-floor-v1.wav",
    "cymbals/crash-v1.wav",
    "cymbals/ride-v1.wav",
    "cymbals/ride-bell-v1.wav"
)

$basePath = "assets\samples\drums-studio"
$missing = @()

foreach ($file in $files) {
    $fullPath = Join-Path $basePath $file
    if (-not (Test-Path $fullPath)) {
        $missing += $file
        Write-Host "MISSING: $file" -ForegroundColor Red
    } else {
        Write-Host "OK: $file" -ForegroundColor Green
    }
}

if ($missing.Count -eq 0) {
    Write-Host "`nAll samples found! Ready to build." -ForegroundColor Green
} else {
    Write-Host "`nMissing $($missing.Count) files." -ForegroundColor Yellow
}
```

## Alternative: Use Free Drum Samples

If VSCO 2 CE doesn't have the right drums, you can use these free alternatives:

### Option A: Freesound.org (CC0 Samples)

1. Go to https://freesound.org
2. Search for: "kick drum one shot"
3. Filter by: License = CC0 (Public Domain)
4. Download and rename to match the required filenames above

### Option B: Generate Synthetic Drums

If you can't find samples, I can help you:

1. Improve the kick drum synthesis (add pitch envelope)
2. Add better drum synthesis algorithms
3. Use the current synth engine with better presets

## What Happens Next?

Once samples are in place, I will:

1. **Connect sampler to UI** - Wire `SamplerNode` into the track system
2. **Add instrument selector** - UI to switch between synth and sampler
3. **Test drum playback** - Verify samples load and play correctly
4. **Add more instruments** - Piano, strings, brass, etc.

## Troubleshooting

### "File not found" errors when loading instrument

- Check that file paths in `drums-studio.json` match actual files
- Ensure WAV files are 44.1kHz or 48kHz, 16-bit or 24-bit, mono or stereo
- File names are case-sensitive on Linux/macOS

### Samples sound wrong

- Verify sample rate matches (48kHz recommended)
- Check that samples are not compressed (MP3/OGG won't work, must be WAV)
- Ensure samples are normalized (not clipping or too quiet)

### Out of memory

- Start with lite versions (single velocity layer)
- Use 16-bit WAV instead of 24-bit
- Reduce sample rate to 44.1kHz if needed

## Next Steps

After you've downloaded and organized the samples, let me know and I'll:

1. Update the code to load samples from `assets/samples/`
2. Connect the sampler to the UI track system
3. Add an instrument browser
4. Test everything works

**Estimated time**: 2-3 hours of coding after samples are ready.
