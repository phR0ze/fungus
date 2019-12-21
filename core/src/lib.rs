/// Documentation for crate
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

/// Import traits and other top level namespace entries.
/// Including preamble to have pareity with other libs in rs but its not required, you can
/// import using `use rs::core::*;` just fine.
pub mod preamble {
    pub use super::*;
}
