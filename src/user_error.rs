use failure::Fail;
use std::fmt;
use std::path::{Path, PathBuf};

// An error indicating that something went wrong with a user operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Fail)]
pub enum UserError {
    /// An error indicating that the use does not exist.
    DoesNotExistById(u32),

    /// An error indicating a failure to convert the user name to a string.
    FailedToString(u32),
}
impl UserError {
    /// Return an error indicating that the user does not exist
    pub fn does_not_exist_by_id(uid: u32) -> UserError {
        UserError::DoesNotExistById(uid)
    }

    /// Return an error indicating a failure to convert the user name to a string
    pub fn failed_to_string(uid: u32) -> UserError {
        UserError::FailedToString(uid)
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserError::DoesNotExistById(ref uid) => write!(f, "user does not exist: {}", uid),
            UserError::FailedToString(ref uid) => write!(f, "failed to convert to string for user: {}", uid),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_errors() {
        assert_eq!(UserError::does_not_exist_by_id(1000), UserError::DoesNotExistById(1000));
        assert_eq!(format!("{}", UserError::DoesNotExistById(1000)), "user does not exist: 1000");
        assert_eq!(UserError::failed_to_string(1000), UserError::FailedToString(1000));
        assert_eq!(format!("{}", UserError::FailedToString(1000)), "failed to convert to string for user: 1000");
    }
}
