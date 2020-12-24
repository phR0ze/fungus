// macro import has to happend before other modules
#[macro_use]
pub mod macros;

mod finally;
mod iter;
mod option;
mod string;

// Export contents of modules into core
pub use finally::*;
pub use iter::*;
pub use option::*;
pub use string::*;
