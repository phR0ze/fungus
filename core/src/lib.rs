// Documentation for crate

mod error;
mod iter;
mod iter_error;
mod miscs;
mod option;
mod path_error;
mod result;
mod sys_error;
pub use error::*;
pub use iter::*;
pub use iter_error::*;
pub use miscs::*;
pub use option::*;
pub use path_error::*;
pub use result::*;
pub use sys_error::*;

// Get traits and other top level namespace entries
pub mod preamble {
    use super::*;
    pub use error::*;
    pub use iter::*;
    pub use iter_error::*;
    pub use miscs::*;
    pub use option::*;
    pub use path_error::*;
    pub use result::*;
    pub use sys_error::*;
}
