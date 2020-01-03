// macro import has to happend before other modules
#[macro_use]
pub mod core;

mod file;
mod libc;
mod os;
mod path;

pub mod exec;
pub mod net;
pub mod user;

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
    pub use crate::exec;

    pub use crate::path::PathExt;
    pub mod sys {
        pub use crate::file::*;
        pub use crate::os::*;
        pub use crate::path::*;
    }
    pub use crate::user;

    // Re-exports for sys
    pub use chrono;
    pub use colored::*;
    pub use lazy_static::*;
    pub use log;
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
    pub use crate::net;
    pub use crate::net::agent;
}
