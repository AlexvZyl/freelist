use std::mem;
use std::vec::Vec;

pub struct Freelist<T> {

    _type_size: usize,
    pub _data: Vec<T>

}

impl<T> Freelist<T> {

    pub fn new() -> Self {
        Freelist {
            _type_size: mem::size_of::<T>(),
            _data: Vec::new()
        } 
    }

    pub fn type_size(&self) -> usize {
        self._type_size
    }

}
