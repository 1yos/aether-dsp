//! # AetherDSP JUCE Bridge
//!
//! C FFI bridge for integrating AetherDSP's world music tuning systems with JUCE plugins.
//!
//! ## Quick Start
//!
//! ```cpp
//! #include "aetherdsp_juce_bridge.h"
//!
//! // Create Ethiopian Tizita tuning
//! AetherTuningTable* tuning = aether_tuning_ethiopian_tizita();
//!
//! // Get frequency for MIDI note 60 (Middle C)
//! float freq;
//! aether_tuning_get_frequency(tuning, 60, &freq);
//!
//! // Use in your oscillator
//! myOscillator.setFrequency(freq);
//!
//! // Clean up
//! aether_tuning_free(tuning);
//! ```

use aether_midi::tuning::TuningTable;
use std::ffi::c_char;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Opaque handle to a tuning table
#[repr(C)]
pub struct AetherTuningTable {
    _private: [u8; 0],
}

/// Result code for API calls
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AetherResult {
    Ok = 0,
    ErrorNullPointer = 1,
    ErrorInvalidNote = 2,
    ErrorUnknown = 99,
}

// ============================================================================
// INTERNAL CONVERSION
// ============================================================================

unsafe fn tuning_from_handle(handle: *const AetherTuningTable) -> Option<&'static TuningTable> {
    if handle.is_null() {
        return None;
    }
    Some(&*(handle as *const TuningTable))
}

// Standard concert A frequency (A4 = 440 Hz)
const CONCERT_A: f32 = 440.0;

// ============================================================================
// TUNING SYSTEMS
// ============================================================================

/// Create Ethiopian Tizita tuning (pentatonic, characteristic of Ethiopian blues)
#[no_mangle]
pub extern "C" fn aether_tuning_ethiopian_tizita() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::ethiopian_tizita(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Ethiopian Bati tuning (minor pentatonic variant)
#[no_mangle]
pub extern "C" fn aether_tuning_ethiopian_bati() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::ethiopian_bati(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Ethiopian Ambassel tuning (pentatonic with raised 4th)
#[no_mangle]
pub extern "C" fn aether_tuning_ethiopian_ambassel() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::ethiopian_ambassel(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Arabic Maqam Rast tuning (quarter-tone flats on 3rd and 7th)
#[no_mangle]
pub extern "C" fn aether_tuning_arabic_rast() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::arabic_maqam_rast(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Arabic Maqam Bayati tuning (half-flat on 2nd degree)
#[no_mangle]
pub extern "C" fn aether_tuning_arabic_bayati() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::arabic_maqam_bayati(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Arabic Maqam Hijaz tuning (augmented 2nd between 2nd and 3rd degrees)
#[no_mangle]
pub extern "C" fn aether_tuning_arabic_hijaz() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::arabic_maqam_hijaz(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Indian Raga Yaman tuning (raised 4th, Kalyan thaat)
#[no_mangle]
pub extern "C" fn aether_tuning_indian_yaman() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::indian_raga_yaman(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Gamelan Slendro tuning (5-tone Javanese scale)
#[no_mangle]
pub extern "C" fn aether_tuning_gamelan_slendro() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::gamelan_slendro(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Gamelan Slendro Stretched tuning (1210-cent octaves, ethnomusicologically accurate)
#[no_mangle]
pub extern "C" fn aether_tuning_gamelan_slendro_stretched() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::gamelan_slendro_stretched(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Gamelan Pelog tuning (7-tone Javanese scale with unequal intervals)
#[no_mangle]
pub extern "C" fn aether_tuning_gamelan_pelog() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::gamelan_pelog(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Just Intonation (5-limit) tuning (pure thirds and fifths)
#[no_mangle]
pub extern "C" fn aether_tuning_just_intonation() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::just_intonation(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create Just Intonation (7-limit) tuning (septimal intervals for blues and barbershop)
#[no_mangle]
pub extern "C" fn aether_tuning_just_intonation_7_limit() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::just_intonation_7_limit(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Create standard 12-TET tuning (equal temperament, 12 equal divisions of octave)
#[no_mangle]
pub extern "C" fn aether_tuning_equal_temperament() -> *mut AetherTuningTable {
    let tuning = Box::new(TuningTable::equal_temperament(CONCERT_A));
    Box::into_raw(tuning) as *mut AetherTuningTable
}

/// Free a tuning table
///
/// # Safety
/// `tuning` must be a valid handle from an `aether_tuning_*()` function.
/// Do not use the handle after calling this function.
#[no_mangle]
pub unsafe extern "C" fn aether_tuning_free(tuning: *mut AetherTuningTable) {
    if !tuning.is_null() {
        let _ = Box::from_raw(tuning as *mut TuningTable);
    }
}

/// Get the frequency in Hz for a given MIDI note from a tuning table
///
/// # Arguments
/// * `tuning` - Tuning table handle
/// * `midi_note` - MIDI note number (0-127, where 60 = Middle C)
/// * `out_frequency` - Pointer to receive the frequency in Hz
///
/// # Returns
/// AetherResult::Ok on success, error code on failure
///
/// # Safety
/// `tuning` and `out_frequency` must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn aether_tuning_get_frequency(
    tuning: *const AetherTuningTable,
    midi_note: u8,
    out_frequency: *mut f32,
) -> AetherResult {
    if out_frequency.is_null() {
        return AetherResult::ErrorNullPointer;
    }

    let tuning = match tuning_from_handle(tuning) {
        Some(t) => t,
        None => return AetherResult::ErrorNullPointer,
    };

    *out_frequency = tuning.frequency(midi_note);
    AetherResult::Ok
}

/// Get the complete frequency table (128 values, one for each MIDI note)
///
/// # Arguments
/// * `tuning` - Tuning table handle
/// * `out_frequencies` - Pointer to array of 128 floats to receive frequencies
///
/// # Returns
/// AetherResult::Ok on success
///
/// # Safety
/// `out_frequencies` must point to an array of at least 128 floats
#[no_mangle]
pub unsafe extern "C" fn aether_tuning_get_all_frequencies(
    tuning: *const AetherTuningTable,
    out_frequencies: *mut f32,
) -> AetherResult {
    if out_frequencies.is_null() {
        return AetherResult::ErrorNullPointer;
    }

    let tuning = match tuning_from_handle(tuning) {
        Some(t) => t,
        None => return AetherResult::ErrorNullPointer,
    };

    let out_slice = std::slice::from_raw_parts_mut(out_frequencies, 128);
    for (i, freq) in out_slice.iter_mut().enumerate() {
        *freq = tuning.frequency(i as u8);
    }

    AetherResult::Ok
}

// ============================================================================
// VERSION INFO
// ============================================================================

/// Get the AetherDSP version string
///
/// # Returns
/// Null-terminated version string (e.g., "0.1.6")
/// Do not free this pointer - it points to static data
#[no_mangle]
pub extern "C" fn aether_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Get the number of available tuning systems
///
/// # Returns
/// The total count of built-in tuning systems (currently 13)
#[no_mangle]
pub extern "C" fn aether_tuning_count() -> u32 {
    13
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuning_lifecycle() {
        unsafe {
            let tuning = aether_tuning_ethiopian_tizita();
            assert!(!tuning.is_null());

            let mut freq = 0.0f32;
            let result = aether_tuning_get_frequency(tuning, 60, &mut freq);
            assert_eq!(result, AetherResult::Ok);
            assert!(freq > 200.0 && freq < 300.0); // Middle C should be ~261 Hz

            aether_tuning_free(tuning);
        }
    }

    #[test]
    fn test_all_tuning_systems() {
        unsafe {
            let tunings = [
                aether_tuning_ethiopian_tizita(),
                aether_tuning_ethiopian_bati(),
                aether_tuning_ethiopian_ambassel(),
                aether_tuning_arabic_rast(),
                aether_tuning_arabic_bayati(),
                aether_tuning_arabic_hijaz(),
                aether_tuning_indian_yaman(),
                aether_tuning_gamelan_slendro(),
                aether_tuning_gamelan_slendro_stretched(),
                aether_tuning_gamelan_pelog(),
                aether_tuning_just_intonation(),
                aether_tuning_just_intonation_7_limit(),
                aether_tuning_equal_temperament(),
            ];

            for tuning in &tunings {
                assert!(!tuning.is_null());

                let mut freq = 0.0f32;
                let result = aether_tuning_get_frequency(*tuning, 60, &mut freq);
                assert_eq!(result, AetherResult::Ok);
                assert!(freq > 0.0);
            }

            for tuning in tunings {
                aether_tuning_free(tuning);
            }
        }
    }

    #[test]
    fn test_get_all_frequencies() {
        unsafe {
            let tuning = aether_tuning_arabic_hijaz();
            let mut frequencies = [0.0f32; 128];

            let result = aether_tuning_get_all_frequencies(tuning, frequencies.as_mut_ptr());
            assert_eq!(result, AetherResult::Ok);

            // Check that all frequencies are reasonable
            for (i, freq) in frequencies.iter().enumerate() {
                assert!(*freq > 0.0, "Note {} has invalid frequency", i);
            }

            aether_tuning_free(tuning);
        }
    }

    #[test]
    fn test_version() {
        let version = aether_version();
        assert!(!version.is_null());
    }

    #[test]
    fn test_tuning_count() {
        assert_eq!(aether_tuning_count(), 13);
    }
}
