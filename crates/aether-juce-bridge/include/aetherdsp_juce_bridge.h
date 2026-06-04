#ifndef AETHERDSP_JUCE_BRIDGE_H
#define AETHERDSP_JUCE_BRIDGE_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Result code for API calls
 */
typedef enum AetherResult {
  Ok = 0,
  ErrorNullPointer = 1,
  ErrorInvalidNote = 2,
  ErrorUnknown = 99,
} AetherResult;

/**
 * Opaque handle to a tuning table
 */
typedef struct AetherTuningTable {
  uint8_t _private[0];
} AetherTuningTable;

/**
 * Create Ethiopian Tizita major tuning (pentatonic, characteristic of Ethiopian blues)
 */
struct AetherTuningTable *aether_tuning_ethiopian_tizita(void);

/**
 * Create Ethiopian Tizita minor tuning (nostalgic, melancholic pentatonic variant)
 */
struct AetherTuningTable *aether_tuning_ethiopian_tizita_minor(void);

/**
 * Create Ethiopian Bati minor tuning (standard minor pentatonic variant)
 */
struct AetherTuningTable *aether_tuning_ethiopian_bati(void);

/**
 * Create Ethiopian Bati major tuning (bright, uplifting pentatonic variant)
 */
struct AetherTuningTable *aether_tuning_ethiopian_bati_major(void);

/**
 * Create Ethiopian Ambassel tuning (pentatonic with flat 2nd)
 */
struct AetherTuningTable *aether_tuning_ethiopian_ambassel(void);

/**
 * Create Ethiopian Anchihoye tuning (pentatonic without 3rd degree)
 */
struct AetherTuningTable *aether_tuning_ethiopian_anchihoye(void);

/**
 * Create Arabic Maqam Rast tuning (quarter-tone flats on 3rd and 7th)
 */
struct AetherTuningTable *aether_tuning_arabic_rast(void);

/**
 * Create Arabic Maqam Bayati tuning (half-flat on 2nd degree)
 */
struct AetherTuningTable *aether_tuning_arabic_bayati(void);

/**
 * Create Arabic Maqam Hijaz tuning (augmented 2nd between 2nd and 3rd degrees)
 */
struct AetherTuningTable *aether_tuning_arabic_hijaz(void);

/**
 * Create Indian Raga Yaman tuning (raised 4th, Kalyan thaat)
 */
struct AetherTuningTable *aether_tuning_indian_yaman(void);

/**
 * Create Gamelan Slendro tuning (5-tone Javanese scale)
 */
struct AetherTuningTable *aether_tuning_gamelan_slendro(void);

/**
 * Create Gamelan Slendro Stretched tuning (1210-cent octaves, ethnomusicologically accurate)
 */
struct AetherTuningTable *aether_tuning_gamelan_slendro_stretched(void);

/**
 * Create Gamelan Pelog tuning (7-tone Javanese scale with unequal intervals)
 */
struct AetherTuningTable *aether_tuning_gamelan_pelog(void);

/**
 * Create Just Intonation (5-limit) tuning (pure thirds and fifths)
 */
struct AetherTuningTable *aether_tuning_just_intonation(void);

/**
 * Create Just Intonation (7-limit) tuning (septimal intervals for blues and barbershop)
 */
struct AetherTuningTable *aether_tuning_just_intonation_7_limit(void);

/**
 * Create standard 12-TET tuning (equal temperament, 12 equal divisions of octave)
 */
struct AetherTuningTable *aether_tuning_equal_temperament(void);

/**
 * Free a tuning table
 *
 * # Safety
 * `tuning` must be a valid handle from an `aether_tuning_*()` function.
 * Do not use the handle after calling this function.
 */
void aether_tuning_free(struct AetherTuningTable *tuning);

/**
 * Get the frequency in Hz for a given MIDI note from a tuning table
 *
 * # Arguments
 * * `tuning` - Tuning table handle
 * * `midi_note` - MIDI note number (0-127, where 60 = Middle C)
 * * `out_frequency` - Pointer to receive the frequency in Hz
 *
 * # Returns
 * AetherResult::Ok on success, error code on failure
 *
 * # Safety
 * `tuning` and `out_frequency` must be valid pointers
 */
enum AetherResult aether_tuning_get_frequency(const struct AetherTuningTable *tuning,
                                              uint8_t midi_note,
                                              float *out_frequency);

/**
 * Get the complete frequency table (128 values, one for each MIDI note)
 *
 * # Arguments
 * * `tuning` - Tuning table handle
 * * `out_frequencies` - Pointer to array of 128 floats to receive frequencies
 *
 * # Returns
 * AetherResult::Ok on success
 *
 * # Safety
 * `out_frequencies` must point to an array of at least 128 floats
 */
enum AetherResult aether_tuning_get_all_frequencies(const struct AetherTuningTable *tuning,
                                                    float *out_frequencies);

/**
 * Get the AetherDSP version string
 *
 * # Returns
 * Null-terminated version string (e.g., "0.1.6")
 * Do not free this pointer - it points to static data
 */
const char *aether_version(void);

/**
 * Get the number of available tuning systems
 *
 * # Returns
 * The total count of built-in tuning systems (currently 17)
 */
uint32_t aether_tuning_count(void);

#endif  /* AETHERDSP_JUCE_BRIDGE_H */
