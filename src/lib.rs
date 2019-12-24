mod file;
mod path;
mod users;
pub use file::files::*;
pub use path::paths::*;

/// Import traits and other top level namespace entities.
///
/// ### Examples
/// ```
/// use std::env;
/// use std::path::PathBuf;
/// use sys::preamble::*;
///
/// let home = env::var("HOME").unwrap();
/// assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
/// ```
pub mod preamble {
    use super::*;
    pub use path::PathExt;
    pub use std::env;
    pub use std::ffi::OsStr;
    pub use std::fs;
    pub use std::os::unix::fs::PermissionsExt;
    pub use std::path::{Component, Path, PathBuf};
    pub use users::*;
}
