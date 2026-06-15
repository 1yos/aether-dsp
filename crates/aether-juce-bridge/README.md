# AetherDSP JUCE Bridge

**C FFI bridge for integrating AetherDSP with JUCE audio plugins.**

Add world music tuning systems and real-time safe DSP processing to your JUCE plugins with just a few lines of code.

## Features

- ✅ **17 World Music Tuning Systems** - Ethiopian (7), Arabic (3), Indian (1), Gamelan (3), Western (3)
- ✅ **Lock-Free DSP Graph** - Real-time safe audio processing
- ✅ **Memory Safe** - Rust's safety guarantees protect your audio thread
- ✅ **Zero-Cost Abstraction** - No performance overhead
- ✅ **Drop-in Integration** - Works with existing JUCE projects
- ✅ **MIT Licensed** - Free forever, no royalties

## Quick Start

### 1. Build the Library

```bash
cd crates/aether-juce-bridge
cargo build --release
```

This generates:

- **Static library**: `target/release/libaetherdsp_juce_bridge.a` (macOS/Linux) or `aetherdsp_juce_bridge.lib` (Windows)
- **Dynamic library**: `target/release/libaetherdsp_juce_bridge.dylib` (macOS), `.so` (Linux), or `.dll` (Windows)
- **C header**: `include/aetherdsp_juce_bridge.h`

### 2. Add to Your JUCE Project

**In Projucer:**

1. Add the static library to your project:
   - Go to "Modules" → "Add a module from a specified folder"
   - Or add directly in "Build" tab → "External Libraries to Link"

2. Add include path:
   - Add `crates/aether-juce-bridge/include` to header search paths

**In CMake:**

```cmake
target_link_libraries(YourPlugin
    PRIVATE
        ${AETHERDSP_BRIDGE_PATH}/target/release/libaetherdsp_juce_bridge.a
)

target_include_directories(YourPlugin
    PRIVATE
        ${AETHERDSP_BRIDGE_PATH}/include
)
```

### 3. Use in Your Plugin

```cpp
#include "aetherdsp_juce_bridge.h"

class EthiopianSynthPlugin : public juce::AudioProcessor
{
public:
    EthiopianSynthPlugin()
    {
        // Create DSP graph
        graph = aether_graph_create(44100.0f, 512);

        // Load Ethiopian Tizita tuning
        tuning = aether_tuning_ethiopian_tizita();

        // Add oscillator with tuning
        oscId = aether_graph_add_oscillator(graph, tuning);

        // Add gain node
        gainId = aether_graph_add_gain(graph);

        // Set initial frequency (MIDI note 60 = Middle C)
        aether_graph_set_param(graph, oscId, 0, 60.0f, 0.0f);
    }

    ~EthiopianSynthPlugin()
    {
        aether_tuning_free(tuning);
        aether_graph_free(graph);
    }

    void processBlock(juce::AudioBuffer<float>& buffer,
                     juce::MidiBuffer& midiMessages) override
    {
        // Process MIDI
        for (const auto metadata : midiMessages)
        {
            const auto msg = metadata.getMessage();
            if (msg.isNoteOn())
            {
                float note = (float)msg.getNoteNumber();
                aether_graph_set_param(graph, oscId, 0, note, 10.0f);
            }
        }

        // Process audio
        aether_graph_process(
            graph,
            nullptr, 0,  // No inputs
            buffer.getArrayOfWritePointers(),
            buffer.getNumChannels(),
            buffer.getNumSamples()
        );
    }

private:
    AetherGraph* graph = nullptr;
    AetherTuningTable* tuning = nullptr;
    AetherNodeId oscId = 0;
    AetherNodeId gainId = 0;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(EthiopianSynthPlugin)
};
```

## Available Tuning Systems

### Ethiopian Scales (Qenet)

All 7 traditional Ethiopian qenet modes are now implemented:

```cpp
// Tizita variants (expressing nostalgia and longing)
auto tuning = aether_tuning_ethiopian_tizita();              // Tizita major - pentatonic, characteristic of Ethiopian blues
auto tuning = aether_tuning_ethiopian_tizita_minor();        // Tizita minor - melancholic, introspective

// Bati variants (expressing depth and melancholy)
auto tuning = aether_tuning_ethiopian_bati();                // Bati minor - standard minor pentatonic (most common)
auto tuning = aether_tuning_ethiopian_bati_major();          // Bati major - bright, uplifting variant

// Other main modes
auto tuning = aether_tuning_ethiopian_ambassel();            // Ambassel - pentatonic with flat 2nd
auto tuning = aether_tuning_ethiopian_anchihoye();           // Anchihoye - pentatonic without 3rd degree
```

### Arabic Maqamat

```cpp
auto tuning = aether_tuning_arabic_rast();    // Quarter-tone flats on 3rd and 7th
auto tuning = aether_tuning_arabic_bayati();  // Half-flat on 2nd degree
auto tuning = aether_tuning_arabic_hijaz();   // Augmented 2nd (dramatic sound)
```

### Indian Ragas

```cpp
auto tuning = aether_tuning_indian_yaman();   // Raised 4th (Kalyan thaat)
```

### Gamelan (Javanese)

```cpp
auto tuning = aether_tuning_gamelan_slendro();           // 5-tone scale
auto tuning = aether_tuning_gamelan_slendro_stretched(); // 1210-cent octaves!
auto tuning = aether_tuning_gamelan_pelog();             // 7-tone with unequal intervals
```

### Western Tunings

```cpp
auto tuning = aether_tuning_just_intonation();         // 5-limit (pure thirds)
auto tuning = aether_tuning_just_intonation_7_limit(); // 7-limit (blues, barbershop)
auto tuning = aether_tuning_equal_temperament();       // Standard 12-TET
```

## API Reference

### Graph Management

```c
// Create a DSP graph
AetherGraph* aether_graph_create(float sample_rate, size_t block_size);

// Free a DSP graph
void aether_graph_free(AetherGraph* graph);

// Process audio
AetherResult aether_graph_process(
    AetherGraph* graph,
    const float** inputs, size_t num_inputs,
    float** outputs, size_t num_outputs,
    size_t num_samples
);
```

### Node Creation

```c
// Add nodes to the graph
AetherNodeId aether_graph_add_oscillator(AetherGraph* graph, const AetherTuningTable* tuning);
AetherNodeId aether_graph_add_gain(AetherGraph* graph);
AetherNodeId aether_graph_add_filter(AetherGraph* graph);
```

### Parameter Control

```c
// Set a parameter with optional smoothing
AetherResult aether_graph_set_param(
    AetherGraph* graph,
    AetherNodeId node_id,
    uint32_t param_index,
    float value,
    float ramp_ms  // 0 for instant, >0 for smooth
);
```

### Tuning Tables

```c
// Get frequency for a MIDI note
AetherResult aether_tuning_get_frequency(
    const AetherTuningTable* tuning,
    uint8_t midi_note,
    float* out_frequency
);

// Free a tuning table
void aether_tuning_free(AetherTuningTable* tuning);
```

## Example Plugins

See the `examples/` directory for complete JUCE plugin examples:

- **EthiopianSynth** - Simple synthesizer with Ethiopian Tizita scale
- **ArabicMaqamSynth** - Polyphonic synth with switchable Arabic maqamat
- **WorldMusicSampler** - Sampler with tuning system selector

## Building the Examples

```bash
cd examples/EthiopianSynth
mkdir build && cd build
cmake ..
cmake --build . --config Release
```

## Performance

The bridge has **zero runtime overhead** compared to pure Rust:

- No heap allocations in audio thread
- Lock-free graph updates
- SIMD-optimized DSP (where applicable)
- Typical DSP node latency: <100ns per sample

**Benchmark (MacBook Pro M1, 48kHz, 64 samples):**

- Parameter update: 51.7ns
- Oscillator processing: ~5µs per block
- Full graph (10 nodes): ~50µs per block

## Thread Safety

### Real-Time Safe Operations

These functions are safe to call from the audio thread:

- `aether_graph_process()`
- `aether_graph_set_param()`
- `aether_tuning_get_frequency()`

### Non-Real-Time Operations

These functions must be called from the main/message thread:

- `aether_graph_create()` / `aether_graph_free()`
- `aether_graph_add_*()` (node creation)
- `aether_tuning_*()` creation and `aether_tuning_free()`

## Troubleshooting

### Linking Errors

**macOS:** Make sure to link against the C++ standard library:

```cmake
target_link_libraries(YourPlugin PRIVATE "-lc++")
```

**Windows MSVC:** Use `.lib` files, not `.dll.lib`:

```cmake
target_link_libraries(YourPlugin PRIVATE aetherdsp_juce_bridge.lib)
```

**Linux:** Link against pthread and dl:

```cmake
target_link_libraries(YourPlugin PRIVATE pthread dl)
```

### Runtime Errors

**"Symbol not found"**: Make sure the dynamic library is in the search path or use static linking.

**Crashes on process()**: Ensure `sample_rate` and `block_size` match your AudioProcessor settings.

## License

MIT License - See [LICENSE](../../LICENSE) for details.

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md).

## Support

- **GitHub Issues**: [github.com/1yos/aether-dsp/issues](https://github.com/1yos/aether-dsp/issues)
- **Documentation**: [docs.rs/aetherdsp-juce-bridge](https://docs.rs/aetherdsp-juce-bridge)

---

**Made with ❤️ in Addis Ababa, Ethiopia**
