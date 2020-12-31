use std::{error::Error as StdError, fmt};

// An error indicating that something went wrong with a file operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StringError {
    /// An error indicating a failure to convert the file value to a string.
    FailedToString,
}

impl StdError for StringError {}

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
