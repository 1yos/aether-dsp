//! Sample-accurate parameter automation.
//!
//! Each Param smooths from `current` toward `target` over a fixed ramp.
//! No allocations. No locks. Safe to read/write from the RT thread.

/// A single smoothed parameter.
/// Uses a linear ramp: current += step each sample until target is reached.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Param {
    pub current: f32,
    pub target: f32,
    /// Per-sample increment. Set by `set_target`.
    pub step: f32,
}

impl Param {
    pub fn new(value: f32) -> Self {
        Self {
            current: value,
            target: value,
            step: 0.0,
        }
    }

    /// Schedule a ramp to `target` over `ramp_samples` samples.
    /// Call from the control thread before pushing a `UpdateParam` command.
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

    /// Advance by one sample. Call once per sample in the RT loop.
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
    /// Avoids branching inside the hot loop.
    #[inline]
    pub fn fill_buffer(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            *sample = self.current;
            self.tick();
        }
    }
}

/// A fixed-size block of parameters for a node.
/// Sized to fit common DSP nodes without heap allocation.
#[derive(Debug, Clone, Copy)]
pub struct ParamBlock {
    pub params: [Param; 8],
    pub count: usize,
}

impl ParamBlock {
    pub fn new() -> Self {
        Self {
            params: [Param::new(0.0); 8],
            count: 0,
        }
    }

    pub fn add(&mut self, value: f32) -> usize {
        let idx = self.count;
        self.params[idx] = Param::new(value);
        self.count += 1;
        idx
    }

    #[inline(always)]
    pub fn get(&self, idx: usize) -> &Param {
        &self.params[idx]
    }

    #[inline(always)]
    pub fn get_mut(&mut self, idx: usize) -> &mut Param {
        &mut self.params[idx]
    }

    /// Tick all active params by one sample.
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
