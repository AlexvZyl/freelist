#[cfg(test)]
mod tests {

    use freelist::Freelist;

    #[test]
    fn start_empty() {
        let fl = Freelist::<u64>::new();
        assert_eq!(fl.capacity_blocks(), 0)
    }
    
    #[test]
    #[should_panic]
    fn type_too_small() {
        Freelist::<i32>::new();
    }
}
