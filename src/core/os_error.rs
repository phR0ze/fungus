use failure::Fail;
use std::fmt;

// An error indicating that something went wrong with an os operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Fail)]
pub enum OsError {
    /// An error indicating that the kernel release was not found.
    KernelReleaseNotFound,

    /// An error indicating that the kernel version was not found.
    KernelVersionNotFound,
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
    use crate::core::*;

    #[test]
    fn test_errors() {
        assert_eq!(format!("{}", OsError::KernelReleaseNotFound), "kernel release was not found");
        assert_eq!(format!("{}", OsError::KernelVersionNotFound), "kernel version was not found");
    }
}
