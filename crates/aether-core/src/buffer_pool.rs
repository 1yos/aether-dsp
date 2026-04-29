//! Pre-allocated audio buffer pool.
//!
//! All buffers are allocated at startup. The RT thread only borrows slices —
//! no allocation, no deallocation, no system calls.

use crate::{BUFFER_SIZE, MAX_BUFFERS};

/// Opaque handle to a buffer in the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferId(pub u32);

impl BufferId {
    pub const SILENCE: Self = Self(u32::MAX);
}

/// Structure-of-Arrays audio buffer pool.
/// Each buffer is exactly BUFFER_SIZE f32 samples.
pub struct BufferPool {
    /// Flat storage: `buffers[id * BUFFER_SIZE .. (id+1) * BUFFER_SIZE]`
    storage: Vec<f32>,
    free_list: Vec<u32>,
    capacity: usize,
}

impl BufferPool {
    pub fn new(capacity: usize) -> Self {
        let storage = vec![0.0f32; capacity * BUFFER_SIZE];
        let free_list: Vec<u32> = (0..capacity as u32).rev().collect();
        Self {
            storage,
            free_list,
            capacity,
        }
    }

    /// Acquire a zeroed buffer. O(1). Returns None if pool is exhausted.
    pub fn acquire(&mut self) -> Option<BufferId> {
        let id = self.free_list.pop()?;
        // Zero the buffer before handing it out.
        let start = id as usize * BUFFER_SIZE;
        self.storage[start..start + BUFFER_SIZE].fill(0.0);
        Some(BufferId(id))
    }

    /// Release a buffer back to the pool. O(1).
    pub fn release(&mut self, id: BufferId) {
        debug_assert!((id.0 as usize) < self.capacity, "BufferId out of range");
        self.free_list.push(id.0);
    }

    /// Get a read-only slice for a buffer.
    #[inline(always)]
    pub fn get(&self, id: BufferId) -> &[f32; BUFFER_SIZE] {
        let start = id.0 as usize * BUFFER_SIZE;
        self.storage[start..start + BUFFER_SIZE].try_into().unwrap()
    }

    /// Get a mutable slice for a buffer.
    #[inline(always)]
    pub fn get_mut(&mut self, id: BufferId) -> &mut [f32; BUFFER_SIZE] {
        let start = id.0 as usize * BUFFER_SIZE;
        (&mut self.storage[start..start + BUFFER_SIZE])
            .try_into()
            .unwrap()
    }

    /// Return a zeroed silence buffer (static, no allocation).
    pub fn silence() -> &'static [f32; BUFFER_SIZE] {
        static SILENCE: [f32; BUFFER_SIZE] = [0.0f32; BUFFER_SIZE];
        &SILENCE
    }

    pub fn available(&self) -> usize {
        self.free_list.len()
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new(MAX_BUFFERS)
    }
}
