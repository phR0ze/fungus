use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt;

use crate::error::*;

/// An error indicating that something went wrong with an IO operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EnvError {
    /// The specified environment variable was not present in the current
    /// process's environment.
    NotPresent(String),

    /// The specified environment variable was found, but it did not contain
    /// valid unicode data.
    NotUnicode { key: String, value: OsString },
}
impl EnvError {}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EnvError::NotPresent(ref key) => write!(f, "io error"),
            EnvError::NotUnicode { ref key, ref value } => write!(f, "io error"),
        }
    }
}

impl From<EnvError> for Error {
    fn from(err: EnvError) -> Error {
        Error::from(ErrorKind::Env(err))
    }
}

impl From<env::VarError> for EnvError {
    fn from(err: env::VarError) -> EnvError {
        match err {
            env::VarError::NotPresent => EnvError::NotPresent(String::from("foo")),
            env::VarError::NotUnicode(_) => EnvError::NotUnicode { key: String::from("foo"), value: OsString::from("blah") },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use failure::Fail;
    use std::path::PathBuf;

    // when the iter error gets converted to a Result<i32, Error> it will get the failure goodness
    // fn Io_error_result() -> Result<i32> {
    //     let path = PathBuf::new().as_path();
    // }

    // #[test]
    // fn test_backtrace() {
    //     let err = iter_error_result().unwrap_err();
    //     println!("{:?}", err)
    // }

    // #[test]
    // fn test_iter_error_display() {
    //     assert_eq!("iterator item not found", format!("{}", IterError::item_not_found().kind()));
    // }

    // #[test]
    // fn test_matching_error() {
    //     if let ErrorKind::Iter(err) = iter_error_result().unwrap_err().kind() {
    //         assert_eq!(&IterError::ItemNotFound, err);
    //     } else {
    //         panic!();
    //     }
    // }
}
