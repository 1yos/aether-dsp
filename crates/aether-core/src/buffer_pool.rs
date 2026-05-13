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

/// Pre-allocated audio buffer pool.
///
/// Manages a fixed pool of audio buffers (64 samples each) with O(1) acquire/release.
/// All memory is allocated at startup - the RT thread only borrows slices with
/// zero allocation overhead.
///
/// # Design
///
/// - **Structure-of-Arrays:** Flat `Vec<f32>` storage for cache efficiency
/// - **Free list:** Stack of available buffer IDs
/// - **Pre-zeroed:** Buffers are zeroed on acquire
/// - **Real-time safe:** No allocation, no locks, bounded time
///
/// # Example
///
/// ```
/// use aether_core::buffer_pool::BufferPool;
/// use aether_core::BUFFER_SIZE;
///
/// let mut pool = BufferPool::new(100);
///
/// // Acquire buffers
/// let buf1 = pool.acquire().unwrap();
/// let buf2 = pool.acquire().unwrap();
///
/// // Use buffers
/// {
///     let data = pool.get_mut(buf1);
///     data[0] = 1.0;
/// }
///
/// // Release buffers
/// pool.release(buf1);
/// pool.release(buf2);
///
/// // Buffers are recycled
/// let buf3 = pool.acquire().unwrap();
/// ```
///
/// # Capacity
///
/// The pool has a fixed capacity set at creation. When exhausted,
/// `acquire()` returns `None`. Released buffers are immediately available
/// for reuse.
///
/// # See Also
///
/// * [`BufferId`] - Opaque buffer handle
pub struct BufferPool {
    /// Flat storage: `buffers[id * BUFFER_SIZE .. (id+1) * BUFFER_SIZE]`
    storage: Vec<f32>,
    free_list: Vec<u32>,
    capacity: usize,
}

impl BufferPool {
    /// Creates a new buffer pool with the specified capacity.
    ///
    /// Pre-allocates all buffers upfront. No further allocation occurs
    /// during audio processing.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Number of buffers to pre-allocate
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::buffer_pool::BufferPool;
    ///
    /// let pool = BufferPool::new(100);
    /// assert_eq!(pool.capacity(), 100);
    /// assert_eq!(pool.available(), 100);
    /// ```
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
    ///
    /// Pops a buffer ID from the free list and zeros the buffer before
    /// returning it. The buffer is guaranteed to contain all zeros.
    ///
    /// # Returns
    ///
    /// * `Some(BufferId)` - Handle to an available buffer
    /// * `None` - Pool is exhausted (all buffers in use)
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::buffer_pool::BufferPool;
    ///
    /// let mut pool = BufferPool::new(10);
    ///
    /// let buf = pool.acquire().unwrap();
    /// let data = pool.get(buf);
    /// assert_eq!(data[0], 0.0); // Guaranteed zero
    /// ```
    ///
    /// # Performance
    ///
    /// - Time: O(1)
    /// - Zeros 64 samples (256 bytes)
    /// - Real-time safe
    pub fn acquire(&mut self) -> Option<BufferId> {
        let id = self.free_list.pop()?;
        // Zero the buffer before handing it out.
        let start = id as usize * BUFFER_SIZE;
        self.storage[start..start + BUFFER_SIZE].fill(0.0);
        Some(BufferId(id))
    }

    /// Release a buffer back to the pool. O(1).
    ///
    /// Returns the buffer to the free list for reuse. The buffer's contents
    /// are not cleared until the next `acquire()`.
    ///
    /// # Arguments
    ///
    /// * `id` - Buffer ID to release
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::buffer_pool::BufferPool;
    ///
    /// let mut pool = BufferPool::new(10);
    ///
    /// let buf = pool.acquire().unwrap();
    /// assert_eq!(pool.available(), 9);
    ///
    /// pool.release(buf);
    /// assert_eq!(pool.available(), 10);
    /// ```
    ///
    /// # Performance
    ///
    /// - Time: O(1)
    /// - No zeroing (deferred to acquire)
    /// - Real-time safe
    pub fn release(&mut self, id: BufferId) {
        debug_assert!((id.0 as usize) < self.capacity, "BufferId out of range");
        self.free_list.push(id.0);
    }

    /// Get a read-only slice for a buffer.
    ///
    /// Returns a reference to the 64-sample buffer identified by `id`.
    /// This is a zero-cost operation - just pointer arithmetic.
    ///
    /// # Arguments
    ///
    /// * `id` - Buffer ID from `acquire()`
    ///
    /// # Returns
    ///
    /// A reference to a 64-sample audio buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::buffer_pool::BufferPool;
    ///
    /// let mut pool = BufferPool::new(10);
    /// let buf = pool.acquire().unwrap();
    ///
    /// // Write to buffer
    /// pool.get_mut(buf)[0] = 1.0;
    ///
    /// // Read from buffer
    /// let data = pool.get(buf);
    /// assert_eq!(data[0], 1.0);
    /// ```
    ///
    /// # Performance
    ///
    /// - Time: O(1) - inline pointer arithmetic
    /// - No bounds checking in release builds
    /// - Real-time safe
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `id` is out of range.
    #[inline(always)]
    pub fn get(&self, id: BufferId) -> &[f32; BUFFER_SIZE] {
        let start = id.0 as usize * BUFFER_SIZE;
        self.storage[start..start + BUFFER_SIZE].try_into().unwrap()
    }

    /// Get a mutable slice for a buffer.
    ///
    /// Returns a mutable reference to the 64-sample buffer identified by `id`.
    /// Use this to write audio data into the buffer.
    ///
    /// # Arguments
    ///
    /// * `id` - Buffer ID from `acquire()`
    ///
    /// # Returns
    ///
    /// A mutable reference to a 64-sample audio buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::buffer_pool::BufferPool;
    ///
    /// let mut pool = BufferPool::new(10);
    /// let buf = pool.acquire().unwrap();
    ///
    /// // Fill buffer with sine wave
    /// let data = pool.get_mut(buf);
    /// for (i, sample) in data.iter_mut().enumerate() {
    ///     let phase = i as f32 / 64.0;
    ///     *sample = (phase * std::f32::consts::TAU).sin();
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// - Time: O(1) - inline pointer arithmetic
    /// - No bounds checking in release builds
    /// - Real-time safe
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `id` is out of range.
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
