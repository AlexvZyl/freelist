#[cfg(test)]
mod tests {

    // Struct used for testing.
    struct Entity {
        _a: i32,
        _b: i32,
        _c: i32
    }

    use std::mem;

    use freelist::Freelist;

    #[test]
    fn default_constructor() {
        let fl = Freelist::<Entity>::new();
        assert_eq!(fl.capacity_blocks(), 0);
        assert_eq!(fl.used_blocks(), 0);
        assert_eq!(fl.free_blocks(), 0);
        assert_eq!(fl.type_size_bytes(), mem::size_of::<Entity>() as i32);
    }
    
    #[test]
    #[should_panic]
    fn type_too_small() {
        Freelist::<u32>::new();
    }

    #[test]
    fn reserve_exact() {
        let mut fl = Freelist::<Entity>::new();
        fl.reserve_exact(100);
        assert_eq!(fl.capacity_blocks(), 100);
        fl.reserve_exact(150);
        assert_eq!(fl.capacity_blocks(), 150);
    }
}
