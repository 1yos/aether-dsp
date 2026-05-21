//! Tuning System Comparison
//!
//! This example demonstrates the difference between various tuning systems
//! by showing how the same MIDI notes map to different frequencies.
//!
//! Run with: cargo run --example tuning_comparison -p aetherdsp-midi

use aether_midi::tuning::TuningTable;

fn main() {
    println!("AetherDSP MIDI - Tuning System Comparison");
    println!("==========================================\n");

    // Create tuning tables (all use A4=440Hz)
    let equal_temp = TuningTable::equal_temperament(440.0);
    let tizita = TuningTable::ethiopian_tizita(440.0);
    let rast = TuningTable::arabic_maqam_rast(440.0);
    let yaman = TuningTable::indian_raga_yaman(440.0);

    println!("Comparing tuning systems for one octave (C4-C5):\n");
    println!(
        "{:4} | {:>10} | {:>10} | {:>10} | {:>10} | {:>8}",
        "MIDI", "12-TET", "Tizita", "Rast", "Yaman", "Cents"
    );
    println!("{}", "-".repeat(70));

    // Compare middle C octave (MIDI 60-72)
    for midi_note in 60..=72 {
        let freq_12tet = equal_temp.frequency(midi_note);
        let freq_tizita = tizita.frequency(midi_note);
        let freq_rast = rast.frequency(midi_note);
        let freq_yaman = yaman.frequency(midi_note);

        // Calculate cents deviation from 12-TET
        let cents_tizita: f32 = 1200.0 * (freq_tizita / freq_12tet).log2();
        let cents_rast: f32 = 1200.0 * (freq_rast / freq_12tet).log2();
        let cents_yaman: f32 = 1200.0 * (freq_yaman / freq_12tet).log2();

        let note_name = get_note_name(midi_note);

        println!(
            "{:4} | {:10.2} | {:10.2} | {:10.2} | {:10.2} | {:+8.1}",
            format!("{} ({})", midi_note, note_name),
            freq_12tet,
            freq_tizita,
            freq_rast,
            freq_yaman,
            cents_tizita.max(cents_rast).max(cents_yaman)
        );
    }

    println!("\n📊 Key Observations:");
    println!("  • Ethiopian Tizita: Pentatonic scale with characteristic flat intervals");
    println!("  • Arabic Maqam Rast: Quarter-tone flats on 3rd and 7th degrees");
    println!("  • Indian Raga Yaman: Just intonation with raised 4th (Kalyan thaat)");
    println!("  • Cents: Deviation from 12-TET (100 cents = 1 semitone)");

    println!("\n✓ Try these tuning systems in your music!");
}

fn get_note_name(midi: u8) -> &'static str {
    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    note_names[(midi % 12) as usize]
}
