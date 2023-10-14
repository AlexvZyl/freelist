#[cfg(test)]
mod tests {

    use std::mem;
    use freelist::Freelist;

    struct Entity {
        _a: i32,
        _b: i32,
        _c: i32,
    }

    struct SmallEntity {
        _a: i16,
        _b: i16,
        _c: i16
    }

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
        Freelist::<SmallEntity>::new();
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
