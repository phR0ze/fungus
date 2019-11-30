use std::error::Error as ErrorTrait;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::error::*;

/// An error indicating that something went wrong with an IO operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IoError {
    /// An error coming up from the io package
    System { kind: io::ErrorKind, desc: String },
}
impl IoError {}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IoError::System { ref kind, ref desc } => write!(f, "io error"),
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::from(ErrorKind::Io(err))
    }
}

impl From<io::Error> for IoError {
    fn from(err: io::Error) -> IoError {
        IoError::System { kind: err.kind(), desc: err.description().to_string() }
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
