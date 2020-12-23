pub mod sys;

/// Import all essential symbols in a simple consumable way
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
/// ```
pub mod prelude {
    pub use super::sys::{self};

    // Re-exports
    //----------------------------------------------------------------------------------------------
    pub use lazy_static::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
