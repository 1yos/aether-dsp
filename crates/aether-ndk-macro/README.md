# aether-ndk-macro

[![crates.io](https://img.shields.io/crates/v/aether-ndk-macro.svg)](https://crates.io/crates/aether-ndk-macro)
[![docs.rs](https://docs.rs/aether-ndk-macro/badge.svg)](https://docs.rs/aether-ndk-macro)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Procedural macro for the [Aether Node Development Kit](https://crates.io/crates/aether-ndk).

Use via `aether-ndk` — don't depend on this crate directly.

```rust
use aether_ndk::prelude::*;

#[aether_node]
pub struct Tremolo {
    #[param(name = "Rate",  min = 0.1, max = 20.0, default = 4.0)]
    rate: f32,
    #[param(name = "Depth", min = 0.0, max = 1.0,  default = 0.5)]
    depth: f32,
    phase: f32,  // internal state — no #[param]
}
```

## What `#[aether_node]` generates

- `Default` impl using `#[param(default = ...)]` values
- `AetherNodeMeta` trait with `type_name()` and `param_defs()`
- Static `PARAM_COUNT` constant
- Strips `#[param]` attributes so the struct compiles cleanly

## License

MIT — see [LICENSE](../../LICENSE)
