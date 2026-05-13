//! Generational arena allocator.
//!
//! Provides O(1) alloc/dealloc with use-after-free detection via generation counters.
//! All memory is pre-allocated at startup — zero heap activity in the RT thread.

use crate::MAX_NODES;

/// A typed, generational index into the arena.
///
/// Combines an index with a generation counter to prevent use-after-free bugs
/// and ABA problems. When a slot is reused, its generation is incremented,
/// invalidating all old `NodeId`s pointing to that slot.
///
/// # Example
///
/// ```
/// use aether_core::arena::{Arena, NodeId};
///
/// let mut arena: Arena<i32> = Arena::with_capacity(10);
///
/// let id1 = arena.insert(42).unwrap();
/// arena.remove(id1);
///
/// let id2 = arena.insert(99).unwrap();
///
/// // Same slot, different generation
/// assert_eq!(id1.index, id2.index);
/// assert_ne!(id1.generation, id2.generation);
///
/// // Old ID is invalid
/// assert!(arena.get(id1).is_none());
/// // New ID is valid
/// assert_eq!(*arena.get(id2).unwrap(), 99);
/// ```
///
/// # Safety
///
/// The generation counter prevents:
/// - **Use-after-free:** Old IDs become invalid when slots are reused
/// - **ABA problem:** Can't confuse old and new values in the same slot
/// - **Dangling references:** Stale IDs return `None` instead of wrong data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    pub index: u32,
    pub generation: u32,
}

impl NodeId {
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: u32::MAX,
    };
}

struct Entry<T> {
    /// The stored value, valid only when `occupied` is true.
    value: Option<T>,
    /// Current generation. Incremented on each removal.
    generation: u32,
    /// Next free slot index, valid only when `value` is None.
    next_free: Option<u32>,
}

/// Fixed-capacity generational arena.
///
/// A pre-allocated, generational arena allocator that provides O(1) insert/remove
/// with use-after-free detection. All memory is allocated upfront - zero heap
/// activity during audio processing.
///
/// # Features
///
/// - **O(1) insert/remove:** Constant-time operations
/// - **Generational indices:** Prevents use-after-free bugs
/// - **Pre-allocated:** No runtime allocation
/// - **Real-time safe:** No locks, no allocation, bounded time
///
/// # Example
///
/// ```
/// use aether_core::arena::Arena;
///
/// let mut arena: Arena<String> = Arena::with_capacity(100);
///
/// // Insert values
/// let id1 = arena.insert("Hello".to_string()).unwrap();
/// let id2 = arena.insert("World".to_string()).unwrap();
///
/// // Access values
/// assert_eq!(arena.get(id1).unwrap(), "Hello");
/// assert_eq!(arena.get(id2).unwrap(), "World");
///
/// // Remove and reuse slots
/// arena.remove(id1);
/// let id3 = arena.insert("Rust".to_string()).unwrap();
///
/// // Old ID is invalid, new ID is valid
/// assert!(arena.get(id1).is_none());
/// assert_eq!(arena.get(id3).unwrap(), "Rust");
/// ```
///
/// # Capacity
///
/// The arena has a fixed capacity set at creation. When full, `insert()`
/// returns `None`. Removed slots are recycled via a free list.
///
/// # Use Case
///
/// Perfect for managing DSP nodes in an audio graph where:
/// - Nodes are added/removed dynamically
/// - Need to prevent dangling references
/// - Must avoid runtime allocation
/// - Require O(1) operations
///
/// # See Also
///
/// * [`NodeId`] - Generational index type
pub struct Arena<T> {
    entries: Vec<Entry<T>>,
    free_head: Option<u32>,
    len: usize,
}

impl<T> Arena<T> {
    /// Allocate a new arena with `capacity` pre-reserved slots.
    ///
    /// All memory is allocated upfront. The arena can hold up to `capacity`
    /// items simultaneously. Removed slots are recycled.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of items the arena can hold
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::arena::Arena;
    ///
    /// let arena: Arena<i32> = Arena::with_capacity(1000);
    /// assert_eq!(arena.len(), 0);
    /// assert!(arena.is_empty());
    /// ```
    ///
    /// # Performance
    ///
    /// - Time: O(capacity) - initializes free list
    /// - Space: O(capacity) - pre-allocates all slots
    /// - No further allocation after construction
    pub fn with_capacity(capacity: usize) -> Self {
        let mut entries = Vec::with_capacity(capacity);
        for i in 0..capacity {
            let next = if i + 1 < capacity {
                Some((i + 1) as u32)
            } else {
                None
            };
            entries.push(Entry {
                value: None,
                generation: 0,
                next_free: next,
            });
        }
        Self {
            entries,
            free_head: if capacity > 0 { Some(0) } else { None },
            len: 0,
        }
    }

    /// Insert a value, returning its generational id.
    ///
    /// Allocates a slot from the free list and stores the value.
    /// Returns a `NodeId` that can be used to access the value later.
    ///
    /// # Arguments
    ///
    /// * `value` - Value to insert
    ///
    /// # Returns
    ///
    /// * `Some(NodeId)` - Generational ID for the inserted value
    /// * `None` - Arena is full (all slots occupied)
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::arena::Arena;
    ///
    /// let mut arena: Arena<String> = Arena::with_capacity(10);
    ///
    /// let id = arena.insert("Hello".to_string()).unwrap();
    /// assert_eq!(arena.get(id).unwrap(), "Hello");
    /// assert_eq!(arena.len(), 1);
    /// ```
    ///
    /// # Capacity Exhaustion
    ///
    /// ```
    /// use aether_core::arena::Arena;
    ///
    /// let mut arena: Arena<i32> = Arena::with_capacity(2);
    ///
    /// let id1 = arena.insert(1).unwrap();
    /// let id2 = arena.insert(2).unwrap();
    /// let id3 = arena.insert(3); // None - arena is full
    ///
    /// assert!(id3.is_none());
    ///
    /// // Remove a slot to make space
    /// arena.remove(id1);
    /// let id4 = arena.insert(4).unwrap(); // Now succeeds
    /// ```
    ///
    /// # Performance
    ///
    /// - Time: O(1)
    /// - Space: O(0) - no allocation
    /// - Real-time safe
    pub fn insert(&mut self, value: T) -> Option<NodeId> {
        let index = self.free_head?;
        let entry = &mut self.entries[index as usize];
        self.free_head = entry.next_free;
        let generation = entry.generation;
        entry.value = Some(value);
        entry.next_free = None;
        self.len += 1;
        Some(NodeId { index, generation })
    }

    /// Remove a value by id. Returns the value if the id was valid.
    ///
    /// Removes the value from the arena, increments the slot's generation
    /// counter (invalidating all existing IDs), and returns the slot to
    /// the free list for reuse.
    ///
    /// # Arguments
    ///
    /// * `id` - Generational ID of the value to remove
    ///
    /// # Returns
    ///
    /// * `Some(T)` - The removed value (if ID was valid)
    /// * `None` - ID was invalid (wrong generation or already removed)
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::arena::Arena;
    ///
    /// let mut arena: Arena<i32> = Arena::with_capacity(10);
    ///
    /// let id = arena.insert(42).unwrap();
    /// assert_eq!(arena.len(), 1);
    ///
    /// let value = arena.remove(id).unwrap();
    /// assert_eq!(value, 42);
    /// assert_eq!(arena.len(), 0);
    ///
    /// // ID is now invalid
    /// assert!(arena.get(id).is_none());
    /// assert!(arena.remove(id).is_none());
    /// ```
    ///
    /// # Generation Bump
    ///
    /// ```
    /// use aether_core::arena::Arena;
    ///
    /// let mut arena: Arena<i32> = Arena::with_capacity(10);
    ///
    /// let id1 = arena.insert(1).unwrap();
    /// let gen1 = id1.generation;
    ///
    /// arena.remove(id1);
    ///
    /// let id2 = arena.insert(2).unwrap();
    /// let gen2 = id2.generation;
    ///
    /// // Same slot, different generation
    /// assert_eq!(id1.index, id2.index);
    /// assert_eq!(gen2, gen1 + 1);
    /// ```
    ///
    /// # Performance
    ///
    /// - Time: O(1)
    /// - Space: O(0) - no allocation
    /// - Real-time safe
    pub fn remove(&mut self, id: NodeId) -> Option<T> {
        let entry = self.entries.get_mut(id.index as usize)?;
        if entry.generation != id.generation || entry.value.is_none() {
            return None;
        }
        let value = entry.value.take();
        // Bump generation to invalidate all existing ids pointing here.
        entry.generation = entry.generation.wrapping_add(1);
        entry.next_free = self.free_head;
        self.free_head = Some(id.index);
        self.len -= 1;
        value
    }

    /// Get a shared reference. Returns `None` for stale ids.
    ///
    /// Returns a reference to the value if the ID is valid (correct generation
    /// and slot is occupied). Returns `None` if the ID is stale or invalid.
    ///
    /// # Arguments
    ///
    /// * `id` - Generational ID to look up
    ///
    /// # Returns
    ///
    /// * `Some(&T)` - Reference to the value (if ID is valid)
    /// * `None` - ID is invalid or stale
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::arena::Arena;
    ///
    /// let mut arena: Arena<String> = Arena::with_capacity(10);
    ///
    /// let id = arena.insert("Hello".to_string()).unwrap();
    ///
    /// // Valid ID
    /// assert_eq!(arena.get(id).unwrap(), "Hello");
    ///
    /// arena.remove(id);
    ///
    /// // Stale ID
    /// assert!(arena.get(id).is_none());
    /// ```
    ///
    /// # Performance
    ///
    /// - Time: O(1)
    /// - Inlined for zero call overhead
    /// - Real-time safe
    #[inline]
    pub fn get(&self, id: NodeId) -> Option<&T> {
        let entry = self.entries.get(id.index as usize)?;
        if entry.generation == id.generation {
            entry.value.as_ref()
        } else {
            None
        }
    }

    /// Get a mutable reference. Returns `None` for stale ids.
    ///
    /// Returns a mutable reference to the value if the ID is valid.
    /// Returns `None` if the ID is stale or invalid.
    ///
    /// # Arguments
    ///
    /// * `id` - Generational ID to look up
    ///
    /// # Returns
    ///
    /// * `Some(&mut T)` - Mutable reference to the value (if ID is valid)
    /// * `None` - ID is invalid or stale
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::arena::Arena;
    ///
    /// let mut arena: Arena<i32> = Arena::with_capacity(10);
    ///
    /// let id = arena.insert(42).unwrap();
    ///
    /// // Modify the value
    /// if let Some(value) = arena.get_mut(id) {
    ///     *value = 99;
    /// }
    ///
    /// assert_eq!(*arena.get(id).unwrap(), 99);
    /// ```
    ///
    /// # Performance
    ///
    /// - Time: O(1)
    /// - Inlined for zero call overhead
    /// - Real-time safe
    #[inline]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        let entry = self.entries.get_mut(id.index as usize)?;
        if entry.generation == id.generation {
            entry.value.as_mut()
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Default arena sized for the project's node limit.
pub fn default_node_arena<T>() -> Arena<T> {
    Arena::with_capacity(MAX_NODES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get_remove() {
        let mut arena: Arena<i32> = Arena::with_capacity(4);
        let id = arena.insert(42).unwrap();
        assert_eq!(*arena.get(id).unwrap(), 42);
        let val = arena.remove(id).unwrap();
        assert_eq!(val, 42);
        assert!(arena.get(id).is_none());
    }

    #[test]
    fn generation_prevents_aba() {
        let mut arena: Arena<i32> = Arena::with_capacity(4);
        let id1 = arena.insert(1).unwrap();
        arena.remove(id1).unwrap();
        let id2 = arena.insert(2).unwrap();
        // Same slot index, bumped generation.
        assert_eq!(id1.index, id2.index);
        assert_ne!(id1.generation, id2.generation);
        assert!(arena.get(id1).is_none());
        assert_eq!(*arena.get(id2).unwrap(), 2);
    }

    #[test]
    fn capacity_exhaustion() {
        let mut arena: Arena<i32> = Arena::with_capacity(2);
        let a = arena.insert(1).unwrap();
        let _b = arena.insert(2).unwrap();
        assert!(arena.insert(3).is_none()); // full
        arena.remove(a).unwrap();
        assert!(arena.insert(3).is_some()); // slot recycled
    }
}
