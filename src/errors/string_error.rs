use std::error::Error as StdError;
use std::fmt;

// An error indicating that something went wrong with a file operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StringError {
    /// An error indicating a failure to convert the file value to a string.
    FailedToString,
}
impl StringError {
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

impl StdError for StringError {}

impl AsRef<dyn StdError> for StringError {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        self
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StringError::FailedToString => write!(f, "failed to convert value to string"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::*;

    #[test]
    fn test_errors() {
        assert_eq!(format!("{}", StringError::FailedToString), "failed to convert value to string");
    }
}
