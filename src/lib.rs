#![allow(dead_code)]

use std::mem::{size_of, transmute};
use std::vec::Vec;

mod block;
use block::Block;

/// The maximum size (in bytes) of the freelist.
const MAX_SIZE_BYTES: i32 = 2147483647;

/// A cache coherent, heap allocated collection.
/// This data structure uses i32 instead of usize due to the constraints placed
/// on `Block`. It will never require 64 bit indexing.  What about smaller
/// architectures?
pub struct Freelist<T>
{
    /// Pointer to the data located on the heap.
    heap_data: Vec<T>,
    /// Index to the first free block in the list.
    first_free_block: Option<i32>,
    /// The number of allocated blocks.
    used_blocks: i32,
}

// Freelist implementations.
impl<T> Freelist<T>
{
    /// Create a new, empty freelist.
    pub fn new() -> Self
    {
        // Need to assert the size of the type to ensure `Block` can fit.
        // This is currently done at runtime, can it be done at compile time?
        assert!(size_of::<T>() >= size_of::<Block>());
        Freelist { heap_data: Vec::with_capacity(0),
                   first_free_block: None,
                   used_blocks: 0 }
    }

    /// Allocate enough memory for the amount of elements requested.
    /// This is regarded as a low-level function and does not do any checks,
    /// manipulation or lifetime management.
    /// See this as a call to `malloc()`, but with the existing data being
    /// copied over.
    ///
    /// # Safety
    ///
    /// This is highly unsafe.
    ///
    /// * The vector can be truncated without `T` being dropped.
    /// * When extending the vector, the memory is uninitialized (which is
    ///   actually better for performance).
    unsafe fn allocate(&mut self, block_count: i32)
    {
        // Does this copy all of the data to the new vector?
        self.heap_data.set_len(block_count as usize);
    }

    /// Increases the amount of memory available by the specified amount.  
    /// Ensures blocks are handled corretly.
    ///
    /// # Safety.
    ///
    /// This is highly unsafe.
    ///
    /// * `Freelist::allocate()` is called.
    unsafe fn extend_by(&mut self, block_count: i32)
    {
        // Allocate the required memory.
        self.allocate(self.capacity_blocks() + block_count);
    }

    /// Get a mutable ref the block at the given index.
    ///
    /// # Safety
    ///
    /// This is unsafe.  
    ///
    /// * Performs a non-primitive cast.
    unsafe fn get_block_mut(&mut self, index: i32) -> &mut Block
    {
        transmute(&mut self.heap_data[index as usize])
    }

    /// Get a const ref the block at the given index.
    ///
    /// # Safety
    ///
    /// This is unsafe.
    ///
    /// * Performs a non-primitive cast.
    unsafe fn get_block(&self, index: i32) -> &Block { transmute(&self.heap_data[index as usize]) }

    /// Checks if the blocks are adjacent.
    ///
    /// # Safety
    ///
    /// This is unsafe.
    ///
    /// * Performs a non-primitive cast when checknig adjacency.
    unsafe fn blocks_are_adjacent(&self, first_block_index: i32, second_block_index: i32) -> bool
    {
        first_block_index + self.get_block(first_block_index).element_count == second_block_index
    }

    /// Find and commit a block that fits the size requirement.
    /// If it does not find a block large enough it will resize.  The caller is
    /// guaranteed to get a block.
    fn find_and_commit_block(&mut self, element_count: i32) -> i32
    {
        let (prev_block_ixd, block_idx) = self.find_first_fit(element_count);
        match block_idx
        {
            // No blocks are large enough.
            None =>
            {
                // TODO: Allocate more memory if no blocks are found.
                // Once more memory has been allocated the last block can be used.
                return block_idx.unwrap();
            }

            // Commit the block.
            Some(..) =>
            {
                return block_idx.unwrap();
            }
        }
    }

    /// Commit the block at the index and update the blocks.
    // This function has a lot of match blocks, can this be reduced?  This is due to the extensive
    // use of Option<T>.
    fn commit_block(&mut self, prev_block_index: Option<i32>, block_idx: i32, element_count: i32)
    {
        // Getting blocks, performs non-primitive casts.
        unsafe {

            let mut block = self.get_block_mut(block_idx);

            // Entire block is consumed.
            if element_count == block.element_count
            {
                match prev_block_index
                {
                    None => self.first_free_block = block.get_next_block_index(),

                    Some(..) => 
                    {
                        let next_block = block.get_next_block_index();
                        let prev_block = self.get_block_mut(prev_block_index.unwrap());
                        match next_block
                        {
                            None => prev_block.set_no_next_block(),
                            Some(..) => prev_block.connect(next_block.unwrap()),
                        }
                    }
                }
            }
            
            // Part of block is consumed.
            else if element_count < block.element_count
            {
                block.element_count -= element_count  
            }

            // TODO: Throw. Too many elements for block.
            else 
            {

            }
        }
    }

    /// Connect two blocks based on the index.
    fn connect_blocks(&mut self, first_block_index: i32, second_block_index: i32)
    {
        unsafe {
            self.get_block_mut(first_block_index).connect(second_block_index);
        }
    }

    /// Create a new block at the index with the given values and return a mutable reference.
    fn create_new_block_mut(&mut self, block_index: i32, element_count: i32, next_block_index: Option<i32>) -> &mut Block
    {
        unsafe {
            let mut new_block = self.get_block_mut(block_index);
            new_block.set_next_block_index(next_block_index);
            new_block.element_count = element_count;
            return new_block
        }
    }

    /// Create a new block at the index with the given values and return a const reference.
    fn create_new_block(&mut self, block_index: i32, element_count: i32, next_block_index: Option<i32>) -> &Block 
    {
        self.create_new_block_mut(block_index, element_count, next_block_index)
    }

    /// Traverse the list to find the last free block.
    /// Returns `None` if there are no free blocks.
    fn last_block(&self) -> Option<i32>
    {
        // Use first free block to start searching.
        match self.first_free_block
        {
            // There are no blocks.
            None => return None,

            // Search blocks.
            Some(..) =>
            {
                let mut current_block_index = self.first_free_block.unwrap();
                // Has to get a block, which does a non-primitive cast.
                unsafe {
                    let mut current_block = self.get_block(current_block_index);
                    loop
                    {
                        // Found the last block.
                        if !current_block.has_next_block()
                        {
                            return Some(current_block_index);
                        };
                        // Get next block.
                        current_block_index = current_block.get_next_block_index().unwrap();
                        current_block = self.get_block(current_block_index);
                    }
                }
            }
        }
    }

    /// Find the first free block that fits the size requirement.
    ///
    /// Returns a tuple that contains:
    /// 0: Index of the free block before found one.
    /// 1: Index to the block that fits.
    /// The previous block is sometimes required and this prevents having to
    /// search the list more than once.
    fn find_first_fit(&self, element_count: i32) -> (Option<i32>, Option<i32>)
    {
        // Use first free block to start searching.
        match self.first_free_block
        {
            // No free blocks, cannot search.
            None => return (None, None),

            // Search blocks.
            Some(..) =>
            {
                let mut prev_block_index = None;
                // Already checked if the first block is not none.
                let mut current_block_index = self.first_free_block.unwrap();
                // Has to access the blocks, which does a non-primitive cast.
                unsafe {
                    let mut current_block = self.get_block(current_block_index);
                    loop
                    {
                        // Found large enough block.
                        if current_block.element_count >= element_count
                        {
                            return (prev_block_index, Some(current_block_index));
                        }

                        // Check the next block.
                        match current_block.get_next_block_index()
                        {
                            // No block found.
                            None => return (prev_block_index, None),

                            // Update search.
                            Some(..) =>
                            {
                                prev_block_index = Some(current_block_index);
                                current_block_index = current_block.get_next_block_index().unwrap();
                                current_block = self.get_block(current_block_index);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get the size of the type in bytes (includes alignment).
    // This *can* be evauluated at compile-time, but is it always?
    pub const fn type_size_bytes(&self) -> i32 { size_of::<T>() as i32 }

    /// Check if the freelist has an empty (free) block.
    pub fn has_free_block(&self) -> bool { self.first_free_block != None }

    /// Get the capacity of the freelist.
    pub fn capacity_blocks(&self) -> i32 { self.heap_data.len() as i32 }

    /// Get the capacity of the freelist in bytes.
    pub fn capacity_bytes(&self) -> i32 { self.capacity_blocks() * self.type_size_bytes() }

    /// Get the number blocks currently being used.
    pub fn used_blocks(&self) -> i32 { self.used_blocks }

    /// Get the amount of memory currently used.
    pub fn used_bytes(&self) -> i32 { self.used_blocks * self.type_size_bytes() }

    /// Get the amount of free blocks.
    pub fn free_blocks(&self) -> i32 { self.capacity_blocks() - self.used_blocks() }

    /// Get the amount of free memory.
    pub fn free_bytes(&self) -> i32 { self.free_blocks() * self.type_size_bytes() }
}
