mod file;
mod iter;
mod iter_error;
mod libc;
mod miscs;
mod option;
mod os;
mod path;
mod path_error;
mod result;
mod user_error;

/// Export user module
pub mod user;

/// Export core module
pub mod core {
    pub use crate::iter::*;
    pub use crate::iter_error::*;
    pub use crate::miscs::*;
    pub use crate::option::*;
    pub use crate::path_error::*;
    pub use crate::result::*;
    pub use crate::user_error::*;
}

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
        pub use crate::os::*;
        pub use crate::path::*;
    }

    pub use std::env;
    pub use std::ffi::OsStr;
    pub use std::fs::{self, File};
    pub use std::io::{self, prelude::*, BufRead, BufReader};
    pub use std::os::unix::fs::{MetadataExt, PermissionsExt};
    pub use std::path::{Component, Path, PathBuf};
    pub use std::str;
}
