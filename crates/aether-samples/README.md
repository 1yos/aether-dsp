# aether-samples

On-demand sample pack download and management for [AetherDSP](https://github.com/1yos/aether-dsp).

Handles fetching, verifying, extracting, and tracking instrument sample packs distributed via GitHub Releases. Used by Aether Studio to give every instrument its real recorded sound.

## Features

- **Manifest-driven** — a `samples-manifest.json` index describes all available packs with version, size, SHA-256 checksum, and instrument IDs
- **Verified downloads** — every pack is SHA-256 checked after download before extraction
- **Compressed archives** — packs are distributed as `.tar.zst` (Zstandard) for fast downloads
- **Local state tracking** — `installed.json` records which packs are installed and where
- **Progress callbacks** — real-time download progress (bytes downloaded, phase: Downloading / Verifying / Extracting / Complete)
- **Path resolution** — given an instrument ID and a relative sample path, resolves the absolute path on disk

## Storage layout

```
{data_dir}/AetherDSP/samples/
  installed.json              — local state: which packs are installed
  drums-studio-v1/            — extracted pack directory
    kick/
      kick-v1-p1.wav
      ...
  piano-grand-v1/
    ...
```

Platform defaults:

- **Windows**: `%APPDATA%\AetherDSP\samples`
- **Linux**: `~/.local/share/AetherDSP/samples`
- **macOS**: `~/Library/Application Support/AetherDSP/samples`

## Usage

```toml
[dependencies]
aether-samples = { path = "...", features = ["download"] }
```

```rust
use aether_samples::SampleManager;

// Initialize (creates storage directory)
let mut manager = SampleManager::new()?;

// Fetch the manifest from GitHub Releases
manager.fetch_manifest()?;

// Check what's installed
for (id, status) in manager.all_pack_statuses() {
    println!("{id}: {status:?}");
}

// Download a pack with progress reporting
manager.download_pack("drums-studio-v1", |progress| {
    println!("{:.0}% — {:?}",
        progress.bytes_downloaded as f64 / progress.bytes_total as f64 * 100.0,
        progress.phase
    );
})?;

// Resolve a sample path for playback
if let Some(path) = manager.resolve_sample_path("drums-studio", "kick/kick-v1-p1.wav") {
    println!("Load from: {}", path.display());
}
```

## Feature flags

| Feature    | Description                                                               |
| ---------- | ------------------------------------------------------------------------- |
| `download` | Enables `fetch_manifest()` and `download_pack()` via `reqwest` + `ruzstd` |

Without the `download` feature, the crate compiles with only local state management — useful for embedded or offline environments.

## License

MIT — see [LICENSE](../../LICENSE)
