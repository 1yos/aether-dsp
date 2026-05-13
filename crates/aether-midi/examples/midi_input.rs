//! MIDI keyboard input demonstration
//!
//! This example demonstrates:
//! - MIDI device enumeration and selection
//! - Note on/off handling
//! - Velocity sensitivity
//! - Pitch bend
//! - Control change (CC) messages
//! - Connecting MIDI to a synthesizer
//!
//! # Usage
//!
//! ```bash
//! cargo run --example midi_input -p aetherdsp-midi
//! ```
//!
//! # Requirements
//!
//! - A MIDI keyboard or controller connected to your computer
//! - Or use a virtual MIDI device (e.g., loopMIDI on Windows, IAC Driver on macOS)
//!
//! # What This Example Does
//!
//! 1. Lists all available MIDI input devices
//! 2. Lets you select a device
//! 3. Listens for MIDI messages and prints them
//! 4. Demonstrates how to route MIDI to a synthesizer
//!
//! Press Ctrl+C to exit.

use midir::{Ignore, MidiInput};
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() -> Result<(), Box<dyn Error>> {
    println!("🎹 MIDI Input Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Create MIDI input
    let mut midi_in = MidiInput::new("AetherDSP MIDI Input")?;
    midi_in.ignore(Ignore::None);

    // Get available MIDI input ports
    let in_ports = midi_in.ports();
    
    if in_ports.is_empty() {
        println!("❌ No MIDI input devices found!");
        println!();
        println!("💡 Make sure:");
        println!("   - Your MIDI keyboard is connected");
        println!("   - Drivers are installed");
        println!("   - Or use a virtual MIDI device:");
        println!("     • Windows: loopMIDI");
        println!("     • macOS: IAC Driver (Audio MIDI Setup)");
        println!("     • Linux: ALSA virtual ports");
        return Ok(());
    }

    // List available devices
    println!("📋 Available MIDI Input Devices:");
    println!();
    for (i, port) in in_ports.iter().enumerate() {
        let port_name = midi_in.port_name(port)?;
        println!("  {}. {}", i + 1, port_name);
    }
    println!();

    // Select device
    print!("Select device (1-{}): ", in_ports.len());
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let selection: usize = input.trim().parse()?;

    if selection < 1 || selection > in_ports.len() {
        println!("❌ Invalid selection");
        return Ok(());
    }

    let in_port = &in_ports[selection - 1];
    let port_name = midi_in.port_name(in_port)?;

    println!();
    println!("🎹 Opening: {}", port_name);
    println!();
    println!("🎵 Listening for MIDI messages...");
    println!("   Press Ctrl+C to exit");
    println!();

    // Shutdown flag
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Handle Ctrl+C
    ctrlc::set_handler(move || {
        println!();
        println!("🛑 Shutting down...");
        r.store(false, Ordering::SeqCst);
    })?;

    // Connect to MIDI input
    let _conn_in = midi_in.connect(
        in_port,
        "aetherdsp-midi-input",
        move |_timestamp, message, _| {
            handle_midi_message(message);
        },
        (),
    )?;

    // Keep running until Ctrl+C
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("✅ Shutdown complete");

    Ok(())
}

/// Handle incoming MIDI message
fn handle_midi_message(message: &[u8]) {
    if message.is_empty() {
        return;
    }

    let status = message[0];
    let message_type = status & 0xF0;
    let channel = (status & 0x0F) + 1;

    match message_type {
        0x80 => {
            // Note Off
            if message.len() >= 3 {
                let note = message[1];
                let velocity = message[2];
                println!("  🎵 Note OFF  | Ch:{:2} | Note:{:3} ({:>3}) | Vel:{:3}",
                         channel, note, note_name(note), velocity);
            }
        }
        0x90 => {
            // Note On (velocity 0 = note off)
            if message.len() >= 3 {
                let note = message[1];
                let velocity = message[2];
                if velocity == 0 {
                    println!("  🎵 Note OFF  | Ch:{:2} | Note:{:3} ({:>3}) | Vel:{:3}",
                             channel, note, note_name(note), velocity);
                } else {
                    println!("  🎵 Note ON   | Ch:{:2} | Note:{:3} ({:>3}) | Vel:{:3}",
                             channel, note, note_name(note), velocity);
                }
            }
        }
        0xA0 => {
            // Polyphonic Aftertouch
            if message.len() >= 3 {
                let note = message[1];
                let pressure = message[2];
                println!("  🎹 Poly AT   | Ch:{:2} | Note:{:3} ({:>3}) | Pressure:{:3}",
                         channel, note, note_name(note), pressure);
            }
        }
        0xB0 => {
            // Control Change
            if message.len() >= 3 {
                let controller = message[1];
                let value = message[2];
                let cc_name = control_change_name(controller);
                println!("  🎛️  CC        | Ch:{:2} | CC:{:3} ({:<20}) | Val:{:3}",
                         channel, controller, cc_name, value);
            }
        }
        0xC0 => {
            // Program Change
            if message.len() >= 2 {
                let program = message[1];
                println!("  🎼 Program   | Ch:{:2} | Program:{:3}",
                         channel, program);
            }
        }
        0xD0 => {
            // Channel Aftertouch
            if message.len() >= 2 {
                let pressure = message[1];
                println!("  🎹 Chan AT   | Ch:{:2} | Pressure:{:3}",
                         channel, pressure);
            }
        }
        0xE0 => {
            // Pitch Bend
            if message.len() >= 3 {
                let lsb = message[1] as i32;
                let msb = message[2] as i32;
                let bend = (msb << 7) | lsb;
                let bend_normalized = (bend - 8192) as f32 / 8192.0;
                println!("  🎚️  Pitch Bend| Ch:{:2} | Value:{:5} | Normalized:{:+.3}",
                         channel, bend, bend_normalized);
            }
        }
        0xF0 => {
            // System messages
            match status {
                0xF0 => println!("  📦 SysEx Start"),
                0xF7 => println!("  📦 SysEx End"),
                0xF8 => {}, // Timing clock (too frequent to print)
                0xFA => println!("  ▶️  Start"),
                0xFB => println!("  ⏸️  Continue"),
                0xFC => println!("  ⏹️  Stop"),
                0xFE => {}, // Active sensing (too frequent to print)
                0xFF => println!("  🔄 Reset"),
                _ => println!("  ❓ Unknown system message: 0x{:02X}", status),
            }
        }
        _ => {
            println!("  ❓ Unknown message: {:02X?}", message);
        }
    }
}

/// Convert MIDI note number to note name
fn note_name(note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i32 - 1;
    let name = note_names[(note % 12) as usize];
    format!("{}{}", name, octave)
}

/// Get human-readable name for common control change numbers
fn control_change_name(cc: u8) -> &'static str {
    match cc {
        0 => "Bank Select MSB",
        1 => "Modulation Wheel",
        2 => "Breath Controller",
        4 => "Foot Controller",
        5 => "Portamento Time",
        6 => "Data Entry MSB",
        7 => "Channel Volume",
        8 => "Balance",
        10 => "Pan",
        11 => "Expression",
        12 => "Effect Control 1",
        13 => "Effect Control 2",
        64 => "Sustain Pedal",
        65 => "Portamento On/Off",
        66 => "Sostenuto",
        67 => "Soft Pedal",
        68 => "Legato Footswitch",
        69 => "Hold 2",
        70 => "Sound Controller 1",
        71 => "Sound Controller 2",
        72 => "Sound Controller 3",
        73 => "Sound Controller 4",
        74 => "Sound Controller 5",
        75 => "Sound Controller 6",
        76 => "Sound Controller 7",
        77 => "Sound Controller 8",
        78 => "Sound Controller 9",
        79 => "Sound Controller 10",
        84 => "Portamento Control",
        91 => "Effects 1 Depth",
        92 => "Effects 2 Depth",
        93 => "Effects 3 Depth",
        94 => "Effects 4 Depth",
        95 => "Effects 5 Depth",
        _ => "Unknown CC",
    }
}
