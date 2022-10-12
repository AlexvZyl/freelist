#![allow(dead_code)]

use freelist::Freelist;

struct Test
{
    a: i32,
    b: i32,
}

impl Drop for Test
{
    fn drop(&mut self)
    {
        print!("Ddropping!");
    }
}

fn main()
{
    println!("{}", std::mem::size_of::<Option<i32>>());
    let fl = Freelist::<Test>::new();
    print!("{}", fl.type_size_bytes());
}
