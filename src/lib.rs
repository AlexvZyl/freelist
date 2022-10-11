use std::mem;
use std::vec::Vec;

/// A cache coherent, heap allocated collection.
pub struct Freelist<T> {
    /// Pointer to the data located on the heap.
    heap_data: Vec<T>,
    /// Index to the first free block in the list.
    first_free_block: Option<i32>,
}

/// Used to describe an open block in freelist::Freelist<T>.
/// A block can consist of many blocks, if they are contiguous.
struct Block {
    /// How many of <T> can be fit into the current block.
    /// (A block can consist of many contiguous blocks)
    count: i32,
    /// Index to the next free block in the freelist.
    next_block_index: Option<i32>,
}

// Freelist implementations.
impl<T> Freelist<T> {

    /// Create a new empty freelist.
    pub fn new() -> Self {
        Freelist {
            heap_data: Vec::with_capacity(0),
            first_free_block: Some(0)
        }
    }

    /// Get the size of the type in bytes (includes alignment).
    // This *can* be evauluated at compile-time, but is it always?
    pub fn type_size(&self) -> usize {
        mem::size_of::<T>()
    }

    /// Check if the freelist has an empty (free) block.
    fn has_free_block(&self) -> bool {
        self.first_free_block != None
    }

    /// Allocate enough memory for the amount of elements requested.
    /// This is only unsafe since the memory will be uninitialized.
    fn allocate(&mut self, element_count:usize) {
        unsafe { self.heap_data.set_len(element_count); }
    }
}
