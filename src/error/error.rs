use crate::error::*;
use std::error::Error as StdError;
use std::{env, fmt, io};

/// `Result<T>` provides a simplified result type with a common error type
pub type Result<T> = std::result::Result<T, Error>;

/// Define common error wrapper type
#[derive(Debug)]
pub enum Error {
    File(FileError),
    GlobPattern(glob::PatternError),
    Io(io::Error),
    Iter(IterError),
    Path(PathError),
    Os(OsError),
    Regex(regex::Error),
    String(StringError),
    User(UserError),
    Var(env::VarError),
    WalkDir(walkdir::Error),
}
impl Error {
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            //BadPath {path: PathRef} => write!(f, "{}", path),
            Error::File(ref err) => write!(f, "{}", err),
            Error::GlobPattern(ref err) => write!(f, "{}", err),
            Error::Io(ref err) => write!(f, "{}", err),
            Error::Iter(ref err) => write!(f, "{}", err),
            Error::Path(ref err) => write!(f, "{}", err),
            Error::Os(ref err) => write!(f, "{}", err),
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
        self
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
            Error::Path(ref err) => Some(err),
            Error::Os(ref err) => Some(err),
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

impl From<PathError> for Error {
    fn from(err: PathError) -> Error {
        Error::Path(err)
    }
}

impl From<OsError> for Error {
    fn from(err: OsError) -> Error {
        Error::Os(err)
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

// #[cfg(test)]
// mod tests {
//     use crate::prelude::*;

//     #[test]
//     fn test_errors() {}
// }
