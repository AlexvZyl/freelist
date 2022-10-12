#![allow(dead_code)]

use std::mem::{size_of, transmute};
use std::vec::Vec;

mod block;
use block::Block;

/// The maximum size (in bytes) of the freelist.
const MAX_SIZE_BYTES: i32 = 2147483647;

/// A cache coherent, heap allocated collection.
/// This data structure uses i32 instead of usize due to the constrasints placed
/// on `Block`. It will never require 64 bit indexing.  What about smaller
/// architectures?
pub struct Freelist<T>
{
    /// Pointer to the data located on the heap.
    heap_data: Vec<T>,
    /// Index to the first free block in the list.
    first_free_block: Option<i32>,
    /// The amount of elements the freelist can hold.
    capacity: i32,
}

// Freelist implementations.
impl<T> Freelist<T>
{
    /// Create a new, empty freelist.
    pub fn new() -> Self
    {
        // Need to assert the size of the type to ensure `Block` can fit.
        // This is currently done at runtime, can't it be done at compile time?
        assert!(size_of::<T>() >= size_of::<Block>());
        Freelist { heap_data: Vec::with_capacity(0),
                   first_free_block: None,
                   capacity: 0 }
    }

    /// Get the size of the type in bytes (includes alignment).
    // This *can* be evauluated at compile-time, but is it always?
    pub const fn type_size_bytes(&self) -> i32 { size_of::<T>() as i32 }

    /// Check if the freelist has an empty (free) block.
    pub fn has_free_block(&self) -> bool { self.first_free_block != None }

    /// Allocate enough memory for the amount of elements requested.
    /// This is regarded as a low-level function and does not do any required
    /// checks.
    ///
    /// # Safety
    ///
    /// This is unsafe.
    ///
    /// * The vector can be truncated without `T` being dropped.
    /// * When extending the vector the memory is uninitialized (which is
    ///   actually better for performance in this case).
    fn allocate(&mut self, element_count: i32)
    {
        unsafe {
            self.heap_data.set_len(element_count as usize);
        }
    }

    /// Get a mutable ref the block at the given index.
    ///
    /// # Safety
    ///
    /// This is highly unsafe.  
    ///
    /// * Performs a non-primitive cast.
    fn get_block_mut(&mut self, index: i32) -> &mut Block
    {
        unsafe { transmute(&mut self.heap_data[index as usize]) }
    }

    /// Get a const ref the block at the given index.
    ///
    /// # Safety
    ///
    /// This is unsafe.
    ///
    /// * Performs a non-primitive cast.
    fn get_block(&self, index: i32) -> &Block
    {
        unsafe { transmute(&self.heap_data[index as usize]) }
    }

    /// Checks if the blocks are adjacent.
    fn blocks_are_adjacent(&self, index_1: i32, index_2: i32) -> bool
    {
        index_1 + self.get_block(index_1).count == index_2
    }

    /// Shrink the freelist to the smallest it can be.
    pub fn shrink_to_fit() {}

    /// Traverse the list to find the last free block.
    /// Returns -1 if none is found.
    fn find_last_free_block(&self) -> i32
    {
        // No blocks to search.
        if !self.has_free_block()
        {
            return -1;
        };
        // Search blocks.
        loop
        {
            let current_block_index = self.first_free_block.unwrap();
            let current_block = self.get_block(current_block_index);
            if !current_block.has_next_block()
            {
                return current_block_index;
            }
        }
    }

    /// Find the first free block that fits the size requirement.
    /// Returns the index to the block.
    fn find_first_free_block(&self, element_count: i32) -> i32
    {
        // No blocks to search.
        if !self.has_free_block()
        {
            return -1;
        };
        // Search blocks.
        loop
        {
            let current_block_index = self.first_free_block.unwrap();
            let current_block = self.get_block(current_block_index);
            // Found large enough block.
            if current_block.count >= element_count
            {
                return current_block_index;
            }
            // Could not find a block.
            if !current_block.has_next_block()
            {
                return -1;
            };
        }
    }

    /// Get the capacity of the freelist.
    pub fn capacity(&self) -> i32 { self.capacity }

    /// Get the capacity of the freelist in bytes.
    pub fn capacity_bytes(&self) -> i32 { self.capacity() * self.type_size_bytes() }
}
