/**
 * Simple example demonstrating AetherDSP JUCE Bridge usage
 * 
 * Compile:
 *   g++ -o simple_example simple_example.cpp -I../include -L../../../target/release -laetherdsp_juce_bridge
 * 
 * Run:
 *   ./simple_example
 */

#include <iostream>
#include <iomanip>
#include "../include/aetherdsp_juce_bridge.h"

int main() {
    std::cout << "AetherDSP JUCE Bridge Example\n";
    std::cout << "==============================\n\n";
    
    // Get version
    const char* version = aether_version();
    std::cout << "AetherDSP Version: " << version << "\n";
    std::cout << "Available tuning systems: " << aether_tuning_count() << "\n\n";
    
    // Create Ethiopian Tizita tuning
    std::cout << "Creating Ethiopian Tizita tuning...\n";
    AetherTuningTable* tizita = aether_tuning_ethiopian_tizita();
    if (tizita == nullptr) {
        std::cerr << "Failed to create tuning!\n";
        return 1;
    }
    
    // Get frequencies for a few notes
    std::cout << "\nFrequencies in Ethiopian Tizita tuning:\n";
    std::cout << std::fixed << std::setprecision(2);
    
    for (int note = 60; note <= 72; note++) {  // Middle C to C one octave up
        float freq;
        AetherResult result = aether_tuning_get_frequency(tizita, note, &freq);
        
        if (result == Ok) {
            const char* note_names[] = {"C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"};
            std::cout << "  MIDI " << note << " (" << note_names[note % 12] 
                      << "): " << freq << " Hz\n";
        }
    }
    
    // Clean up
    aether_tuning_free(tizita);
    
    std::cout << "\n";
    
    // Try Arabic Maqam Hijaz
    std::cout << "Creating Arabic Maqam Hijaz tuning...\n";
    AetherTuningTable* hijaz = aether_tuning_arabic_hijaz();
    
    std::cout << "\nFrequencies in Arabic Maqam Hijaz (notes 60-67):\n";
    for (int note = 60; note <= 67; note++) {
        float freq;
        aether_tuning_get_frequency(hijaz, note, &freq);
        std::cout << "  MIDI " << note << ": " << freq << " Hz\n";
    }
    
    aether_tuning_free(hijaz);
    
    std::cout << "\n";
    
    // Demonstrate getting all frequencies at once
    std::cout << "Getting all 128 frequencies from Gamelan Slendro Stretched...\n";
    AetherTuningTable* gamelan = aether_tuning_gamelan_slendro_stretched();
    
    float all_frequencies[128];
    aether_tuning_get_all_frequencies(gamelan, all_frequencies);
    
    std::cout << "First 12 frequencies:\n";
    for (int i = 0; i < 12; i++) {
        std::cout << "  Note " << i << ": " << all_frequencies[i] << " Hz\n";
    }
    
    aether_tuning_free(gamelan);
    
    std::cout << "\nExample completed successfully!\n";
    
    return 0;
}
