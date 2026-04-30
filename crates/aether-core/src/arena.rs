//! Generational arena allocator.
//!
//! Provides O(1) alloc/dealloc with use-after-free detection via generation counters.
//! All memory is pre-allocated at startup — zero heap activity in the RT thread.

use crate::MAX_NODES;

/// A typed, generational index into the arena.
/// The generation field prevents ABA problems and dangling references.
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
pub struct Arena<T> {
    entries: Vec<Entry<T>>,
    free_head: Option<u32>,
    len: usize,
}

impl<T> Arena<T> {
    /// Allocate a new arena with `capacity` pre-reserved slots.
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
    /// Returns `None` if the arena is full.
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
