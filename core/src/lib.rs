// Documentation for crate

mod iter;
mod iter_error;
mod miscs;
mod option;
mod path_error;
mod result;
pub use iter::*;
pub use iter_error::*;
pub use miscs::*;
pub use option::*;
pub use path_error::*;
pub use result::*;

// Get traits and other top level namespace entries
pub mod preamble {
    use super::*;
    pub use iter::*;
    pub use iter_error::*;
    pub use miscs::*;
    pub use option::*;
    pub use path_error::*;
    pub use result::*;
}
