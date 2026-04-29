# aether-ndk — Node Development Kit v0.1

Build custom DSP nodes for AetherDSP in minutes.

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

impl DspProcess for Tremolo {
    fn process(&mut self, inputs: &NodeInputs, output: &mut NodeOutput,
               params: &mut ParamBlock, sample_rate: f32) {
        let input = inputs.get(0);
        for (i, out) in output.iter_mut().enumerate() {
            let lfo = 1.0 - params.get(1).current * 0.5
                * (1.0 - (self.phase * std::f32::consts::TAU).cos());
            *out = input[i] * lfo;
            self.phase = (self.phase + params.get(0).current / sample_rate).fract();
            params.tick_all();
        }
    }
}
```

## What `#[aether_node]` generates

- `Default` impl using `#[param(default = ...)]` values
- `AetherNodeMeta` trait with `type_name()` and `param_defs()`
- Static `PARAM_COUNT` constant
- Strips `#[param]` attributes from the output struct

## RT Safety

Your `process` impl must be:

- **No allocation** — no `Box`, `Vec`, `String`
- **No locks** — no `Mutex`, `RwLock`
- **No I/O** — no `println!`, file access
- **Bounded** — no unbounded loops or recursion

## Examples

```bash
cargo run --example tremolo       -p aether-ndk
cargo run --example bitcrusher    -p aether-ndk
cargo run --example registry_demo -p aether-ndk
```

## Docs

- [NDK Guide](../../docs/sdk/NDK_GUIDE.md)
- [Manifest Spec](../../docs/sdk/MANIFEST_SPEC.md)
- [CLI Reference](../../docs/sdk/CLI_REFERENCE.md)
