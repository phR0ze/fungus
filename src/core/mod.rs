// macro import has to happend before other modules
#[macro_use]
pub mod macros;

mod arch_error;
mod file_error;
mod finally;
mod git_error;
mod iter;
mod iter_error;
mod logger;
mod miscs;
mod option;
mod os_error;
mod path_error;
mod result;
mod user_error;

// Exports
pub use arch_error::*;
pub use file_error::*;
pub use finally::*;
pub use git_error::*;
pub use iter::*;
pub use iter_error::*;
pub use logger::*;
pub use macros::*;
pub use miscs::*;
pub use option::*;
pub use os_error::*;
pub use path_error::*;
pub use result::*;
pub use user_error::*;

// Re-exports
pub use log::{debug, error, info, log, trace, warn};
