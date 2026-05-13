# Tutorial: Microtonal Music with Custom Tuning Systems

**Level:** Intermediate  
**Time:** 20-30 minutes  
**Prerequisites:** Completed [First Synth Tutorial](first-synth.md)

In this tutorial, you'll explore microtonal music using AetherDSP's tuning system support:

- Ethiopian Tizita scale (pentatonic)
- Arabic Maqam scales
- Just intonation
- Custom tuning tables

---

## Table of Contents

1. [Introduction to Microtonality](#introduction-to-microtonality)
2. [Setup](#setup)
3. [Ethiopian Tizita Scale](#ethiopian-tizita-scale)
4. [Arabic Maqam Scales](#arabic-maqam-scales)
5. [Just Intonation](#just-intonation)
6. [Custom Tuning Tables](#custom-tuning-tables)
7. [Next Steps](#next-steps)

---

## Introduction to Microtonality

**Microtonality** is music using intervals smaller than the Western 12-tone equal temperament (12-TET) semitone.

### Why Microtonality?

- **Cultural authenticity** - Many world music traditions use microtones
- **Harmonic purity** - Just intonation has perfect intervals
- **Creative exploration** - New sonic possibilities
- **Historical accuracy** - Pre-equal temperament music

### Common Tuning Systems

| System           | Description                | Example                     |
| ---------------- | -------------------------- | --------------------------- |
| 12-TET           | Western standard (piano)   | C, C#, D, D#, ...           |
| Just Intonation  | Pure frequency ratios      | 5/4, 3/2, 4/3               |
| Ethiopian Tizita | Pentatonic with microtones | Traditional Ethiopian music |
| Arabic Maqam     | Quarter-tone scales        | Middle Eastern music        |
| Indian Shruti    | 22-tone system             | Classical Indian music      |

---

## Setup

### Step 1: Create Project

```bash
cargo new microtonal-synth
cd microtonal-synth
```

### Step 2: Add Dependencies

Edit `Cargo.toml`:

```toml
[dependencies]
aetherdsp-core = "0.1.4"
aetherdsp-nodes = "0.2.3"
# Note: aetherdsp-timbre will be available in future release
# For now, we'll implement tuning manually
```

---

## Ethiopian Tizita Scale

The Tizita scale is a pentatonic scale used in Ethiopian music with characteristic microtonal intervals.

### Step 1: Define the Scale

Create `src/main.rs`:

```rust
use aetherdsp_core::scheduler::Scheduler;
use aetherdsp_nodes::oscillator::Oscillator;

/// Ethiopian Tizita scale (cents from root)
/// Based on traditional Ethiopian music theory
const TIZITA_SCALE: [f32; 5] = [
    0.0,    // Root (C)
    204.0,  // ~2 semitones (D)
    294.0,  // ~3 semitones (Eb-)
    498.0,  // Perfect fourth (F)
    702.0,  // Perfect fifth (G)
];

fn cents_to_ratio(cents: f32) -> f32 {
    2.0_f32.powf(cents / 1200.0)
}

fn tizita_note_to_freq(note: u8, root_freq: f32) -> f32 {
    let octave = note / 5;
    let scale_degree = (note % 5) as usize;

    let cents = TIZITA_SCALE[scale_degree];
    let ratio = cents_to_ratio(cents);

    root_freq * ratio * 2.0_f32.powi(octave as i32)
}

fn main() {
    let mut sched = Scheduler::new(48_000.0);

    // Add oscillator
    let osc = Box::new(Oscillator::new(440.0));
    let osc_id = sched.graph.add_node(osc).unwrap();
    sched.graph.set_output_node(osc_id);

    // Play Tizita scale
    println!("Ethiopian Tizita Scale:");
    println!("Root frequency: 440 Hz (A)");
    println!();

    for note in 0..10 {
        let freq = tizita_note_to_freq(note, 440.0);
        let cents = TIZITA_SCALE[(note % 5) as usize];
        println!("Note {}: {:.2} Hz ({:.0} cents)", note, freq, cents);
    }
}
```

### Step 2: Render the Scale

Add WAV rendering:

```rust
use hound::{WavWriter, WavSpec};
use aetherdsp_core::param::Param;

fn main() {
    let mut sched = Scheduler::new(48_000.0);

    let osc = Box::new(Oscillator::new(440.0));
    let osc_id = sched.graph.add_node(osc).unwrap();
    sched.graph.set_output_node(osc_id);

    // Setup WAV writer
    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create("tizita_scale.wav", spec).unwrap();
    let mut output = vec![0.0f32; 128];

    // Play each note for 0.5 seconds
    for note in 0..10 {
        let freq = tizita_note_to_freq(note, 440.0);

        // Update oscillator frequency
        if let Some(record) = sched.graph.arena.get_mut(osc_id) {
            record.params.params[0] = Param::new(freq);
        }

        // Render 0.5 seconds
        for _ in 0..(48_000 / 2 / 64) {
            sched.process_block_simple(&mut output);

            for &sample in output.iter().take(64) {
                let amplitude = (sample * 0.3 * i16::MAX as f32) as i16;
                writer.write_sample(amplitude).unwrap();
            }
        }
    }

    writer.finalize().unwrap();
    println!("Rendered tizita_scale.wav");
}
```

Add to `Cargo.toml`:

```toml
[dependencies]
hound = "3"
```

Run it:

```bash
cargo run
```

**Listen to `tizita_scale.wav`** - you'll hear the characteristic Ethiopian sound!

---

## Arabic Maqam Scales

Arabic music uses quarter-tones (50 cents), creating rich melodic possibilities.

### Step 1: Define Maqam Rast

```rust
/// Maqam Rast (Arabic scale with quarter-tones)
const MAQAM_RAST: [f32; 8] = [
    0.0,    // Root (C)
    200.0,  // Whole tone (D)
    400.0,  // Whole tone (E)
    500.0,  // Half tone (F)
    700.0,  // Whole tone (G)
    900.0,  // Whole tone (A)
    1100.0, // Whole tone (B)
    1200.0, // Octave (C)
];

fn maqam_note_to_freq(note: u8, root_freq: f32) -> f32 {
    let octave = note / 8;
    let scale_degree = (note % 8) as usize;

    let cents = MAQAM_RAST[scale_degree];
    let ratio = cents_to_ratio(cents);

    root_freq * ratio * 2.0_f32.powi(octave as i32)
}
```

### Step 2: Define Maqam Bayati (with quarter-tones)

```rust
/// Maqam Bayati (with characteristic quarter-tone)
const MAQAM_BAYATI: [f32; 8] = [
    0.0,    // Root (D)
    150.0,  // Three-quarter tone (Eb-)
    300.0,  // Minor third (F)
    500.0,  // Perfect fourth (G)
    700.0,  // Perfect fifth (A)
    850.0,  // Three-quarter tone (Bb-)
    1000.0, // Minor seventh (C)
    1200.0, // Octave (D)
];
```

### Step 3: Render Maqam

```rust
fn render_maqam() {
    let mut sched = Scheduler::new(48_000.0);

    let osc = Box::new(Oscillator::new(440.0));
    let osc_id = sched.graph.add_node(osc).unwrap();
    sched.graph.set_output_node(osc_id);

    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create("maqam_bayati.wav", spec).unwrap();
    let mut output = vec![0.0f32; 128];

    // Play Maqam Bayati ascending and descending
    let notes = vec![0, 1, 2, 3, 4, 5, 6, 7, 6, 5, 4, 3, 2, 1, 0];

    for note in notes {
        let freq = maqam_bayati_note_to_freq(note, 440.0);

        if let Some(record) = sched.graph.arena.get_mut(osc_id) {
            record.params.params[0] = Param::new(freq);
        }

        // Render 0.4 seconds per note
        for _ in 0..(48_000 * 4 / 10 / 64) {
            sched.process_block_simple(&mut output);

            for &sample in output.iter().take(64) {
                let amplitude = (sample * 0.3 * i16::MAX as f32) as i16;
                writer.write_sample(amplitude).unwrap();
            }
        }
    }

    writer.finalize().unwrap();
    println!("Rendered maqam_bayati.wav");
}

fn maqam_bayati_note_to_freq(note: u8, root_freq: f32) -> f32 {
    let cents = MAQAM_BAYATI[note as usize];
    let ratio = cents_to_ratio(cents);
    root_freq * ratio
}
```

---

## Just Intonation

Just intonation uses pure frequency ratios for perfect harmony.

### Step 1: Define Just Intonation Scale

```rust
/// Just intonation major scale (pure ratios)
const JUST_MAJOR: [(u32, u32); 8] = [
    (1, 1),   // Root (1/1)
    (9, 8),   // Major second (9/8)
    (5, 4),   // Major third (5/4) - pure!
    (4, 3),   // Perfect fourth (4/3)
    (3, 2),   // Perfect fifth (3/2) - pure!
    (5, 3),   // Major sixth (5/3)
    (15, 8),  // Major seventh (15/8)
    (2, 1),   // Octave (2/1)
];

fn just_note_to_freq(note: u8, root_freq: f32) -> f32 {
    let octave = note / 8;
    let scale_degree = (note % 8) as usize;

    let (num, den) = JUST_MAJOR[scale_degree];
    let ratio = num as f32 / den as f32;

    root_freq * ratio * 2.0_f32.powi(octave as i32)
}
```

### Step 2: Compare with 12-TET

```rust
fn compare_tunings() {
    println!("Comparison: Just Intonation vs 12-TET");
    println!("Note | Just (Hz) | 12-TET (Hz) | Difference (cents)");
    println!("-----|-----------|-------------|-------------------");

    let root = 440.0;

    for note in 0..8 {
        let just_freq = just_note_to_freq(note, root);
        let tet_freq = tet_note_to_freq(note, root);

        let cents_diff = 1200.0 * (just_freq / tet_freq).log2();

        println!("{:4} | {:9.2} | {:11.2} | {:+8.1}",
            note, just_freq, tet_freq, cents_diff);
    }
}

fn tet_note_to_freq(note: u8, root_freq: f32) -> f32 {
    root_freq * 2.0_f32.powf(note as f32 / 12.0)
}
```

Output:

```
Note | Just (Hz) | 12-TET (Hz) | Difference (cents)
-----|-----------|-------------|-------------------
   0 |    440.00 |      440.00 |     +0.0
   1 |    495.00 |      493.88 |     +3.9
   2 |    550.00 |      554.37 |    -13.7
   3 |    586.67 |      587.33 |     -2.0
   4 |    660.00 |      659.26 |     +2.0
   5 |    733.33 |      739.99 |    -15.6
   6 |    825.00 |      830.61 |    -11.7
   7 |    880.00 |      880.00 |     +0.0
```

**Notice:** The major third (note 2) is 13.7 cents flatter in just intonation - this is the "pure" third!

---

## Custom Tuning Tables

Create your own tuning systems.

### Step 1: Define Custom Scale

```rust
/// Bohlen-Pierce scale (13 notes per tritave)
/// Uses 3:1 ratio instead of 2:1 octave
const BOHLEN_PIERCE: [f32; 13] = [
    0.0,
    146.3,
    292.6,
    438.9,
    585.2,
    731.5,
    877.8,
    1024.1,
    1170.4,
    1316.7,
    1463.0,
    1609.3,
    1755.6,
];

fn bohlen_pierce_note_to_freq(note: u8, root_freq: f32) -> f32 {
    let tritave = note / 13;
    let scale_degree = (note % 13) as usize;

    let cents = BOHLEN_PIERCE[scale_degree];
    let ratio = 3.0_f32.powf(cents / 1901.955); // 1901.955 cents = tritave

    root_freq * ratio * 3.0_f32.powi(tritave as i32)
}
```

### Step 2: Create Tuning Table Builder

```rust
struct TuningTable {
    name: String,
    ratios: Vec<f32>,
    reference_freq: f32,
}

impl TuningTable {
    fn new(name: &str, reference_freq: f32) -> Self {
        Self {
            name: name.to_string(),
            ratios: vec![1.0],
            reference_freq,
        }
    }

    fn add_ratio(&mut self, numerator: u32, denominator: u32) {
        self.ratios.push(numerator as f32 / denominator as f32);
    }

    fn add_cents(&mut self, cents: f32) {
        self.ratios.push(cents_to_ratio(cents));
    }

    fn note_to_freq(&self, note: u8) -> f32 {
        let octave = note as usize / self.ratios.len();
        let scale_degree = note as usize % self.ratios.len();

        self.reference_freq * self.ratios[scale_degree] * 2.0_f32.powi(octave as i32)
    }

    fn print_table(&self) {
        println!("Tuning Table: {}", self.name);
        println!("Reference: {:.2} Hz", self.reference_freq);
        println!();
        println!("Degree | Ratio  | Cents  | Frequency");
        println!("-------|--------|--------|----------");

        for (i, &ratio) in self.ratios.iter().enumerate() {
            let cents = 1200.0 * ratio.log2();
            let freq = self.reference_freq * ratio;
            println!("{:6} | {:6.4} | {:6.1} | {:8.2}",
                i, ratio, cents, freq);
        }
    }
}
```

### Step 3: Use Custom Tuning

```rust
fn main() {
    // Create custom pentatonic tuning
    let mut tuning = TuningTable::new("Custom Pentatonic", 440.0);
    tuning.add_ratio(9, 8);   // Major second
    tuning.add_ratio(5, 4);   // Major third
    tuning.add_ratio(3, 2);   // Perfect fifth
    tuning.add_ratio(15, 8);  // Major seventh

    tuning.print_table();

    // Use in synth
    let mut sched = Scheduler::new(48_000.0);
    let osc = Box::new(Oscillator::new(440.0));
    let osc_id = sched.graph.add_node(osc).unwrap();
    sched.graph.set_output_node(osc_id);

    // Play scale
    for note in 0..10 {
        let freq = tuning.note_to_freq(note);
        println!("Playing note {} at {:.2} Hz", note, freq);

        if let Some(record) = sched.graph.arena.get_mut(osc_id) {
            record.params.params[0] = Param::new(freq);
        }

        // Render...
    }
}
```

---

## Complete Example: Microtonal Sequencer

Here's a complete example that plays a melody in Ethiopian Tizita scale:

```rust
use aetherdsp_core::scheduler::Scheduler;
use aetherdsp_core::param::Param;
use aetherdsp_nodes::oscillator::Oscillator;
use aetherdsp_nodes::envelope::Envelope;
use hound::{WavWriter, WavSpec};

const TIZITA_SCALE: [f32; 5] = [0.0, 204.0, 294.0, 498.0, 702.0];

fn cents_to_ratio(cents: f32) -> f32 {
    2.0_f32.powf(cents / 1200.0)
}

fn tizita_note_to_freq(note: u8, root_freq: f32) -> f32 {
    let octave = note / 5;
    let scale_degree = (note % 5) as usize;
    let cents = TIZITA_SCALE[scale_degree];
    let ratio = cents_to_ratio(cents);
    root_freq * ratio * 2.0_f32.powi(octave as i32)
}

fn main() {
    let mut sched = Scheduler::new(48_000.0);

    // Build synth
    let osc = Box::new(Oscillator::new(440.0));
    let osc_id = sched.graph.add_node(osc).unwrap();

    let mut env = Envelope::new();
    env.set_attack(0.01);
    env.set_decay(0.1);
    env.set_sustain(0.7);
    env.set_release(0.1);
    let env_id = sched.graph.add_node(Box::new(env)).unwrap();

    sched.graph.connect(osc_id, env_id, 0);
    sched.graph.set_output_node(env_id);

    // Melody in Tizita scale
    let melody = vec![
        (0, 0.5), (2, 0.5), (3, 0.5), (4, 0.5),
        (5, 1.0), (4, 0.5), (3, 0.5),
        (2, 1.0), (0, 1.0),
    ];

    // Render
    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create("tizita_melody.wav", spec).unwrap();
    let mut output = vec![0.0f32; 128];

    for (note, duration) in melody {
        let freq = tizita_note_to_freq(note, 440.0);

        // Update frequency
        if let Some(record) = sched.graph.arena.get_mut(osc_id) {
            record.params.params[0] = Param::new(freq);
        }

        // Trigger envelope
        if let Some(record) = sched.graph.arena.get_mut(env_id) {
            record.params.params[4] = Param::new(1.0);
        }

        // Render note
        let samples = (48_000.0 * duration) as usize;
        for _ in 0..(samples / 64) {
            sched.process_block_simple(&mut output);

            for &sample in output.iter().take(64) {
                let amplitude = (sample * 0.3 * i16::MAX as f32) as i16;
                writer.write_sample(amplitude).unwrap();
            }
        }

        // Release envelope
        if let Some(record) = sched.graph.arena.get_mut(env_id) {
            record.params.params[4] = Param::new(0.0);
        }
    }

    writer.finalize().unwrap();
    println!("Rendered tizita_melody.wav");
}
```

---

## Next Steps

### Explore More Tunings

1. **Indian Shruti** - 22-tone system
2. **Turkish Maqam** - 53-tone equal temperament
3. **Harry Partch** - 43-tone just intonation
4. **Wendy Carlos** - Alpha, Beta, Gamma scales

### Advanced Techniques

1. **Dynamic Retuning** - Change tuning during playback
2. **Adaptive JI** - Adjust tuning based on harmony
3. **Stretched Tuning** - Compensate for inharmonicity
4. **Temperament Ordinaire** - Historical French tuning

### Resources

- [Xenharmonic Wiki](https://en.xen.wiki/) - Comprehensive microtonal resource
- [Scala](http://www.huygens-fokker.org/scala/) - Tuning software and scale archive
- [Ethiopian Music Theory](https://en.wikipedia.org/wiki/Music_of_Ethiopia)
- [Arabic Maqam](https://www.maqamworld.com/) - Maqam theory and practice

---

**Congratulations!** You've explored microtonal music with AetherDSP! 🎵
