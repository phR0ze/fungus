mod file;
mod path;
pub mod user;

/// Import sys module
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let home = env::var("HOME").unwrap();
/// assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
/// ```
pub mod presys {
    pub use crate::path::PathExt;
    pub mod sys {
        pub use crate::file::*;
        pub use crate::path::*;
    }
    pub use std::env;
    pub use std::ffi::OsStr;
    pub use std::fs;
    pub use std::os::unix::fs::PermissionsExt;
    pub use std::path::{Component, Path, PathBuf};
}
