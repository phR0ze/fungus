use fungus::prelude::*;

fn main() {
    let _defer = defer(|| println!("should print after"));
    println!("should print before defer");
}
