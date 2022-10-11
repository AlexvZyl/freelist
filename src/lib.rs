use std::mem;
use std::vec::Vec;

/// A cache coherent, heap allocated collection.
pub struct Freelist<T> {
    type_size: usize,
    data: Vec<T>,
    first_free_block: Option<i32>,
}

/// Used to describe an open block in freelist::Freelist<T>.
/// A block can consist of many blocks, if they are contiguous.
struct Block {
    /// Describes the size of the block by how many of <T> it can fit.
    /// (A block can consist of many contiguous blocks)
    size: i32,
    /// Index to the next free block in the freelist.
    next_block: Option<i32>,
}

// Freelist implementations.
impl<T> Freelist<T> {
    /// Create a new empty freelist.
    pub fn new() -> Self {
        Freelist {
            type_size: mem::size_of::<T>(),
            data: Vec::new(),
            first_free_block: Some::<i32>(0),
        }
    }

    pub fn type_size(&self) -> usize {
        self.type_size
    }

    fn has_free_block(&self) -> bool {
        self.first_free_block != None
    }
}
