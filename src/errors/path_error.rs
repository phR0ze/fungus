use std::error::Error as StdError;
use std::fmt;
use std::path::{Path, PathBuf};

// An error indicating that something went wrong with a path operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathError {
    /// An error indicating that the path does not exist.
    DoesNotExist(PathBuf),

    /// An error indicating that the path is empty.
    Empty,

    /// An error indicating that the path exists already.
    ExistsAlready(PathBuf),

    /// An error indicating that the path does not have an extension.
    ExtensionNotFound(PathBuf),

    /// An error indicating a failure to convert the path to a string.
    FailedToString(PathBuf),

    /// An error indicating that the path does not contain a filename.
    FileNameNotFound(PathBuf),

    /// An error indicating that the path failed to expand properly.
    InvalidExpansion(PathBuf),

    /// An error indicating that the path is not a directory.
    IsNotDir(PathBuf),

    /// An error indicating that the path is not an executable file.
    IsNotExec(PathBuf),

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

    /// Return an error indicating that the path exists already
    pub fn exists_already<T: AsRef<Path>>(path: T) -> PathError {
        PathError::ExistsAlready(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path extension was not found
    pub fn extension_not_found<T: AsRef<Path>>(path: T) -> PathError {
        PathError::ExtensionNotFound(path.as_ref().to_path_buf())
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

    /// Return an error indicating that the path is not an executable
    pub fn is_not_exec<T: AsRef<Path>>(path: T) -> PathError {
        PathError::IsNotExec(path.as_ref().to_path_buf())
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

impl StdError for PathError {}

impl AsRef<dyn StdError> for PathError {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        self
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PathError::DoesNotExist(ref path) => write!(f, "path does not exist: {}", path.display()),
            PathError::Empty => write!(f, "path empty"),
            PathError::ExistsAlready(ref path) => write!(f, "path exists already: {}", path.display()),
            PathError::ExtensionNotFound(ref path) => write!(f, "path extension not found: {}", path.display()),
            PathError::FailedToString(ref path) => write!(f, "failed to convert to string for path: {}", path.display()),
            PathError::FileNameNotFound(ref path) => write!(f, "filename not found for path: {}", path.display()),
            PathError::InvalidExpansion(ref path) => write!(f, "invalid expansion for path: {}", path.display()),
            PathError::IsNotDir(ref path) => write!(f, "is not a directory: {}", path.display()),
            PathError::IsNotExec(ref path) => write!(f, "is not an executable: {}", path.display()),
            PathError::IsNotFile(ref path) => write!(f, "is not a file: {}", path.display()),
            PathError::IsNotFileOrSymlinkToFile(ref path) => write!(f, "is not a file or a symlink to a file: {}", path.display()),
            PathError::MultipleHomeSymbols(ref path) => write!(f, "multiple home symbols for path: {}", path.display()),
            PathError::ParentNotFound(ref path) => write!(f, "parent not found for path: {}", path.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    fn path_empty() -> Result<PathBuf> {
        Err(PathError::Empty.into())
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
        assert_eq!(format!("{}", parent_not_found().unwrap_err().downcast_ref::<PathError>().unwrap()), "parent not found for path: foo");
    }

    #[test]
    fn test_other_errors() {
        assert_eq!(PathError::does_not_exist(Path::new("foo")), PathError::DoesNotExist(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::DoesNotExist(PathBuf::from("foo"))), "path does not exist: foo");
        assert_eq!(format!("{}", PathError::Empty), "path empty");
        assert_eq!(PathError::exists_already(Path::new("foo")), PathError::ExistsAlready(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::ExistsAlready(PathBuf::from("foo"))), "path exists already: foo");
        assert_eq!(PathError::extension_not_found(Path::new("foo")), PathError::ExtensionNotFound(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::ExtensionNotFound(PathBuf::from("foo"))), "path extension not found: foo");
        assert_eq!(PathError::failed_to_string(Path::new("foo")), PathError::FailedToString(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::failed_to_string(PathBuf::from("foo"))), "failed to convert to string for path: foo");
        assert_eq!(PathError::filename_not_found(Path::new("foo")), PathError::FileNameNotFound(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::filename_not_found(PathBuf::from("foo"))), "filename not found for path: foo");
        assert_eq!(PathError::invalid_expansion(Path::new("foo")), PathError::InvalidExpansion(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::invalid_expansion(PathBuf::from("foo"))), "invalid expansion for path: foo");
        assert_eq!(PathError::is_not_dir(Path::new("foo")), PathError::IsNotDir(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::is_not_dir(PathBuf::from("foo"))), "is not a directory: foo");
        assert_eq!(PathError::is_not_exec(Path::new("foo")), PathError::IsNotExec(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::is_not_exec(PathBuf::from("foo"))), "is not an executable: foo");
        assert_eq!(PathError::is_not_file(Path::new("foo")), PathError::IsNotFile(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::is_not_file(PathBuf::from("foo"))), "is not a file: foo");
        assert_eq!(PathError::is_not_file_or_symlink_to_file(Path::new("foo")), PathError::IsNotFileOrSymlinkToFile(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::is_not_file_or_symlink_to_file(PathBuf::from("foo"))), "is not a file or a symlink to a file: foo");
        assert_eq!(PathError::multiple_home_symbols(Path::new("foo")), PathError::MultipleHomeSymbols(PathBuf::from("foo")));
        assert_eq!(format!("{}", PathError::multiple_home_symbols(PathBuf::from("foo"))), "multiple home symbols for path: foo");
    }

    #[test]
    fn test_backtrace() {
        let err = path_empty().unwrap_err();
        println!("{:?}", err);
    }
}
