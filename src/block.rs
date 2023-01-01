/// Used to describe an open block in the freelist.
/// A block can consist of many elements, if they are contiguous.
// This struct does not use `usize` since I want to force it to
// be 8 bytes (64 bits).
// I want to use `Option` in this type, but it uses 8 bytes instead of 4.
// Instead, using an API to get this functionality.  Is there a significant
// overhead?
pub struct Block
{
    /// How many elements can be fit into the block.
    /// (A block can consist of many contiguous elements)
    // Should this be manipulated via an API instead of directly?
    pub element_count: i32,
    /// Index to the next free block in the freelist.
    next_block_index: i32,
}

// Block implementations.
impl Block
{
    /// Checks if the block has a block that sits after it.
    pub fn has_next_block(&self) -> bool { self.next_block_index != -1 }

    /// Checks if the block is empty.
    /// The block should be removed from the freelist if this is true.
    pub fn is_empty(&self) -> bool { self.element_count == 0 }

    /// Get the index of the next block.
    // Using an API to make sure we keep the size at 8 bytes.
    pub fn get_next_block_index(&self) -> Option<i32>
    {
        match self.next_block_index
        {
            -1 => None,
            _ => Some(self.next_block_index),
        }
    }

    /// Connect the block to the block at `block_index`.  This basically just
    /// changes `next_block_index`, since blocks do not have references to
    /// the previous block.
    pub fn connect(&mut self, block_index: Option<i32>)
    {
        match block_index
        {
            None => self.set_next_block_none(),
            Some(..) => self.next_block_index = block_index.unwrap(),
        }
    }

    /// Set the block to have no next block.
    pub fn set_next_block_none(&mut self) { self.next_block_index = -1; }

    /// Set the next block index of the current block.
    pub fn set_next_block_index(&mut self, next_block_index: Option<i32>)
    {
        match next_block_index
        {
            None => self.next_block_index = -1,
            Some(..) => self.next_block_index = next_block_index.unwrap(),
        }
    }
}
