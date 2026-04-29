# AetherDSP v0.1 Release Checklist

## Build & Compile

- [x] `cargo check --workspace` — zero errors, zero warnings
- [x] `cargo build --release` — optimized binary produced
- [x] `cargo bench -p aether-core` — Criterion suite running
- [x] `npm run build` — production UI assets generated (296 KB JS)

## Code Quality

- [x] `cargo clippy --workspace -- -D warnings` passes
- [x] All unused imports removed
- [x] TypeScript strict mode clean
- [ ] `cargo fmt --all` — run before tagging
- [ ] `cargo doc --workspace --no-deps` — verify docs build

## Functional Verification

- [ ] Sine oscillator produces audible output at 440 Hz
- [ ] Filter cutoff sweep produces no clicks or XRuns
- [ ] Dynamic node add/remove during playback — no glitches
- [ ] WebSocket param update reflected in audio within 20 ms
- [ ] UI loads and displays graph snapshot on connect

## Performance Targets

| Metric                 | Target     | Status             |
| ---------------------- | ---------- | ------------------ |
| Buffer size            | 64 samples | ✅                 |
| Latency                | ≤ 1.33 ms  | ✅                 |
| `param_fill_buffer_64` | < 100 ns   | ✅ 51.7 ns         |
| Arena alloc/dealloc    | O(1)       | ✅                 |
| XRuns under load       | 0          | Pending audio test |

## Documentation

- [x] Architecture diagram (`docs/architecture.svg`)
- [x] Release checklist (`docs/RELEASE_CHECKLIST.md`)
- [x] Conference paper draft (`docs/paper_draft.md`)
- [ ] API docs via `cargo doc`
- [ ] README.md with build instructions

## Repository

- [ ] `git init && git add . && git commit -m "AetherDSP v0.1"`
- [ ] Tag: `git tag -a v0.1.0 -m "Initial release"`
- [ ] Push to GitHub
- [ ] Create GitHub Release with binary artifacts

## Plugin Support (v0.2 target)

- [ ] Add `aether-plugin` crate to workspace
- [ ] Implement NIH-plug CLAP wrapper
- [ ] Export `aether_plugin_main!` entry point
- [ ] Test in Reaper / Bitwig
