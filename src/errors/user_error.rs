use std::{error::Error as StdError, fmt};

// An error indicating that something went wrong with a user operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UserError {
    /// An error indicating that the use does not exist.
    DoesNotExistById(u32),
}
impl UserError {
    /// Return an error indicating that the user does not exist
    pub fn does_not_exist_by_id(uid: u32) -> UserError {
        UserError::DoesNotExistById(uid)
    }
}

impl StdError for UserError {}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserError::DoesNotExistById(ref uid) => write!(f, "user does not exist: {}", uid),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_errors() {
        assert_eq!(UserError::does_not_exist_by_id(1000), UserError::DoesNotExistById(1000));
        assert_eq!(format!("{}", UserError::DoesNotExistById(1000)), "user does not exist: 1000");
    }
}
