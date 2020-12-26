use crate::errors::*;
use std::error::Error as StdError;
use std::{env, ffi, fmt, io};

/// `Result<T>` provides a simplified result type with a common error type
pub type Result<T> = std::result::Result<T, Error>;

/// Define common error wrapper type
#[derive(Debug)]
pub enum Error {
    File(FileError),
    GlobPattern(glob::PatternError),
    Io(io::Error),
    Iter(IterError),
    Nul(ffi::NulError),
    Os(OsError),
    Path(PathError),
    Regex(regex::Error),
    String(StringError),
    User(UserError),
    Var(env::VarError),
    WalkDir(walkdir::Error),
}
impl Error {
    /// Implemented directly on the `Error` type to reduce casting required
    pub fn is<T: StdError + 'static>(&self) -> bool {
        self.as_ref().is::<T>()
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn downcast_ref<T: StdError + 'static>(&self) -> Option<&T> {
        self.as_ref().downcast_ref::<T>()
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn downcast_mut<T: StdError + 'static>(&mut self) -> Option<&mut T> {
        self.as_mut().downcast_mut::<T>()
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.as_ref().source()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::File(ref err) => write!(f, "{}", err),
            Error::GlobPattern(ref err) => write!(f, "{}", err),
            Error::Io(ref err) => write!(f, "{}", err),
            Error::Iter(ref err) => write!(f, "{}", err),
            Error::Nul(ref err) => write!(f, "{}", err),
            Error::Os(ref err) => write!(f, "{}", err),
            Error::Path(ref err) => write!(f, "{}", err),
            Error::Regex(ref err) => write!(f, "{}", err),
            Error::String(ref err) => write!(f, "{}", err),
            Error::User(ref err) => write!(f, "{}", err),
            Error::Var(ref err) => write!(f, "{}", err),
            Error::WalkDir(ref err) => write!(f, "{}", err),
        }
    }
}

impl AsRef<dyn StdError> for Error {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        match *self {
            Error::File(ref err) => err,
            Error::GlobPattern(ref err) => err,
            Error::Io(ref err) => err,
            Error::Iter(ref err) => err,
            Error::Nul(ref err) => err,
            Error::Os(ref err) => err,
            Error::Path(ref err) => err,
            Error::Regex(ref err) => err,
            Error::String(ref err) => err,
            Error::User(ref err) => err,
            Error::Var(ref err) => err,
            Error::WalkDir(ref err) => err,
        }
    }
}

impl AsMut<dyn StdError> for Error {
    fn as_mut(&mut self) -> &mut (dyn StdError + 'static) {
        match *self {
            Error::File(ref mut err) => err,
            Error::GlobPattern(ref mut err) => err,
            Error::Io(ref mut err) => err,
            Error::Iter(ref mut err) => err,
            Error::Nul(ref mut err) => err,
            Error::Os(ref mut err) => err,
            Error::Path(ref mut err) => err,
            Error::Regex(ref mut err) => err,
            Error::String(ref mut err) => err,
            Error::User(ref mut err) => err,
            Error::Var(ref mut err) => err,
            Error::WalkDir(ref mut err) => err,
        }
    }
}

// Unwrap the internal error and return it
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::File(ref err) => Some(err),
            Error::GlobPattern(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            Error::Iter(ref err) => Some(err),
            Error::Nul(ref err) => Some(err),
            Error::Os(ref err) => Some(err),
            Error::Path(ref err) => Some(err),
            Error::Regex(ref err) => Some(err),
            Error::String(ref err) => Some(err),
            Error::User(ref err) => Some(err),
            Error::Var(ref err) => Some(err),
            Error::WalkDir(ref err) => Some(err),
        }
    }
}

impl From<FileError> for Error {
    fn from(err: FileError) -> Error {
        Error::File(err)
    }
}

impl From<glob::PatternError> for Error {
    fn from(err: glob::PatternError) -> Error {
        Error::GlobPattern(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<IterError> for Error {
    fn from(err: IterError) -> Error {
        Error::Iter(err)
    }
}

impl From<ffi::NulError> for Error {
    fn from(err: ffi::NulError) -> Error {
        Error::Nul(err)
    }
}

impl From<OsError> for Error {
    fn from(err: OsError) -> Error {
        Error::Os(err)
    }
}

impl From<PathError> for Error {
    fn from(err: PathError) -> Error {
        Error::Path(err)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Error {
        Error::Regex(err)
    }
}

impl From<StringError> for Error {
    fn from(err: StringError) -> Error {
        Error::String(err)
    }
}

impl From<UserError> for Error {
    fn from(err: UserError) -> Error {
        Error::User(err)
    }
}

impl From<env::VarError> for Error {
    fn from(err: env::VarError) -> Error {
        Error::Var(err)
    }
}

impl From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Error {
        Error::WalkDir(err)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::io;

    fn path_empty() -> Result<PathBuf> {
        Err(PathError::Empty)?
    }

    #[test]
    fn test_error() {
        let mut err = Error::from(FileError::FailedToExtractString);
        assert_eq!("failed to extract string from file", err.to_string());
        assert_eq!("failed to extract string from file", err.as_ref().to_string());
        assert_eq!("failed to extract string from file", err.as_mut().to_string());
        assert!(err.downcast_ref::<FileError>().is_some());
        assert!(err.downcast_mut::<FileError>().is_some());
        assert!(err.as_ref().source().is_none());

        let mut err = Error::from(io::Error::new(io::ErrorKind::AlreadyExists, "foo"));
        assert_eq!("foo", err.to_string());
        assert_eq!("foo", err.as_ref().to_string());
        assert_eq!("foo", err.as_mut().to_string());
        assert!(err.downcast_ref::<io::Error>().is_some());
        assert!(err.downcast_mut::<io::Error>().is_some());
        assert!(err.as_ref().source().is_none());

        let mut err = Error::from(IterError::ItemNotFound);
        assert_eq!("iterator item not found", err.to_string());
        assert_eq!("iterator item not found", err.as_ref().to_string());
        assert_eq!("iterator item not found", err.as_mut().to_string());
        assert!(err.downcast_ref::<IterError>().is_some());
        assert!(err.downcast_mut::<IterError>().is_some());
        assert!(err.as_ref().source().is_none());

        let mut err = Error::from(OsError::KernelReleaseNotFound);
        assert_eq!("kernel release was not found", err.to_string());
        assert_eq!("kernel release was not found", err.as_ref().to_string());
        assert_eq!("kernel release was not found", err.as_mut().to_string());
        assert!(err.downcast_ref::<OsError>().is_some());
        assert!(err.downcast_mut::<OsError>().is_some());
        assert!(err.as_ref().source().is_none());

        let mut err = Error::from(PathError::Empty);
        assert_eq!("path empty", err.to_string());
        assert_eq!("path empty", err.as_ref().to_string());
        assert_eq!("path empty", err.as_mut().to_string());
        assert!(err.downcast_ref::<PathError>().is_some());
        assert!(err.downcast_mut::<PathError>().is_some());
        assert!(err.as_ref().source().is_none());

        let mut err = Error::from(StringError::FailedToString);
        assert_eq!("failed to convert value to string", err.to_string());
        assert_eq!("failed to convert value to string", err.as_ref().to_string());
        assert_eq!("failed to convert value to string", err.as_mut().to_string());
        assert!(err.downcast_ref::<StringError>().is_some());
        assert!(err.downcast_mut::<StringError>().is_some());
        assert!(err.as_ref().source().is_none());

        let mut err = Error::from(UserError::DoesNotExistById(1));
        assert_eq!("user does not exist: 1", err.to_string());
        assert_eq!("user does not exist: 1", err.as_ref().to_string());
        assert_eq!("user does not exist: 1", err.as_mut().to_string());
        assert!(err.downcast_ref::<UserError>().is_some());
        assert!(err.downcast_mut::<UserError>().is_some());
        assert!(err.as_ref().source().is_none());

        let mut err = Error::from(std::env::VarError::NotPresent);
        assert_eq!("environment variable not found", err.to_string());
        assert_eq!("environment variable not found", err.as_ref().to_string());
        assert_eq!("environment variable not found", err.as_mut().to_string());
        assert!(err.downcast_ref::<std::env::VarError>().is_some());
        assert!(err.downcast_mut::<std::env::VarError>().is_some());
        assert!(err.as_ref().source().is_none());
    }

    #[test]
    fn test_is() {
        assert!(path_empty().is_err());
        assert!(path_empty().unwrap_err().is::<PathError>());
    }

    #[test]
    fn test_downcast_ref() {
        assert!(path_empty().is_err());
        assert_eq!(path_empty().unwrap_err().downcast_ref::<PathError>(), Some(&PathError::Empty));
    }
}
