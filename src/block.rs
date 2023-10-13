/// Used to describe an open block in the freelist.
/// A block can consist of many elements, if they are contiguous.
// This struct does not use `usize` since I want to force it to
// be 8 bytes (64 bits).
// I want to use `Option` in this type, but it uses 8 bytes instead of 4.
// Instead, using an API to get this functionality.  Is there a significant
// overhead?  Or a better way to do this?
pub struct Block {
    /// How many elements are (or can fit) in(to) the block.
    /// (A block can consist of many contiguous elements)
    n_elements: i32,
    /// Index to the next free block.
    next_block_index: i32,
}

const NONE_INT :i32 = -1;

impl Block {
    pub fn get_n_elements(&self) -> i32 { 
        self.n_elements 
    }

    pub fn set_n_elements(&mut self, n_elements: i32) {
        self.n_elements = n_elements
    }

    /// Returns the new number of elements.
    /// Errors if the block overlaps with the following block.
    pub fn grow(&mut self, increase: i32) -> Result<i32, i32> {
        let new_cap = self.n_elements + increase;
        if self.has_next_block() && (new_cap >= self.next_block_index) {
            return Err(new_cap)
        }
        self.n_elements = new_cap;
        Ok(new_cap)
    }

    /// Returns the new number of elements.
    /// Errors if the value is shrunk to or below 0.
    pub fn shrink(&mut self, decrease: i32) -> Result<i32, i32> { 
        let new_cap = self.n_elements - decrease;
        if new_cap <= 0 {
            return Err(new_cap)
        }
        self.n_elements = new_cap;
        Ok(new_cap)
    }

    pub fn has_next_block(&self) -> bool { 
        self.next_block_index != NONE_INT
    }

    /// The block should be removed from the freelist if this is true.
    pub fn is_empty(&self) -> bool { 
        self.n_elements == 0 
    }

    pub fn get_next_block_index(&self) -> Option<i32> {
        match self.next_block_index {
            NONE_INT => None,
            _ => Some(self.next_block_index),
        }
    }

    /// This basically just changes `next_block_index`, since blocks 
    /// do not have references to the previous block (for now?).
    pub fn connect_at(&mut self, block_index: Option<i32>) {
        self.next_block_index = block_index.unwrap_or_else(|| NONE_INT) 
    }
}
