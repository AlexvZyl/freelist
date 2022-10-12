/// Used to describe an open block in freelist::Freelist<T>.
/// A block can consist of many blocks, if they are contiguous.
/// This struct does not use `usize` since I want to force it to
/// be 8 bytes (64 bits).
pub struct Block
{
    /// How many of `T` can be fit into the current block.
    /// (A block can consist of many contiguous blocks)
    pub count: i32,
    /// Index to the next free block in the freelist.
    // I want to use `Option` here, but it uses 8 bytes insread of 4.
    pub next_block_index: i32
}

// Block implementations.
impl Block 
{
    /// Checks if the block has a block that sits after it.
    pub fn has_next_block(&self) -> bool
    {
        self.next_block_index != -1
    }
}
