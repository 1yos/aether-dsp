# AetherDSP Tutorials

Step-by-step guides to help you get started with AetherDSP.

---

## Getting Started

### [Building Your First Synthesizer](first-synth.md)

**Level:** Beginner  
**Time:** 30-45 minutes

Learn the fundamentals by building a complete monophonic synthesizer:

- Setting up a new project with AetherDSP
- Creating audio output with CPAL
- Adding an oscillator (sound source)
- Shaping tone with a filter
- Controlling amplitude with an envelope
- Adding MIDI keyboard control

**What you'll build:** A playable synthesizer with oscillator → filter → envelope signal chain, controllable via MIDI keyboard.

---

## Intermediate Topics

### [Creating Custom DSP Nodes](custom-nodes.md)

**Level:** Intermediate  
**Time:** 20-30 minutes  
**Prerequisites:** Completed First Synth Tutorial

Learn to create your own DSP effects using the Node Development Kit (NDK):

- Using the `#[aether_node]` macro
- Implementing the `DspProcess` trait
- Adding parameters with validation
- Testing with unit tests and property tests
- Publishing your nodes to crates.io

**What you'll build:** Three custom effects - Tremolo (amplitude modulation), Distortion (waveshaping), and a Simple Filter (one-pole lowpass).

---

### [Microtonal Music with Custom Tuning Systems](tuning-systems.md)

**Level:** Intermediate  
**Time:** 20-30 minutes  
**Prerequisites:** Completed First Synth Tutorial

Explore world music and alternative tuning systems:

- Introduction to microtonality
- Ethiopian Tizita scale (pentatonic with microtones)
- Arabic Maqam scales (quarter-tones)
- Just intonation (pure harmonic ratios)
- Creating custom tuning tables
- Building a microtonal sequencer

**What you'll build:** A synthesizer that plays melodies in Ethiopian Tizita scale, with examples of Arabic Maqam and Just Intonation.

---

## Tutorial Index

| Tutorial          | Level        | Time   | Topics                                            |
| ----------------- | ------------ | ------ | ------------------------------------------------- |
| First Synthesizer | Beginner     | 30-45m | CPAL, Oscillator, Filter, Envelope, MIDI          |
| Custom DSP Nodes  | Intermediate | 20-30m | NDK, DspProcess, Parameters, Testing              |
| Tuning Systems    | Intermediate | 20-30m | Microtonality, Ethiopian, Arabic, Just Intonation |

---

## Prerequisites

All tutorials assume:

- **Rust knowledge:** Basic understanding of Rust syntax, ownership, and traits
- **Development environment:** Rust 1.70+ installed with cargo
- **Audio hardware:** Working audio output device
- **MIDI hardware (optional):** MIDI keyboard for interactive tutorials

---

## Learning Path

### Path 1: Audio Developer

1. **First Synthesizer** - Learn the basics
2. **Custom DSP Nodes** - Build your own effects
3. Explore the [NDK Guide](../sdk/NDK_GUIDE.md) for advanced node development

### Path 2: World Music Producer

1. **First Synthesizer** - Learn the basics
2. **Tuning Systems** - Explore microtonal music
3. Experiment with the included world music presets

### Path 3: Plugin Developer

1. **First Synthesizer** - Learn the basics
2. **Custom DSP Nodes** - Build your own effects
3. See `aether-plugin` crate for VST3/CLAP export

---

## Additional Resources

### Documentation

- [AetherDSP Core API](https://docs.rs/aetherdsp-core) - Complete API reference
- [AetherDSP Nodes API](https://docs.rs/aetherdsp-nodes) - Built-in DSP nodes
- [NDK Guide](../sdk/NDK_GUIDE.md) - Node Development Kit documentation
- [Migration Guide](../../crates/aether-core/MIGRATION.md) - Upgrading between versions

### Examples

- [Core Examples](../../crates/aether-core/examples/) - Basic engine usage
- [Node Examples](../../crates/aether-nodes/examples/) - DSP node demonstrations
- [MIDI Examples](../../crates/aether-midi/examples/) - MIDI integration

### Community

- [GitHub Discussions](https://github.com/1yos/aether-dsp/discussions) - Ask questions
- [GitHub Issues](https://github.com/1yos/aether-dsp/issues) - Report bugs
- [Contributing Guide](../../CONTRIBUTING.md) - Contribute to AetherDSP

---

## Troubleshooting

### Common Issues

**No audio output:**

- Check that your audio device is selected correctly
- Verify the sample rate matches your device
- Check system volume and mute settings

**Compilation errors:**

- Ensure Rust 1.70+ is installed: `rustc --version`
- Update dependencies: `cargo update`
- Clean build artifacts: `cargo clean`

**MIDI not working:**

- Check MIDI device is connected and powered on
- Verify MIDI port selection in code
- Test with a MIDI monitor tool

**Clicks and pops:**

- Increase buffer size in CPAL configuration
- Check CPU usage (may be too high)
- Ensure parameter smoothing is enabled

### Getting Help

If you're stuck:

1. Check the [FAQ](../../crates/aether-core/README.md#faq) in the core README
2. Search [GitHub Issues](https://github.com/1yos/aether-dsp/issues)
3. Ask in [GitHub Discussions](https://github.com/1yos/aether-dsp/discussions)
4. Review the [Common Pitfalls](../../crates/aether-core/README.md#common-pitfalls) section

---

## Contributing Tutorials

Want to contribute a tutorial? We'd love to have more!

**Tutorial ideas:**

- Building a drum machine
- Creating a vocoder
- Implementing a sequencer
- Building a polyphonic synthesizer
- Creating a sampler instrument
- Implementing audio effects chains

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

**Ready to start?** Begin with [Building Your First Synthesizer](first-synth.md)!
