use std::mem::{size_of, transmute};
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
    /// How many of `T` can be fit into the current block.
    /// (A block can consist of many contiguous blocks)
    count: i32,
    /// Index to the next free block in the freelist.
    // `Option<i32>` is 8 bytes, why? Not ideal.
    next_block_index: Option<i32>,
}

// Freelist implementations.
impl<T> Freelist<T> {
    /// Create a new, empty freelist.
    pub fn new() -> Self {
        // Need to assert the size of the type to ensure `Block` can fit.
        // This is currently done at run time, can't it be done at compile time?
        assert!(size_of::<T>() >= size_of::<Block>());
        Freelist {
            heap_data: Vec::with_capacity(0),
            first_free_block: None

        }
    } 

    /// Get the size of the type in bytes (includes alignment).
    // This *can* be evauluated at compile-time, but is it always?
    pub const fn type_size(&self) -> usize {
        size_of::<T>()
    }

    /// Check if the freelist has an empty (free) block.
    pub fn has_free_block(&self) -> bool {
        self.first_free_block != None
    }

    /// Allocate enough memory for the amount of elements requested.
    ///
    /// # Safety
    ///
    /// This is unsafe.
    ///
    /// * The vector can be truncated without `T` being dropped.
    /// * When extending the vector the memory is uninitialized (which is actually better for performance in this case).
    unsafe fn allocate(&mut self, element_count:usize) {
        self.heap_data.set_len(element_count);
    }

    /// Get a mutable ref the block at the given index.
    ///
    /// # Safety
    ///
    /// This is highly unsafe.  
    ///
    /// * Performs a non-primitive cast.
    unsafe fn get_block_mut(&mut self, index: usize) -> &mut Block {
        transmute(&mut self.heap_data[index])
    }

    /// Get a const ref the block at the given index.
    ///
    /// # Safety
    ///
    /// This is unsafe.
    ///
    /// * Performs a non-primitive cast.
    unsafe fn get_block(&self, index: usize) -> &Block {
        transmute(&self.heap_data[index])
    }
}
