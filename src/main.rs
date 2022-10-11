use freelist::Freelist;

fn main() {
    let fl = Freelist::<i32>::new();
    print!("{}", fl.type_size());
}
