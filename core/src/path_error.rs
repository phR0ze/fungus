use std::fmt;
use std::path::{Path, PathBuf};

use crate::error::*;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathError {
    /// An error indicating that the desired path component is not found.
    ComponentNotFound(PathBuf),

    /// An error indicating that the path is empty.
    Empty,

    /// An error indicating a failure to convert the path to a string.
    FailedToString(PathBuf),

    /// An error indicating that the path does not contain a filename.
    FileNameNotFound(PathBuf),

    /// An error indicating that the path failed to expand properly.
    InvalidExpansion(PathBuf),

    /// An error indicating that the path contains multiple user home symbols i.e. tilda.
    MultipleHomeSymbols(PathBuf),

    /// An error indicating that the path does not have a valid parent path.
    ParentNotFound(PathBuf),
}
impl PathError {
    /// Return an error indicating that the desired path component is not found
    pub fn component_not_found<T: AsRef<Path>>(path: T) -> Error {
        Error::from(PathError::ComponentNotFound(path.as_ref().to_path_buf()))
    }

    /// Return an error indicating that the path is empty
    pub fn empty() -> Error {
        Error::from(PathError::Empty)
    }

    /// Return an error indicating a failure to convert the path to a string
    pub fn failed_to_string<T: AsRef<Path>>(path: T) -> Error {
        Error::from(PathError::FailedToString(path.as_ref().to_path_buf()))
    }

    /// Return an error indicating that the path does not contain a filename
    pub fn filename_not_found<T: AsRef<Path>>(path: T) -> Error {
        Error::from(PathError::FileNameNotFound(path.as_ref().to_path_buf()))
    }

    /// Return an error indicating that the path failed to expand properly
    pub fn invalid_expansion<T: AsRef<Path>>(path: T) -> Error {
        Error::from(PathError::InvalidExpansion(path.as_ref().to_path_buf()))
    }

    /// Return an error indicating that the path contains multiple user home symbols i.e. tilda
    pub fn multiple_home_symbols<T: AsRef<Path>>(path: T) -> Error {
        Error::from(PathError::MultipleHomeSymbols(path.as_ref().to_path_buf()))
    }

    /// Return an error indicating that the path does not have a valid parent path
    pub fn parent_not_found<T: AsRef<Path>>(path: T) -> Error {
        Error::from(PathError::ComponentNotFound(path.as_ref().to_path_buf()))
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PathError::ComponentNotFound(ref path) => write!(f, "component not found for path {}", path.display()),
            PathError::Empty => write!(f, "path empty"),
            PathError::FailedToString(ref path) => write!(f, "failed to convert to string for path {}", path.display()),
            PathError::FileNameNotFound(ref path) => write!(f, "filename not found for path {}", path.display()),
            PathError::InvalidExpansion(ref path) => write!(f, "invalid path expansion for path {}", path.display()),
            PathError::MultipleHomeSymbols(ref path) => write!(f, "multiple home symbols for path {}", path.display()),
            PathError::ParentNotFound(ref path) => write!(f, "parent not found for path {}", path.display()),
        }
    }
}

impl From<PathError> for Error {
    fn from(err: PathError) -> Error {
        Error::from(ErrorKind::Path(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn path_empty() -> Result<PathBuf> {
        Err(PathError::empty())
    }

    #[test]
    fn test_new_path_empty() {
        assert!(path_empty().is_err());
    }

    #[test]
    fn test_backtrace() {
        let err = path_empty().unwrap_err();
        println!("{:?}", err);
    }
}
