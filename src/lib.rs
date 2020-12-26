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
    pub use super::cfgblock;
    pub use super::defer;

    pub use super::core::*;
    pub use super::errors::*;
    pub use super::sys::{self, exec, ext::*, user};

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

/// Ensure the given closure is executed once the surrounding scope closes despite panics.
/// Inspired by Golang's `defer`, Java's finally and Ruby's `ensure`.
///
/// This provides a mechanism similar to Golang's `defer` that will trigger when the
/// surrounding function goes out of scope.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("core_finally_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
///
/// // Create a scope that will trigger finally's destructor
/// {
///     defer!(sys::remove_all(&tmpdir).unwrap());
/// }
/// assert_eq!(tmpdir.exists(), false);
/// ```
#[macro_export]
macro_rules! defer {
    ($($tokens:tt)*) => {
        let _defer = defer(|| { $($tokens)* });
    };
}

/// Provides the ability to define `#[cfg]` statements for multiple items
///
/// ### Examples
/// ```ignore
/// use fungus::prelude::*;
///
/// cfgblk! {
///     #[cfg(feature = "foo")])
///     use libc;
///     use std::ffi::CString;
/// }
///
/// // Expands to
/// #[cfg(feature = "foo")])
/// use libc;
/// #[cfg(feature = "foo")])
/// use std::ffi::CString;
/// ```
#[macro_export]
macro_rules! cfgblock {

    // Handle a single item
    (#[$attr:meta] $item:item) => {
        #[$attr] $item
    };

    // Handle more than one item recursively
    (#[$attr:meta] $($tail:item)*) => {
        $(cfgblock!{#[$attr] $tail})*
    };
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::cell::Cell;

    #[test]
    fn test_defer_macro() {
        let obj = Cell::new(1);
        defer!(obj.set(2));
        assert_eq!(1, obj.get());
    }
}
