//! State continuity framework.
//!
//! When a node is replaced or reconnected, its predecessor's state is transferred
//! to preserve audio continuity (no clicks, no phase resets).

/// Trait for nodes that carry persistent DSP state across mutations.
pub trait Stateful {
    type State: Copy;

    fn capture_state(&self) -> Self::State;
    fn restore_state(&mut self, state: Self::State);
}

/// Opaque state blob — large enough for all built-in node types.
/// Stored as raw bytes to avoid generics in the graph.
#[derive(Clone, Copy)]
pub struct StateBlob {
    pub bytes: [u8; 256],
    pub len: usize,
}

impl StateBlob {
    pub const EMPTY: Self = Self {
        bytes: [0u8; 256],
        len: 0,
    };

    pub fn from_value<T: Copy>(val: &T) -> Self {
        let size = std::mem::size_of::<T>();
        assert!(size <= 256, "StateBlob overflow: type too large");
        let mut blob = Self::EMPTY;
        blob.len = size;
        // SAFETY: T is Copy, size is bounded, dst is valid.
        unsafe {
            std::ptr::copy_nonoverlapping(
                val as *const T as *const u8,
                blob.bytes.as_mut_ptr(),
                size,
            );
        }
        blob
    }

    pub fn to_value<T: Copy>(&self) -> T {
        assert_eq!(self.len, std::mem::size_of::<T>());
        // SAFETY: bytes were written from a valid T.
        unsafe { std::ptr::read(self.bytes.as_ptr() as *const T) }
    }
}

impl Default for StateBlob {
    fn default() -> Self {
        Self::EMPTY
    }
}
