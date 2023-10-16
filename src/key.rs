/// Represents an index into the freelist.  Contains an ID for the fragmentation state
/// for when defragmentation will be implemented.
// TODO(alex): Well, everything...
pub struct Key {
    index: usize,
    state_id: usize
}
