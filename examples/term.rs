use fungus::prelude::*;

fn main() {
    println!("{}", sys::flag(TERM_COLOR));
    println!("{}", sys::flag_default(TERM_COLOR, true));
    println!("{}", sys::hastty().to_string());
}
