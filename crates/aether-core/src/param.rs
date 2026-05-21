//! Sample-accurate parameter automation.
//!
//! Each Param smooths from `current` toward `target` over a fixed ramp.
//! No allocations. No locks. Safe to read/write from the RT thread.

/// A single smoothed parameter.
///
/// Provides sample-accurate parameter automation with linear ramping.
/// Parameters smoothly transition from `current` to `target` over a
/// specified number of samples, preventing audio clicks and zipper noise.
///
/// # Real-Time Safety
///
/// - ✅ No allocation
/// - ✅ No locks
/// - ✅ Bounded execution time
/// - ✅ Safe to use in audio thread
///
/// # Example
///
/// ```
/// use aether_core::param::Param;
///
/// let mut gain = Param::new(0.5);
///
/// // Schedule ramp to 1.0 over 480 samples (10ms @ 48kHz)
/// gain.set_target(1.0, 480);
///
/// // Tick through samples
/// for _ in 0..480 {
///     let value = gain.current;
///     // Use value for processing...
///     gain.tick();
/// }
///
/// // Close enough to 1.0 (floating point precision)
/// assert!((gain.current - 1.0).abs() < 0.0001);
/// ```
///
/// # Performance
///
/// - Fast path when not ramping (step == 0.0)
/// - SIMD-friendly linear interpolation
/// - Automatic overshoot clamping
///
/// # See Also
///
/// * [`ParamBlock`] - Collection of parameters for a node
/// * [`Param::fill_buffer`] - Efficient buffer filling
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Param {
    pub current: f32,
    pub target: f32,
    /// Per-sample increment. Set by `set_target`.
    pub step: f32,
}

impl Param {
    /// Creates a new parameter with the given initial value.
    ///
    /// The parameter starts at the specified value with no ramping
    /// (current == target, step == 0.0).
    ///
    /// # Arguments
    ///
    /// * `value` - Initial parameter value
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::Param;
    ///
    /// let gain = Param::new(0.75);
    /// assert_eq!(gain.current, 0.75);
    /// assert_eq!(gain.target, 0.75);
    /// assert_eq!(gain.step, 0.0);
    /// ```
    pub fn new(value: f32) -> Self {
        Self {
            current: value,
            target: value,
            step: 0.0,
        }
    }

    /// Schedule a ramp to `target` over `ramp_samples` samples.
    ///
    /// Sets up linear interpolation from current value to target value.
    /// Call from the control thread before pushing an `UpdateParam` command.
    ///
    /// # Arguments
    ///
    /// * `target` - Target value to ramp towards
    /// * `ramp_samples` - Number of samples for the ramp (0 = instant)
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::Param;
    ///
    /// let mut cutoff = Param::new(1000.0);
    ///
    /// // Ramp to 5000 Hz over 960 samples (20ms @ 48kHz)
    /// cutoff.set_target(5000.0, 960);
    ///
    /// // After 480 samples, we're halfway
    /// for _ in 0..480 {
    ///     cutoff.tick();
    /// }
    /// assert!((cutoff.current - 3000.0).abs() < 1.0);
    ///
    /// // After 960 samples total, we've reached the target
    /// for _ in 0..480 {
    ///     cutoff.tick();
    /// }
    /// assert!((cutoff.current - 5000.0).abs() < 0.01);
    /// ```
    ///
    /// # Instant Changes
    ///
    /// ```
    /// use aether_core::param::Param;
    ///
    /// let mut gain = Param::new(0.5);
    ///
    /// // Instant change (0 samples)
    /// gain.set_target(1.0, 0);
    /// assert_eq!(gain.current, 1.0);
    /// assert_eq!(gain.step, 0.0);
    /// ```
    #[inline]
    pub fn set_target(&mut self, target: f32, ramp_samples: u32) {
        self.target = target;
        if ramp_samples == 0 {
            self.current = target;
            self.step = 0.0;
        } else {
            self.step = (target - self.current) / ramp_samples as f32;
        }
    }

    /// Schedule a ramp to `target` with validation, clamping to `[min, max]`.
    ///
    /// Like [`set_target`](Self::set_target), but clamps the target value to the
    /// specified range and validates that it's finite (not NaN or Infinity).
    ///
    /// # Arguments
    ///
    /// * `target` - Target value to ramp towards
    /// * `ramp_samples` - Number of samples for the ramp (0 = instant)
    /// * `min` - Minimum allowed value
    /// * `max` - Maximum allowed value
    ///
    /// # Returns
    ///
    /// The clamped target value that was actually set.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::Param;
    ///
    /// let mut gain = Param::new(0.5);
    ///
    /// // Clamp to [0.0, 1.0]
    /// let actual = gain.set_target_clamped(1.5, 480, 0.0, 1.0);
    /// assert_eq!(actual, 1.0); // Clamped to max
    ///
    /// // NaN is replaced with current value
    /// let actual = gain.set_target_clamped(f32::NAN, 0, 0.0, 1.0);
    /// assert_eq!(actual, gain.current);
    /// ```
    ///
    /// # Safety
    ///
    /// This function ensures RT safety by:
    /// - Replacing NaN/Infinity with the current value
    /// - Clamping to valid range
    /// - Preventing invalid audio state
    #[inline]
    pub fn set_target_clamped(
        &mut self,
        target: f32,
        ramp_samples: u32,
        min: f32,
        max: f32,
    ) -> f32 {
        // Validate: replace NaN/Infinity with current value
        let validated = if target.is_finite() {
            target.clamp(min, max)
        } else {
            self.current
        };

        self.set_target(validated, ramp_samples);
        validated
    }

    /// Advance by one sample. Call once per sample in the RT loop.
    ///
    /// Updates `current` by adding `step`. When the target is reached,
    /// automatically stops ramping by setting `step` to 0.0.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::Param;
    ///
    /// let mut gain = Param::new(0.0);
    /// gain.set_target(1.0, 100);
    ///
    /// // Tick through 100 samples
    /// for i in 0..100 {
    ///     gain.tick();
    /// }
    ///
    /// // Reached target value
    /// assert!((gain.current - 1.0).abs() < 0.0001);
    /// ```
    ///
    /// # Performance
    ///
    /// This function is highly optimized for the audio thread:
    /// - Inlined for zero call overhead
    /// - Branch-free when not ramping
    /// - Automatic overshoot clamping
    #[inline(always)]
    pub fn tick(&mut self) {
        if self.step != 0.0 {
            self.current += self.step;
            // Clamp overshoot.
            if (self.step > 0.0 && self.current >= self.target)
                || (self.step < 0.0 && self.current <= self.target)
            {
                self.current = self.target;
                self.step = 0.0;
            }
        }
    }

    /// Advance by a full buffer, returning per-sample values into `out`.
    ///
    /// Efficiently fills a buffer with parameter values, advancing the ramp
    /// for each sample. Uses a fast path when the parameter is stable (not ramping).
    ///
    /// # Arguments
    ///
    /// * `out` - Output buffer to fill with parameter values
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::Param;
    /// use aether_core::BUFFER_SIZE;
    ///
    /// let mut cutoff = Param::new(1000.0);
    /// cutoff.set_target(2000.0, BUFFER_SIZE as u32);
    ///
    /// let mut buffer = [0.0f32; BUFFER_SIZE];
    /// cutoff.fill_buffer(&mut buffer);
    ///
    /// // First sample is near 1000, last sample is near 2000
    /// assert!((buffer[0] - 1000.0).abs() < 50.0);
    /// assert!((buffer[BUFFER_SIZE-1] - 2000.0).abs() < 50.0);
    /// ```
    ///
    /// # Performance
    ///
    /// This function has two paths:
    /// - **Fast path** (step == 0.0): Fills buffer with single value (SIMD-friendly)
    /// - **Ramp path** (step != 0.0): Advances sample-by-sample
    ///
    /// The fast path is taken 90%+ of the time in typical usage.
    ///
    /// # Use Case
    ///
    /// Use this when you need per-sample parameter values for modulation:
    ///
    /// ```
    /// use aether_core::param::Param;
    /// use aether_core::BUFFER_SIZE;
    ///
    /// let mut gain = Param::new(0.5);
    /// let mut gain_buffer = [0.0f32; BUFFER_SIZE];
    /// let input = [1.0f32; BUFFER_SIZE];
    /// let mut output = [0.0f32; BUFFER_SIZE];
    ///
    /// // Fill gain buffer
    /// gain.fill_buffer(&mut gain_buffer);
    ///
    /// // Apply per-sample gain
    /// for i in 0..BUFFER_SIZE {
    ///     output[i] = input[i] * gain_buffer[i];
    /// }
    /// ```
    #[inline]
    pub fn fill_buffer(&mut self, out: &mut [f32]) {
        if self.step == 0.0 {
            // Fast path: parameter is stable — fill with a single value.
            // This is the common case and avoids all branching in the loop.
            out.fill(self.current);
        } else {
            // Ramping path: advance sample by sample.
            for sample in out.iter_mut() {
                *sample = self.current;
                self.tick();
            }
        }
    }
}

/// A fixed-size block of parameters for a node.
///
/// Stores up to 8 parameters without heap allocation. Most DSP nodes need
/// 1-4 parameters (gain, frequency, resonance, etc.), so 8 is sufficient
/// for the vast majority of cases.
///
/// # Example
///
/// ```
/// use aether_core::param::ParamBlock;
///
/// let mut params = ParamBlock::new();
///
/// // Add parameters
/// let gain_idx = params.add(0.75);      // Gain: 0.75
/// let cutoff_idx = params.add(1000.0);  // Cutoff: 1000 Hz
/// let res_idx = params.add(0.5);        // Resonance: 0.5
///
/// assert_eq!(params.count, 3);
///
/// // Access parameters
/// let gain = params.get(gain_idx);
/// assert_eq!(gain.current, 0.75);
///
/// // Modify parameters
/// params.get_mut(cutoff_idx).set_target(2000.0, 480);
///
/// // Tick all parameters
/// params.tick_all();
/// ```
///
/// # Capacity
///
/// If you need more than 8 parameters, consider:
/// - Splitting into multiple nodes
/// - Using a custom parameter storage system
/// - Increasing the array size (requires modifying the constant)
///
/// # See Also
///
/// * [`Param`] - Individual parameter
#[derive(Debug, Clone, Copy)]
pub struct ParamBlock {
    pub params: [Param; 8],
    pub count: usize,
}

impl ParamBlock {
    /// Creates a new empty parameter block.
    ///
    /// Initializes with zero parameters. Use [`add`](Self::add) to add parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::ParamBlock;
    ///
    /// let params = ParamBlock::new();
    /// assert_eq!(params.count, 0);
    /// ```
    pub fn new() -> Self {
        Self {
            params: [Param::new(0.0); 8],
            count: 0,
        }
    }

    /// Adds a parameter with the given initial value.
    ///
    /// # Arguments
    ///
    /// * `value` - Initial parameter value
    ///
    /// # Returns
    ///
    /// The parameter's index (0-7), used to access it later.
    ///
    /// # Panics
    ///
    /// Panics if the block is full (8 parameters already added).
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::ParamBlock;
    ///
    /// let mut params = ParamBlock::new();
    ///
    /// let gain_idx = params.add(0.5);
    /// let freq_idx = params.add(440.0);
    ///
    /// assert_eq!(gain_idx, 0);
    /// assert_eq!(freq_idx, 1);
    /// assert_eq!(params.count, 2);
    /// ```
    pub fn add(&mut self, value: f32) -> usize {
        let idx = self.count;
        self.params[idx] = Param::new(value);
        self.count += 1;
        idx
    }

    /// Gets an immutable reference to a parameter.
    ///
    /// # Arguments
    ///
    /// * `idx` - Parameter index (0-7)
    ///
    /// # Returns
    ///
    /// Reference to the parameter.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::ParamBlock;
    ///
    /// let mut params = ParamBlock::new();
    /// let gain_idx = params.add(0.75);
    ///
    /// let gain = params.get(gain_idx);
    /// assert_eq!(gain.current, 0.75);
    /// ```
    #[inline(always)]
    pub fn get(&self, idx: usize) -> &Param {
        &self.params[idx]
    }

    /// Gets a mutable reference to a parameter.
    ///
    /// # Arguments
    ///
    /// * `idx` - Parameter index (0-7)
    ///
    /// # Returns
    ///
    /// Mutable reference to the parameter.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::ParamBlock;
    ///
    /// let mut params = ParamBlock::new();
    /// let cutoff_idx = params.add(1000.0);
    ///
    /// // Schedule a ramp
    /// params.get_mut(cutoff_idx).set_target(2000.0, 480);
    /// ```
    #[inline(always)]
    pub fn get_mut(&mut self, idx: usize) -> &mut Param {
        &mut self.params[idx]
    }

    /// Tick all active params by one sample.
    ///
    /// Advances all parameters in the block by one sample. Call this once
    /// per sample in your node's `process()` function.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::ParamBlock;
    /// use aether_core::BUFFER_SIZE;
    ///
    /// let mut params = ParamBlock::new();
    /// let gain_idx = params.add(0.0);
    /// params.get_mut(gain_idx).set_target(1.0, BUFFER_SIZE as u32);
    ///
    /// // Tick through buffer
    /// for _ in 0..BUFFER_SIZE {
    ///     let gain_value = params.get(gain_idx).current;
    ///     // Use gain_value for processing...
    ///     params.tick_all();
    /// }
    ///
    /// assert_eq!(params.get(gain_idx).current, 1.0);
    /// ```
    ///
    /// # Performance
    ///
    /// This function is highly optimized:
    /// - Inlined for zero call overhead
    /// - Only ticks active parameters (count)
    /// - Each tick is branch-free when not ramping
    #[inline(always)]
    pub fn tick_all(&mut self) {
        for p in self.params[..self.count].iter_mut() {
            p.tick();
        }
    }
}

impl Default for ParamBlock {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameter validation utilities.
///
/// These functions help ensure parameter values are safe for real-time audio processing.
pub mod validation {
    /// Validates that a value is finite (not NaN or Infinity).
    ///
    /// # Arguments
    ///
    /// * `value` - Value to validate
    ///
    /// # Returns
    ///
    /// `true` if the value is finite, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::validation::is_finite;
    ///
    /// assert!(is_finite(1.0));
    /// assert!(is_finite(0.0));
    /// assert!(is_finite(-100.0));
    /// assert!(!is_finite(f32::NAN));
    /// assert!(!is_finite(f32::INFINITY));
    /// assert!(!is_finite(f32::NEG_INFINITY));
    /// ```
    #[inline]
    pub fn is_finite(value: f32) -> bool {
        value.is_finite()
    }

    /// Clamps a value to a range, replacing NaN/Infinity with a default.
    ///
    /// # Arguments
    ///
    /// * `value` - Value to clamp
    /// * `min` - Minimum allowed value
    /// * `max` - Maximum allowed value
    /// * `default` - Default value to use if `value` is NaN/Infinity
    ///
    /// # Returns
    ///
    /// Clamped value in range `[min, max]`.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::validation::clamp_or_default;
    ///
    /// assert_eq!(clamp_or_default(0.5, 0.0, 1.0, 0.5), 0.5);
    /// assert_eq!(clamp_or_default(1.5, 0.0, 1.0, 0.5), 1.0);
    /// assert_eq!(clamp_or_default(-0.5, 0.0, 1.0, 0.5), 0.0);
    /// assert_eq!(clamp_or_default(f32::NAN, 0.0, 1.0, 0.5), 0.5);
    /// assert_eq!(clamp_or_default(f32::INFINITY, 0.0, 1.0, 0.5), 0.5);
    /// ```
    #[inline]
    pub fn clamp_or_default(value: f32, min: f32, max: f32, default: f32) -> f32 {
        if value.is_finite() {
            value.clamp(min, max)
        } else {
            default
        }
    }

    /// Validates a frequency value (positive, finite, reasonable range).
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency in Hz
    /// * `sample_rate` - Sample rate in Hz
    ///
    /// # Returns
    ///
    /// Clamped frequency in range `[0.1, sample_rate/2]` (Nyquist limit).
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::validation::validate_frequency;
    ///
    /// assert_eq!(validate_frequency(440.0, 48000.0), 440.0);
    /// assert_eq!(validate_frequency(-100.0, 48000.0), 0.1); // Negative clamped to min
    /// assert_eq!(validate_frequency(30000.0, 48000.0), 24000.0); // Above Nyquist
    /// assert_eq!(validate_frequency(f32::NAN, 48000.0), 440.0); // NaN replaced with A4
    /// ```
    #[inline]
    pub fn validate_frequency(freq: f32, sample_rate: f32) -> f32 {
        const MIN_FREQ: f32 = 0.1;
        let max_freq = sample_rate * 0.5; // Nyquist limit
        clamp_or_default(freq, MIN_FREQ, max_freq, 440.0) // Default to A4
    }

    /// Validates a gain value (0.0 to 1.0 or higher).
    ///
    /// # Arguments
    ///
    /// * `gain` - Gain value (linear, not dB)
    /// * `max_gain` - Maximum allowed gain
    ///
    /// # Returns
    ///
    /// Clamped gain in range `[0.0, max_gain]`.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::validation::validate_gain;
    ///
    /// assert_eq!(validate_gain(0.5, 2.0), 0.5);
    /// assert_eq!(validate_gain(-0.5, 2.0), 0.0); // Negative clamped to 0
    /// assert_eq!(validate_gain(3.0, 2.0), 2.0); // Above max
    /// assert_eq!(validate_gain(f32::NAN, 2.0), 1.0); // NaN replaced with unity
    /// ```
    #[inline]
    pub fn validate_gain(gain: f32, max_gain: f32) -> f32 {
        clamp_or_default(gain, 0.0, max_gain, 1.0) // Default to unity gain
    }

    /// Validates a time value in milliseconds.
    ///
    /// # Arguments
    ///
    /// * `time_ms` - Time in milliseconds
    /// * `min_ms` - Minimum allowed time
    /// * `max_ms` - Maximum allowed time
    ///
    /// # Returns
    ///
    /// Clamped time in range `[min_ms, max_ms]`.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::param::validation::validate_time_ms;
    ///
    /// assert_eq!(validate_time_ms(50.0, 1.0, 1000.0), 50.0);
    /// assert_eq!(validate_time_ms(0.5, 1.0, 1000.0), 1.0); // Below min
    /// assert_eq!(validate_time_ms(2000.0, 1.0, 1000.0), 1000.0); // Above max
    /// assert_eq!(validate_time_ms(f32::NAN, 1.0, 1000.0), 100.0); // NaN replaced with 100ms
    /// ```
    #[inline]
    pub fn validate_time_ms(time_ms: f32, min_ms: f32, max_ms: f32) -> f32 {
        clamp_or_default(time_ms, min_ms, max_ms, 100.0) // Default to 100ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_validation_nan() {
        let mut param = Param::new(0.5);
        let actual = param.set_target_clamped(f32::NAN, 0, 0.0, 1.0);
        assert_eq!(actual, 0.5); // Should keep current value
        assert_eq!(param.current, 0.5);
    }

    #[test]
    fn test_param_validation_infinity() {
        let mut param = Param::new(0.5);
        let actual = param.set_target_clamped(f32::INFINITY, 0, 0.0, 1.0);
        assert_eq!(actual, 0.5); // Should keep current value
        assert_eq!(param.current, 0.5);
    }

    #[test]
    fn test_param_validation_clamp_max() {
        let mut param = Param::new(0.5);
        let actual = param.set_target_clamped(1.5, 0, 0.0, 1.0);
        assert_eq!(actual, 1.0); // Should clamp to max
        assert_eq!(param.current, 1.0);
    }

    #[test]
    fn test_param_validation_clamp_min() {
        let mut param = Param::new(0.5);
        let actual = param.set_target_clamped(-0.5, 0, 0.0, 1.0);
        assert_eq!(actual, 0.0); // Should clamp to min
        assert_eq!(param.current, 0.0);
    }

    #[test]
    fn test_param_validation_valid_value() {
        let mut param = Param::new(0.5);
        let actual = param.set_target_clamped(0.75, 0, 0.0, 1.0);
        assert_eq!(actual, 0.75); // Should accept valid value
        assert_eq!(param.current, 0.75);
    }

    #[test]
    fn test_validation_frequency() {
        use validation::validate_frequency;

        assert_eq!(validate_frequency(440.0, 48000.0), 440.0);
        assert_eq!(validate_frequency(-100.0, 48000.0), 0.1);
        assert_eq!(validate_frequency(30000.0, 48000.0), 24000.0);
        assert_eq!(validate_frequency(f32::NAN, 48000.0), 440.0);
    }

    #[test]
    fn test_validation_gain() {
        use validation::validate_gain;

        assert_eq!(validate_gain(0.5, 2.0), 0.5);
        assert_eq!(validate_gain(-0.5, 2.0), 0.0);
        assert_eq!(validate_gain(3.0, 2.0), 2.0);
        assert_eq!(validate_gain(f32::NAN, 2.0), 1.0);
    }
}
