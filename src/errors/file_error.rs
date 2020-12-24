use std::error::Error as StdError;
use std::fmt;

// An error indicating that something went wrong with a file operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FileError {
    /// An error indicating that a regex string extraction failed.
    FailedToExtractString,
}
impl FileError {
    /// Implemented directly on the `Error` type to reduce casting required
    pub fn is<T: StdError + 'static>(&self) -> bool {
        <dyn StdError + 'static>::is::<T>(self)
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn downcast_ref<T: StdError + 'static>(&self) -> Option<&T> {
        <dyn StdError + 'static>::downcast_ref::<T>(self)
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn downcast_mut<T: StdError + 'static>(&mut self) -> Option<&mut T> {
        <dyn StdError + 'static>::downcast_mut::<T>(self)
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.as_ref().source()
    }
}

impl StdError for FileError {}

impl AsRef<dyn StdError> for FileError {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        self
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FileError::FailedToExtractString => write!(f, "failed to extract string from file"),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::prelude::*;

//     #[test]
//     fn test_errors() {}
// }
