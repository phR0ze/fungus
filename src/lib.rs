pub mod core;
pub mod sys;

/// Types exported directly into the fungus namespace
pub use crate::core::Error;
pub use crate::core::Result;

/// All essential symbols in a simple consumable way
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
/// ```
pub mod prelude {
    pub use super::core::*;
    pub use super::sys::{self, exec, ext::*, user};

    // Re-exports
    //----------------------------------------------------------------------------------------------
    pub use lazy_static::*;
    pub use regex::Regex;
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
