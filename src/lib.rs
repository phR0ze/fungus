#[macro_use]
pub mod core;
pub mod errors;
pub mod sys;

/// Types exported directly into the fungus namespace
pub use crate::errors::Error;
pub use crate::errors::Result;

/// All essential symbols in a simple consumable way
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
/// ```
pub mod prelude {
    pub use super::core::*;
    pub use super::errors::*;
    pub use super::sys::{self, exec, ext::*, term, user};

    // Re-exports
    //----------------------------------------------------------------------------------------------
    pub use gory::*;
    pub use lazy_static::*;
    pub use libc;
    pub use regex::Regex;
    pub use std::ffi::{OsStr, OsString};
    pub use std::fmt;
    pub use std::fs::{self, File, OpenOptions};
    pub use std::io::{self, prelude::*, BufRead, BufReader};
    pub use std::os::unix::fs::{MetadataExt, PermissionsExt};
    pub use std::path::Component;
    pub use std::path::{Path, PathBuf};
    pub use std::str;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
