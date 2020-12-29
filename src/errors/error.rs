use crate::errors::*;
use std::error::Error as StdError;
use std::{env, ffi, fmt, io};

/// `Result<T>` provides a simplified result type with a common error type
pub type FuResult<T> = std::result::Result<T, FuError>;

/// Define common error wrapper type
#[derive(Debug)]
pub enum FuError {
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
impl FuError {
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

impl std::fmt::Display for FuError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FuError::File(ref err) => write!(f, "{}", err),
            FuError::GlobPattern(ref err) => write!(f, "{}", err),
            FuError::Io(ref err) => write!(f, "{}", err),
            FuError::Iter(ref err) => write!(f, "{}", err),
            FuError::Nul(ref err) => write!(f, "{}", err),
            FuError::Os(ref err) => write!(f, "{}", err),
            FuError::Path(ref err) => write!(f, "{}", err),
            FuError::Regex(ref err) => write!(f, "{}", err),
            FuError::String(ref err) => write!(f, "{}", err),
            FuError::User(ref err) => write!(f, "{}", err),
            FuError::Var(ref err) => write!(f, "{}", err),
            FuError::WalkDir(ref err) => write!(f, "{}", err),
        }
    }
}

impl AsRef<dyn StdError> for FuError {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        match *self {
            FuError::File(ref err) => err,
            FuError::GlobPattern(ref err) => err,
            FuError::Io(ref err) => err,
            FuError::Iter(ref err) => err,
            FuError::Nul(ref err) => err,
            FuError::Os(ref err) => err,
            FuError::Path(ref err) => err,
            FuError::Regex(ref err) => err,
            FuError::String(ref err) => err,
            FuError::User(ref err) => err,
            FuError::Var(ref err) => err,
            FuError::WalkDir(ref err) => err,
        }
    }
}

impl AsMut<dyn StdError> for FuError {
    fn as_mut(&mut self) -> &mut (dyn StdError + 'static) {
        match *self {
            FuError::File(ref mut err) => err,
            FuError::GlobPattern(ref mut err) => err,
            FuError::Io(ref mut err) => err,
            FuError::Iter(ref mut err) => err,
            FuError::Nul(ref mut err) => err,
            FuError::Os(ref mut err) => err,
            FuError::Path(ref mut err) => err,
            FuError::Regex(ref mut err) => err,
            FuError::String(ref mut err) => err,
            FuError::User(ref mut err) => err,
            FuError::Var(ref mut err) => err,
            FuError::WalkDir(ref mut err) => err,
        }
    }
}

impl From<FileError> for FuError {
    fn from(err: FileError) -> FuError {
        FuError::File(err)
    }
}

impl From<glob::PatternError> for FuError {
    fn from(err: glob::PatternError) -> FuError {
        FuError::GlobPattern(err)
    }
}

impl From<io::Error> for FuError {
    fn from(err: io::Error) -> FuError {
        FuError::Io(err)
    }
}

impl From<IterError> for FuError {
    fn from(err: IterError) -> FuError {
        FuError::Iter(err)
    }
}

impl From<ffi::NulError> for FuError {
    fn from(err: ffi::NulError) -> FuError {
        FuError::Nul(err)
    }
}

impl From<OsError> for FuError {
    fn from(err: OsError) -> FuError {
        FuError::Os(err)
    }
}

impl From<PathError> for FuError {
    fn from(err: PathError) -> FuError {
        FuError::Path(err)
    }
}

impl From<regex::Error> for FuError {
    fn from(err: regex::Error) -> FuError {
        FuError::Regex(err)
    }
}

impl From<StringError> for FuError {
    fn from(err: StringError) -> FuError {
        FuError::String(err)
    }
}

impl From<UserError> for FuError {
    fn from(err: UserError) -> FuError {
        FuError::User(err)
    }
}

impl From<env::VarError> for FuError {
    fn from(err: env::VarError) -> FuError {
        FuError::Var(err)
    }
}

impl From<walkdir::Error> for FuError {
    fn from(err: walkdir::Error) -> FuError {
        FuError::WalkDir(err)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::io;

    fn path_empty() -> FuResult<PathBuf> {
        Err(PathError::Empty)?
    }

    #[test]
    fn test_error() {
        let mut err = FuError::from(FileError::FailedToExtractString);
        assert_eq!("failed to extract string from file", err.to_string());
        assert_eq!("failed to extract string from file", err.as_ref().to_string());
        assert_eq!("failed to extract string from file", err.as_mut().to_string());
        assert!(err.downcast_ref::<FileError>().is_some());
        assert!(err.downcast_mut::<FileError>().is_some());
        assert!(err.source().is_none());

        let mut err = FuError::from(glob::PatternError { pos: 1, msg: "1" });
        assert_eq!("Pattern syntax error near position 1: 1", err.to_string());
        assert_eq!("Pattern syntax error near position 1: 1", err.as_ref().to_string());
        assert_eq!("Pattern syntax error near position 1: 1", err.as_mut().to_string());
        assert!(err.downcast_ref::<glob::PatternError>().is_some());
        assert!(err.downcast_mut::<glob::PatternError>().is_some());
        assert!(err.source().is_none());

        let mut err = FuError::from(io::Error::new(io::ErrorKind::AlreadyExists, "foo"));
        assert_eq!("foo", err.to_string());
        assert_eq!("foo", err.as_ref().to_string());
        assert_eq!("foo", err.as_mut().to_string());
        assert!(err.downcast_ref::<io::Error>().is_some());
        assert!(err.downcast_mut::<io::Error>().is_some());
        assert!(err.source().is_none());

        let mut err = FuError::from(IterError::ItemNotFound);
        assert_eq!("iterator item not found", err.to_string());
        assert_eq!("iterator item not found", err.as_ref().to_string());
        assert_eq!("iterator item not found", err.as_mut().to_string());
        assert!(err.downcast_ref::<IterError>().is_some());
        assert!(err.downcast_mut::<IterError>().is_some());
        assert!(err.source().is_none());

        let mut err = FuError::from(std::ffi::CString::new(b"f\0oo".to_vec()).unwrap_err());
        assert_eq!("nul byte found in provided data at position: 1", err.to_string());
        assert_eq!("nul byte found in provided data at position: 1", err.as_ref().to_string());
        assert_eq!("nul byte found in provided data at position: 1", err.as_mut().to_string());
        assert!(err.downcast_ref::<std::ffi::NulError>().is_some());
        assert!(err.downcast_mut::<std::ffi::NulError>().is_some());
        assert!(err.source().is_none());

        let mut err = FuError::from(OsError::KernelReleaseNotFound);
        assert_eq!("kernel release was not found", err.to_string());
        assert_eq!("kernel release was not found", err.as_ref().to_string());
        assert_eq!("kernel release was not found", err.as_mut().to_string());
        assert!(err.downcast_ref::<OsError>().is_some());
        assert!(err.downcast_mut::<OsError>().is_some());
        assert!(err.source().is_none());

        let mut err = FuError::from(PathError::Empty);
        assert_eq!("path empty", err.to_string());
        assert_eq!("path empty", err.as_ref().to_string());
        assert_eq!("path empty", err.as_mut().to_string());
        assert!(err.downcast_ref::<PathError>().is_some());
        assert!(err.downcast_mut::<PathError>().is_some());
        assert!(err.source().is_none());

        let mut err = FuError::from(regex::Error::Syntax("foo".to_string()));
        assert_eq!("foo", err.to_string());
        assert_eq!("foo", err.as_ref().to_string());
        assert_eq!("foo", err.as_mut().to_string());
        assert!(err.downcast_ref::<regex::Error>().is_some());
        assert!(err.downcast_mut::<regex::Error>().is_some());
        assert!(err.source().is_none());

        let mut err = FuError::from(StringError::FailedToString);
        assert_eq!("failed to convert value to string", err.to_string());
        assert_eq!("failed to convert value to string", err.as_ref().to_string());
        assert_eq!("failed to convert value to string", err.as_mut().to_string());
        assert!(err.downcast_ref::<StringError>().is_some());
        assert!(err.downcast_mut::<StringError>().is_some());
        assert!(err.source().is_none());

        let mut err = FuError::from(UserError::DoesNotExistById(1));
        assert_eq!("user does not exist: 1", err.to_string());
        assert_eq!("user does not exist: 1", err.as_ref().to_string());
        assert_eq!("user does not exist: 1", err.as_mut().to_string());
        assert!(err.downcast_ref::<UserError>().is_some());
        assert!(err.downcast_mut::<UserError>().is_some());
        assert!(err.source().is_none());

        let mut err = FuError::from(std::env::VarError::NotPresent);
        assert_eq!("environment variable not found", err.to_string());
        assert_eq!("environment variable not found", err.as_ref().to_string());
        assert_eq!("environment variable not found", err.as_mut().to_string());
        assert!(err.downcast_ref::<std::env::VarError>().is_some());
        assert!(err.downcast_mut::<std::env::VarError>().is_some());
        assert!(err.source().is_none());
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
