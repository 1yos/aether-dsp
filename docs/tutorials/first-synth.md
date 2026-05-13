# Tutorial: Building Your First Synthesizer

**Level:** Beginner  
**Time:** 30-45 minutes  
**Prerequisites:** Basic Rust knowledge

In this tutorial, you'll build a complete monophonic synthesizer with:

- Oscillator (sound source)
- Filter (tone shaping)
- Envelope (amplitude control)
- Audio output via CPAL

---

## Table of Contents

1. [Project Setup](#project-setup)
2. [Basic Audio Output](#basic-audio-output)
3. [Adding an Oscillator](#adding-an-oscillator)
4. [Adding a Filter](#adding-a-filter)
5. [Adding an Envelope](#adding-an-envelope)
6. [MIDI Control](#midi-control)
7. [Next Steps](#next-steps)

---

## Project Setup

### Step 1: Create a New Project

```bash
cargo new my-first-synth
cd my-first-synth
```

### Step 2: Add Dependencies

Edit `Cargo.toml`:

```toml
[dependencies]
aetherdsp-core = "0.1.4"
aetherdsp-nodes = "0.2.3"
cpal = "0.15"
```

### Step 3: Verify Installation

```bash
cargo build
```

You should see the dependencies downloading and compiling successfully.

---

## Basic Audio Output

Let's start by getting audio output working with CPAL.

### Step 1: Create the Audio Callback

Edit `src/main.rs`:

```rust
use aetherdsp_core::scheduler::Scheduler;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize CPAL
    let host = cpal::default_host();
    let device = host.default_output_device()
        .expect("No output device available");

    println!("Output device: {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default config: {:?}", config);

    // Create scheduler
    let sample_rate = config.sample_rate().0 as f32;
    let mut scheduler = Scheduler::new(sample_rate);

    // Build audio stream
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Process audio
            scheduler.process_block_simple(data);
        },
        |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    // Start audio
    stream.play()?;

    println!("Audio running. Press Ctrl+C to stop.");

    // Keep running
    std::thread::park();

    Ok(())
}
```

### Step 2: Test It

```bash
cargo run
```

You should see:

```
Output device: Default Audio Device
Default config: ...
Audio running. Press Ctrl+C to stop.
```

**Note:** You won't hear anything yet - the graph is empty!

---

## Adding an Oscillator

Now let's add a sound source.

### Step 1: Import Nodes

Add to the top of `src/main.rs`:

```rust
use aetherdsp_nodes::oscillator::Oscillator;
```

### Step 2: Add Oscillator to Graph

Replace the scheduler creation with:

```rust
// Create scheduler
let sample_rate = config.sample_rate().0 as f32;
let mut scheduler = Scheduler::new(sample_rate);

// Add oscillator (440 Hz = A4)
let osc = Box::new(Oscillator::new(440.0));
let osc_id = scheduler.graph.add_node(osc)
    .expect("Failed to add oscillator");

// Set as output
scheduler.graph.set_output_node(osc_id);

println!("Oscillator added at 440 Hz");
```

### Step 3: Test It

```bash
cargo run
```

**You should hear a 440 Hz sine wave!** 🎵

Press Ctrl+C to stop.

---

## Adding a Filter

Let's shape the tone with a lowpass filter.

### Step 1: Import Filter

Add to imports:

```rust
use aetherdsp_nodes::filter::Filter;
```

### Step 2: Add Filter to Graph

Replace the graph building code:

```rust
// Create scheduler
let sample_rate = config.sample_rate().0 as f32;
let mut scheduler = Scheduler::new(sample_rate);

// Add oscillator
let osc = Box::new(Oscillator::new(440.0));
let osc_id = scheduler.graph.add_node(osc)
    .expect("Failed to add oscillator");

// Add lowpass filter (1000 Hz cutoff, 0.7 resonance)
let filter = Box::new(Filter::lowpass(1000.0, 0.7));
let filter_id = scheduler.graph.add_node(filter)
    .expect("Failed to add filter");

// Connect: Oscillator → Filter
scheduler.graph.connect(osc_id, filter_id, 0);

// Set filter as output
scheduler.graph.set_output_node(filter_id);

println!("Oscillator → Filter chain created");
```

### Step 3: Test It

```bash
cargo run
```

**You should hear a filtered tone** - warmer and less bright than before.

---

## Adding an Envelope

Let's add an ADSR envelope for amplitude control.

### Step 1: Import Envelope

Add to imports:

```rust
use aetherdsp_nodes::envelope::Envelope;
```

### Step 2: Add Envelope to Graph

Replace the graph building code:

```rust
// Create scheduler
let sample_rate = config.sample_rate().0 as f32;
let mut scheduler = Scheduler::new(sample_rate);

// Add oscillator
let osc = Box::new(Oscillator::new(440.0));
let osc_id = scheduler.graph.add_node(osc)
    .expect("Failed to add oscillator");

// Add lowpass filter
let filter = Box::new(Filter::lowpass(1000.0, 0.7));
let filter_id = scheduler.graph.add_node(filter)
    .expect("Failed to add filter");

// Add ADSR envelope
let mut envelope = Envelope::new();
envelope.set_attack(0.01);   // 10ms attack
envelope.set_decay(0.1);     // 100ms decay
envelope.set_sustain(0.7);   // 70% sustain level
envelope.set_release(0.2);   // 200ms release
let env_id = scheduler.graph.add_node(Box::new(envelope))
    .expect("Failed to add envelope");

// Connect: Oscillator → Filter → Envelope
scheduler.graph.connect(osc_id, filter_id, 0);
scheduler.graph.connect(filter_id, env_id, 0);

// Set envelope as output
scheduler.graph.set_output_node(env_id);

println!("Oscillator → Filter → Envelope chain created");
```

### Step 3: Trigger the Envelope

We need to send a gate signal to trigger the envelope. For now, let's trigger it at startup:

```rust
// Trigger envelope (gate on)
use aetherdsp_core::command::Command;
use aetherdsp_core::param::Param;

// Send gate=1.0 to envelope (parameter index 4 is gate)
let gate_cmd = Command::UpdateParam {
    node: env_id,
    param_index: 4,
    new_param: Param::new(1.0),
};

// We'll need to send this via command ring in a real synth
// For now, we'll manually update it
if let Some(record) = scheduler.graph.arena.get_mut(env_id) {
    record.params.params[4] = Param::new(1.0);
}
```

### Step 4: Test It

```bash
cargo run
```

**You should hear the envelope shape the sound** - it fades in (attack) and sustains.

---

## MIDI Control

Let's add MIDI input to control the synthesizer.

### Step 1: Add MIDI Dependency

Edit `Cargo.toml`:

```toml
[dependencies]
aetherdsp-core = "0.1.4"
aetherdsp-nodes = "0.2.3"
cpal = "0.15"
midir = "0.9"
```

### Step 2: Create MIDI Handler

Add to `src/main.rs`:

```rust
use midir::{MidiInput, Ignore};
use std::sync::{Arc, Mutex};

fn setup_midi(scheduler: Arc<Mutex<Scheduler>>, osc_id: aetherdsp_core::arena::NodeId, env_id: aetherdsp_core::arena::NodeId) -> Result<(), Box<dyn std::error::Error>> {
    let mut midi_in = MidiInput::new("AetherDSP")?;
    midi_in.ignore(Ignore::None);

    // List MIDI ports
    let ports = midi_in.ports();
    if ports.is_empty() {
        println!("No MIDI input ports available");
        return Ok(());
    }

    println!("Available MIDI ports:");
    for (i, port) in ports.iter().enumerate() {
        println!("  {}: {}", i, midi_in.port_name(port)?);
    }

    // Connect to first port
    let port = &ports[0];
    println!("Connecting to: {}", midi_in.port_name(port)?);

    let _conn = midi_in.connect(port, "aether-input", move |_stamp, message, _| {
        if message.len() >= 3 {
            let status = message[0];
            let note = message[1];
            let velocity = message[2];

            match status & 0xF0 {
                0x90 if velocity > 0 => {
                    // Note On
                    let freq = midi_note_to_freq(note);
                    println!("Note On: {} ({:.2} Hz)", note, freq);

                    let mut sched = scheduler.lock().unwrap();

                    // Update oscillator frequency
                    if let Some(record) = sched.graph.arena.get_mut(osc_id) {
                        record.params.params[0] = Param::new(freq);
                    }

                    // Trigger envelope
                    if let Some(record) = sched.graph.arena.get_mut(env_id) {
                        record.params.params[4] = Param::new(1.0); // Gate on
                    }
                }
                0x80 | 0x90 => {
                    // Note Off
                    println!("Note Off: {}", note);

                    let mut sched = scheduler.lock().unwrap();

                    // Release envelope
                    if let Some(record) = sched.graph.arena.get_mut(env_id) {
                        record.params.params[4] = Param::new(0.0); // Gate off
                    }
                }
                _ => {}
            }
        }
    }, ())?;

    // Keep connection alive
    std::mem::forget(_conn);

    Ok(())
}

fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}
```

### Step 3: Update Main Function

Replace `main()`:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize CPAL
    let host = cpal::default_host();
    let device = host.default_output_device()
        .expect("No output device available");

    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;

    // Create scheduler
    let mut scheduler = Scheduler::new(sample_rate);

    // Build synth graph
    let osc = Box::new(Oscillator::new(440.0));
    let osc_id = scheduler.graph.add_node(osc).unwrap();

    let filter = Box::new(Filter::lowpass(1000.0, 0.7));
    let filter_id = scheduler.graph.add_node(filter).unwrap();

    let mut envelope = Envelope::new();
    envelope.set_attack(0.01);
    envelope.set_decay(0.1);
    envelope.set_sustain(0.7);
    envelope.set_release(0.2);
    let env_id = scheduler.graph.add_node(Box::new(envelope)).unwrap();

    scheduler.graph.connect(osc_id, filter_id, 0);
    scheduler.graph.connect(filter_id, env_id, 0);
    scheduler.graph.set_output_node(env_id);

    println!("Synth graph created: Oscillator → Filter → Envelope");

    // Wrap scheduler in Arc<Mutex<>> for MIDI thread
    let scheduler = Arc::new(Mutex::new(scheduler));
    let scheduler_clone = scheduler.clone();

    // Setup MIDI
    setup_midi(scheduler_clone, osc_id, env_id)?;

    // Build audio stream
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut sched = scheduler.lock().unwrap();
            sched.process_block_simple(data);
        },
        |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    stream.play()?;

    println!("Synth running. Play your MIDI keyboard!");
    println!("Press Ctrl+C to stop.");

    std::thread::park();

    Ok(())
}
```

### Step 4: Test with MIDI

```bash
cargo run
```

**Play your MIDI keyboard** - you should hear notes!

---

## Complete Code

Here's the final `src/main.rs`:

```rust
use aetherdsp_core::scheduler::Scheduler;
use aetherdsp_core::param::Param;
use aetherdsp_nodes::oscillator::Oscillator;
use aetherdsp_nodes::filter::Filter;
use aetherdsp_nodes::envelope::Envelope;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use midir::{MidiInput, Ignore};
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize CPAL
    let host = cpal::default_host();
    let device = host.default_output_device()
        .expect("No output device available");

    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;

    // Create scheduler
    let mut scheduler = Scheduler::new(sample_rate);

    // Build synth graph
    let osc = Box::new(Oscillator::new(440.0));
    let osc_id = scheduler.graph.add_node(osc).unwrap();

    let filter = Box::new(Filter::lowpass(1000.0, 0.7));
    let filter_id = scheduler.graph.add_node(filter).unwrap();

    let mut envelope = Envelope::new();
    envelope.set_attack(0.01);
    envelope.set_decay(0.1);
    envelope.set_sustain(0.7);
    envelope.set_release(0.2);
    let env_id = scheduler.graph.add_node(Box::new(envelope)).unwrap();

    scheduler.graph.connect(osc_id, filter_id, 0);
    scheduler.graph.connect(filter_id, env_id, 0);
    scheduler.graph.set_output_node(env_id);

    println!("Synth graph created: Oscillator → Filter → Envelope");

    // Wrap scheduler in Arc<Mutex<>> for MIDI thread
    let scheduler = Arc::new(Mutex::new(scheduler));
    let scheduler_clone = scheduler.clone();

    // Setup MIDI
    setup_midi(scheduler_clone, osc_id, env_id)?;

    // Build audio stream
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut sched = scheduler.lock().unwrap();
            sched.process_block_simple(data);
        },
        |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    stream.play()?;

    println!("Synth running. Play your MIDI keyboard!");
    println!("Press Ctrl+C to stop.");

    std::thread::park();

    Ok(())
}

fn setup_midi(
    scheduler: Arc<Mutex<Scheduler>>,
    osc_id: aetherdsp_core::arena::NodeId,
    env_id: aetherdsp_core::arena::NodeId
) -> Result<(), Box<dyn std::error::Error>> {
    let mut midi_in = MidiInput::new("AetherDSP")?;
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();
    if ports.is_empty() {
        println!("No MIDI input ports available");
        return Ok(());
    }

    println!("Available MIDI ports:");
    for (i, port) in ports.iter().enumerate() {
        println!("  {}: {}", i, midi_in.port_name(port)?);
    }

    let port = &ports[0];
    println!("Connecting to: {}", midi_in.port_name(port)?);

    let _conn = midi_in.connect(port, "aether-input", move |_stamp, message, _| {
        if message.len() >= 3 {
            let status = message[0];
            let note = message[1];
            let velocity = message[2];

            match status & 0xF0 {
                0x90 if velocity > 0 => {
                    let freq = midi_note_to_freq(note);
                    println!("Note On: {} ({:.2} Hz)", note, freq);

                    let mut sched = scheduler.lock().unwrap();

                    if let Some(record) = sched.graph.arena.get_mut(osc_id) {
                        record.params.params[0] = Param::new(freq);
                    }

                    if let Some(record) = sched.graph.arena.get_mut(env_id) {
                        record.params.params[4] = Param::new(1.0);
                    }
                }
                0x80 | 0x90 => {
                    println!("Note Off: {}", note);

                    let mut sched = scheduler.lock().unwrap();

                    if let Some(record) = sched.graph.arena.get_mut(env_id) {
                        record.params.params[4] = Param::new(0.0);
                    }
                }
                _ => {}
            }
        }
    }, ())?;

    std::mem::forget(_conn);

    Ok(())
}

fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}
```

---

## Next Steps

Congratulations! You've built a working synthesizer. Here are some ideas to extend it:

### Add More Features

1. **Polyphony** - Multiple voices playing simultaneously
2. **LFO** - Modulate filter cutoff or oscillator pitch
3. **Reverb** - Add spatial depth
4. **Multiple Oscillators** - Richer sound
5. **Velocity Sensitivity** - Use MIDI velocity to control volume

### Example: Add an LFO

```rust
use aetherdsp_nodes::lfo::Lfo;

// Add LFO (5 Hz)
let lfo = Box::new(Lfo::new(5.0));
let lfo_id = scheduler.graph.add_node(lfo).unwrap();

// Connect LFO to filter cutoff (slot 1)
scheduler.graph.connect(lfo_id, filter_id, 1);
```

### Example: Add Reverb

```rust
use aetherdsp_nodes::reverb::Reverb;

// Add reverb
let reverb = Box::new(Reverb::new(0.8)); // 80% room size
let reverb_id = scheduler.graph.add_node(reverb).unwrap();

// Insert before output
scheduler.graph.connect(env_id, reverb_id, 0);
scheduler.graph.set_output_node(reverb_id);
```

---

## Troubleshooting

### No Sound

1. Check audio device is selected correctly
2. Verify graph connections with `println!` statements
3. Check envelope is triggered (gate = 1.0)
4. Verify volume isn't muted

### Clicks/Pops

1. Increase buffer size in CPAL config
2. Use parameter smoothing
3. Check for audio dropouts (CPU too high)

### MIDI Not Working

1. Check MIDI device is connected
2. Verify MIDI port selection
3. Test with MIDI monitor tool
4. Check MIDI message parsing

---

## Resources

- [AetherDSP Documentation](https://docs.rs/aetherdsp-core)
- [CPAL Documentation](https://docs.rs/cpal)
- [midir Documentation](https://docs.rs/midir)
- [Next Tutorial: Custom Nodes](custom-nodes.md)

---

**Congratulations!** You've built your first synthesizer with AetherDSP! 🎉
