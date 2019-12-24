use failure::Fail;
use std::fmt;
use std::path::{Path, PathBuf};

// An error indicating that something went wrong with a path operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Fail)]
pub enum PathError {
    /// An error indicating that the path does not exist.
    DoesNotExist(PathBuf),

    /// An error indicating that the path is empty.
    Empty,

    /// An error indicating that the path exists already.
    ExistsAlready(PathBuf),

    /// An error indicating a failure to convert the path to a string.
    FailedToString(PathBuf),

    /// An error indicating that the path does not contain a filename.
    FileNameNotFound(PathBuf),

    /// An error indicating that the path failed to expand properly.
    InvalidExpansion(PathBuf),

    /// An error indicating that the path is not a directory.
    IsNotDir(PathBuf),

    /// An error indicating that the path is not a file.
    IsNotFile(PathBuf),

    /// An error indicating that the path is not a file or symlink to a file.
    IsNotFileOrSymlinkToFile(PathBuf),

    /// An error indicating that the path contains multiple user home symbols i.e. tilda.
    MultipleHomeSymbols(PathBuf),

    /// An error indicating that the path does not have a valid parent path.
    ParentNotFound(PathBuf),
}
impl PathError {
    /// Return an error indicating that the path does not exist
    pub fn does_not_exist<T: AsRef<Path>>(path: T) -> PathError {
        PathError::DoesNotExist(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path is empty
    pub fn empty() -> PathError {
        PathError::Empty
    }

    /// Return an error indicating that the path exists already
    pub fn exists_already<T: AsRef<Path>>(path: T) -> PathError {
        PathError::ExistsAlready(path.as_ref().to_path_buf())
    }

    /// Return an error indicating a failure to convert the path to a string
    pub fn failed_to_string<T: AsRef<Path>>(path: T) -> PathError {
        PathError::FailedToString(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path does not contain a filename
    pub fn filename_not_found<T: AsRef<Path>>(path: T) -> PathError {
        PathError::FileNameNotFound(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path is not a directory
    pub fn is_not_dir<T: AsRef<Path>>(path: T) -> PathError {
        PathError::IsNotDir(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path is not a file
    pub fn is_not_file<T: AsRef<Path>>(path: T) -> PathError {
        PathError::IsNotFile(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path is not a file or symlink to file
    pub fn is_not_file_or_symlink_to_file<T: AsRef<Path>>(path: T) -> PathError {
        PathError::IsNotFileOrSymlinkToFile(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path failed to expand properly
    pub fn invalid_expansion<T: AsRef<Path>>(path: T) -> PathError {
        PathError::InvalidExpansion(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path contains multiple user home symbols i.e. tilda
    pub fn multiple_home_symbols<T: AsRef<Path>>(path: T) -> PathError {
        PathError::MultipleHomeSymbols(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path does not have a valid parent path
    pub fn parent_not_found<T: AsRef<Path>>(path: T) -> PathError {
        PathError::ParentNotFound(path.as_ref().to_path_buf())
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PathError::DoesNotExist(ref path) => write!(f, "path does not exist {}", path.display()),
            PathError::Empty => write!(f, "path empty"),
            PathError::ExistsAlready(ref path) => write!(f, "path exists already {}", path.display()),
            PathError::FailedToString(ref path) => write!(f, "failed to convert to string for path {}", path.display()),
            PathError::FileNameNotFound(ref path) => write!(f, "filename not found for path {}", path.display()),
            PathError::InvalidExpansion(ref path) => write!(f, "invalid path expansion for path {}", path.display()),
            PathError::IsNotDir(ref path) => write!(f, "is not a directory {}", path.display()),
            PathError::IsNotFile(ref path) => write!(f, "is not a file {}", path.display()),
            PathError::IsNotFileOrSymlinkToFile(ref path) => write!(f, "is not a file or a symlink to a file {}", path.display()),
            PathError::MultipleHomeSymbols(ref path) => write!(f, "multiple home symbols for path {}", path.display()),
            PathError::ParentNotFound(ref path) => write!(f, "parent not found for path {}", path.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn path_empty() -> Result<PathBuf> {
        Err(PathError::empty().into())
    }

    fn parent_not_found() -> Result<PathBuf> {
        Err(PathError::parent_not_found("foo").into())
    }

    #[test]
    fn test_new_path_empty() {
        assert!(path_empty().is_err());
        assert_eq!(path_empty().unwrap_err().downcast_ref::<PathError>(), Some(&PathError::Empty));
    }

    #[test]
    fn test_parent_not_found() {
        assert!(parent_not_found().is_err());
        assert_ne!(parent_not_found().unwrap_err().downcast_ref::<PathError>(), Some(&PathError::parent_not_found("bar")));
        assert_eq!(parent_not_found().unwrap_err().downcast_ref::<PathError>(), Some(&PathError::parent_not_found("foo")));
        assert_eq!(format!("{}", parent_not_found().unwrap_err().downcast_ref::<PathError>().unwrap()), "parent not found for path foo");
    }

    #[test]
    fn test_backtrace() {
        let err = path_empty().unwrap_err();
        println!("{:?}", err);
    }
}
