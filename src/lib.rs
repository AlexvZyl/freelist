#![allow(dead_code)]

use std::mem::{size_of, transmute};
use std::vec::Vec;

mod block;
use block::Block;

mod key;
use key::Key;

/// A cache coherent, heap allocated collection.
/// This data structure uses i32 instead of usize due to the constraints placed
/// on `Block`. It will never require 64 bit indexing.  What about smaller
/// architectures?
pub struct Freelist<T> {
    /// Pointer to the data located on the heap.
    heap_data: Vec<T>,
    /// Index to the first free block in the list.
    first_free_block: Option<usize>,
    /// The number of allocated blocks.
    used_blocks: usize,
    /// Calculates the new capacity when the freelist grows.
    calculate_new_capacity_fn: fn(current_capacity: usize, _requested_capacity: usize) -> usize,
}

impl<T> Freelist<T> {
    pub fn new() -> Self {
        // This is currently done at runtime, can it be done at compile time?
        assert!(size_of::<T>() >= size_of::<Block>());
        Freelist {
            heap_data: Vec::with_capacity(0),
            first_free_block: None,
            used_blocks: 0,
            calculate_new_capacity_fn: Freelist::<T>::calculate_new_capacity_default,
        }
    }

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
    unsafe fn allocate(&mut self, element_count: usize) {
        // Does this copy all of the data to the new vector?
        // TODO(alex): Pretty sure this is wrong
        self.heap_data.set_len(element_count);
    }

    /// Increases the amount of memory available by the specified amount.  
    /// Ensures blocks are handled corretly.
    ///
    /// # Safety.
    ///
    /// This is highly unsafe.
    ///
    /// * `Freelist::allocate()` is called.
    unsafe fn extend_by(&mut self, block_count: usize) {
        self.allocate(self.capacity_blocks() + block_count);
    }

    /// Get a mutable ref the block at the given index.
    ///
    /// # Safety
    ///
    /// This is unsafe.  
    ///
    /// * Performs a non-primitive cast.
    unsafe fn get_block_mut(&mut self, index: usize) -> &mut Block {
        Block::from_source(&mut self.heap_data[index])
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

    /// Checks if the blocks are adjacent.
    ///
    /// # Safety
    ///
    /// This is unsafe.
    ///
    /// * Performs a non-primitive cast when checknig adjacency.
    unsafe fn blocks_are_adjacent(&self, first_block_index: usize, second_block_index: usize) -> bool {
        first_block_index + self.get_block(first_block_index).get_n_elements() == second_block_index
    }

    /// Find and commit a block that fits the size requirement.
    /// If it does not find a block large enough it will resize.  The caller is
    /// guaranteed to get a block.
    fn find_and_commit_block(&mut self, element_count: usize) -> usize {
        let (prev_block_index, block_index) = self.find_first_fit(element_count);
        match block_index {
            // No blocks are large enough.
            None => {
                self.grow_capacity(element_count);
                // Search again.  This is not the most optimal way of doing it.  Will lead to
                // searching over blocks that we know are too small.
                self.find_and_commit_block(element_count)
            }

            // Commit the block.
            Some(..) => {
                self.commit_block(prev_block_index, block_index.unwrap(), element_count);
                return block_index.unwrap();
            }
        }
    }

    /// Commit the block at the index and update the blocks.
    fn commit_block(&mut self, prev_block_index: Option<usize>, block_idx: usize, element_count: usize) {
        unsafe {
            let block = self.get_block(block_idx);

            // Entire block is consumed.
            if element_count == block.get_n_elements() {
                let next_block = block.get_next_block_index();
                match prev_block_index {
                    None => self.first_free_block = next_block,

                    Some(..) => {
                        self.get_block_mut(prev_block_index.unwrap())
                            .connect_at(next_block);
                    }
                }
            }
            // Part of block is consumed.
            else if element_count < block.get_n_elements() {
                // Create new block with reduced size.
                let new_index = block_idx + element_count;
                self.new_block(
                    new_index,
                    block.get_n_elements() - element_count,
                    block.get_next_block_index(),
                );
                // Update the prev block.
                if prev_block_index != None {
                    self.get_block_mut(prev_block_index.unwrap())
                        .connect_at(Some(new_index));
                }
            }
            // TODO: Throw. Too many elements for block.
            else {
            }
        }
    }

    /// Grow the capacity based on the calculation set.
    fn grow_capacity(&mut self, requested_block_element_count: usize) {
        let current_capacity = self.capacity_blocks();
        let new_capacity =
            (self.calculate_new_capacity_fn)(current_capacity, requested_block_element_count);
        let capacity_increase = new_capacity - current_capacity;
        unsafe {
            self.allocate(new_capacity);
            // Just extend the last block's count.
            if self.is_last_block_at_end() {
                let result = self
                    .get_block_mut(self.find_last_block_index().unwrap())
                    .grow(capacity_increase);
                match result {
                    Ok(val) => {}
                    Err(val) => {}
                }
            }
            // Create new block.
            else {
                self.new_block(current_capacity, capacity_increase, None);
                let last_block_index = self.find_last_block_index();
                if last_block_index != None {
                    self.get_block_mut(last_block_index.unwrap())
                        .connect_at(Some(current_capacity));
                }
            }
        }
    }

    /// The default function used when calculating the new capacity of the
    /// freelist.
    fn calculate_new_capacity_default(current_capacity: usize, _requested_capacity: usize) -> usize {
        current_capacity + current_capacity / 2
    }

    /// Checks if the last block sits at the end of the freelist.
    /// Returns false if there are no blocks.
    fn is_last_block_at_end(&self) -> bool {
        unsafe {
            let last_block_index = self.find_last_block_index();
            match last_block_index {
                None => return false,
                Some(..) => {
                    let last_block = self.get_block(last_block_index.unwrap());
                    return last_block_index.unwrap() + last_block.get_n_elements()
                        == self.capacity_blocks();
                }
            }
        }
    }

    /// Connect two blocks based on the index.
    ///
    /// # Safety.
    ///
    /// This is unsafe.
    ///
    /// * Non-primitive casts are performed when getting the blocks.
    unsafe fn connect_blocks(&mut self, first_block_index: usize, second_block_index: usize) {
        self.get_block_mut(first_block_index)
            .connect_at(Some(second_block_index));
    }

    /// Create a new block at the index with the given values and return a
    /// mutable reference.
    ///
    /// # Safety.
    ///
    /// This is unsafe.
    ///
    /// * Data is being written directly to a region of memory.
    unsafe fn new_block_mut(
        &mut self,
        block_index: usize,
        element_count: usize,
        next_block_index: Option<usize>,
    ) -> &mut Block {
        Block::from_source_with_parts(
            &mut self.heap_data[block_index],
            element_count,
            next_block_index
        )
    }

    /// Create a new block at the index with the given values and return a const
    /// reference.
    ///
    /// # Safety.
    ///
    /// This is unsafe.
    ///
    /// * Data is being written directly to a region of memory.
    // Is puting the mut version of the function inside of the constant version good
    // practice?
    unsafe fn new_block(
        &mut self,
        block_index: usize,
        element_count: usize,
        next_block_index: Option<usize>,
    ) -> &Block {
        self.new_block_mut(block_index, element_count, next_block_index)
    }

    /// Traverse the list to find the last free block.
    /// Returns `None` if there are no free blocks.
    // Should I rather keep track of the last block instead of searching for it?
    // Will depend on how much this function is called...
    fn find_last_block_index(&self) -> Option<usize> {
        // Use first free block to start searching.
        match self.first_free_block {
            // There are no blocks.
            None => return None,

            // Search blocks.
            Some(..) => {
                let mut current_block_index = self.first_free_block.unwrap();
                // Has to get a block, which does a non-primitive cast.
                unsafe {
                    let mut current_block = self.get_block(current_block_index);
                    loop {
                        // Found the last block.
                        if !current_block.has_next_block() {
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
    fn find_first_fit(&self, element_count: usize) -> (Option<usize>, Option<usize>) {
        // Use first free block to start searching.
        match self.first_free_block {
            // No free blocks, cannot search.
            None => return (None, None),

            // Search blocks.
            Some(..) => {
                let mut prev_block_index = None;
                // Already checked if the first block is not none.
                let mut current_block_index = self.first_free_block.unwrap();
                // Has to access the blocks, which does a non-primitive cast.
                unsafe {
                    let mut current_block = self.get_block(current_block_index);
                    loop {
                        // Found large enough block.
                        if current_block.get_n_elements() >= element_count {
                            return (prev_block_index, Some(current_block_index));
                        }

                        // Check the next block.
                        match current_block.get_next_block_index() {
                            // No block found.
                            None => return (prev_block_index, None),

                            // Update search.
                            Some(..) => {
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

    /// If the two block are adjacent, merge them.  Returns true if the merge
    /// occured.
    fn attempt_merge(&mut self, first_block_index: usize, second_block_index: usize) -> bool {
        unsafe {
            if self.blocks_are_adjacent(first_block_index, second_block_index) {
                let next_block_index = self.get_block(second_block_index).get_next_block_index();
                let second_block_count = self.get_block(second_block_index).get_n_elements();
                let first_block = self.get_block_mut(first_block_index);
                first_block.connect_at(next_block_index);
                let result = first_block.grow(second_block_count);
                match result {
                    Err(val) => {}
                    Ok(val) => {}
                }
                return true;
            }
        }
        return false;
    }

    /// Get the size of the type in bytes (includes alignment).
    // This *can* be evauluated at compile-time, but is it always?
    pub const fn type_size_bytes(&self) -> usize {
        size_of::<T>()
    }

    /// Check if the freelist has an empty (free) block.
    pub fn has_free_block(&self) -> bool {
        self.first_free_block != None
    }

    /// Get the capacity of the freelist.
    pub fn capacity_blocks(&self) -> usize {
        self.heap_data.capacity()
    }

    /// Get the capacity of the freelist in bytes.
    pub fn capacity_bytes(&self) -> usize {
        self.capacity_blocks() * self.type_size_bytes()
    }

    /// Get the number blocks currently being used.
    pub fn used_blocks(&self) -> usize {
        self.used_blocks
    }

    /// Get the amount of memory currently used.
    pub fn used_bytes(&self) -> usize {
        self.used_blocks * self.type_size_bytes()
    }

    /// Get the amount of free blocks.
    pub fn free_blocks(&self) -> usize {
        self.capacity_blocks() - self.used_blocks()
    }

    /// Get the amount of free memory.
    pub fn free_bytes(&self) -> usize {
        self.free_blocks() * self.type_size_bytes()
    }

    /// Ensure the freelist has enough capcity for the requested amount of
    /// elements.
    // TODO(alex): Last block should increase.
    pub fn reserve_exact(&mut self, total_elements: usize) {
        self.heap_data.reserve_exact(total_elements)
    }
}
