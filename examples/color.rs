use fungus::prelude::*;

fn main() {
    println!("{}", sys::env::flag("TERM_COLOR", true));
    println!("{}", sys::env::isatty().to_string());
    println!("{}", sys::term::is_color().to_string());
}
