mod file;
mod file_error;
mod iter;
mod iter_error;
mod libc;
mod logger;
mod miscs;
mod option;
mod os;
mod os_error;
mod path;
mod path_error;
mod result;
mod user_error;

pub mod agent;
pub mod exec;
pub mod user;

/// Export core module
pub mod core {
    pub use crate::file_error::*;
    pub use crate::iter::*;
    pub use crate::iter_error::*;
    pub use crate::logger::*;
    pub use crate::miscs::*;
    pub use crate::option::*;
    pub use crate::os_error::*;
    pub use crate::path_error::*;
    pub use crate::result::*;
    pub use crate::user_error::*;
    pub use log::{debug, error, info, log, trace, warn};
}

/// Import sys module. Includes core::*, exec, and user modules.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let home = env::var("HOME").unwrap();
/// assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
/// ```
pub mod prelude {

    // system
    //----------------------------------------------------------------------------------------------
    pub use crate::core::*;

    // exec
    pub use crate::exec;

    // sys
    pub use crate::path::PathExt;
    pub mod sys {
        pub use crate::file::*;
        pub use crate::os::*;
        pub use crate::path::*;
    }

    // user
    pub use crate::user;

    // Re-exports for sys
    pub use lazy_static::*;
    pub use regex::Regex;
    pub use std::env;
    pub use std::ffi::OsStr;
    pub use std::fs::{self, File, OpenOptions};
    pub use std::io::{self, prelude::*, BufRead, BufReader};
    pub use std::os::unix::fs::{MetadataExt, PermissionsExt};
    pub use std::path::{Component, Path, PathBuf};
    pub use std::str;

    // network namespace
    //----------------------------------------------------------------------------------------------

    // agent
    pub use crate::agent;

    // net
    pub mod net {}
}
