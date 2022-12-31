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
        print!("Dropping!");
    }
}

fn main()
{
    let fl = Freelist::<Test>::new();
    print!("{}", fl.capacity_bytes());
}
