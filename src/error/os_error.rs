use std::error::Error as StdError;
use std::fmt;

// An error indicating that something went wrong with an os operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OsError {
    /// An error indicating that the kernel release was not found.
    KernelReleaseNotFound,

    /// An error indicating that the kernel version was not found.
    KernelVersionNotFound,
}
impl OsError {
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

impl StdError for OsError {}

impl AsRef<dyn StdError> for OsError {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        self
    }
}

impl fmt::Display for OsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OsError::KernelReleaseNotFound => write!(f, "kernel release was not found"),
            OsError::KernelVersionNotFound => write!(f, "kernel version was not found"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_errors() {
        assert_eq!(format!("{}", OsError::KernelReleaseNotFound), "kernel release was not found");
        assert_eq!(format!("{}", OsError::KernelVersionNotFound), "kernel version was not found");
    }
}
