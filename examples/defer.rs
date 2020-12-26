use fungus::prelude::*;

fn main() {
    defer!(println!("should print after"));
    println!("should print before");
}
