use glob;
use std::env;
use std::error;
use std::fmt;
use std::io;

use crate::iter_error::*;
use crate::path_error::*;

/// The canonical `Result` type.
pub type Result<T> = std::result::Result<T, Error>;

// Error for the single extension method.
//--------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Env(env::VarError),
    Glob(glob::GlobError),
    GlobPattern(glob::PatternError),
    Path(PathError),
    Iter(IterError),
}

impl Error {
    // Creates a new Error from a known kind of error as well as an
    // arbitrary error payload.
    pub fn new() -> Error {
        Error::Io(io::Error::new(io::ErrorKind::Other, "test"))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(ref err) => err.fmt(fmt),
            Error::Env(ref err) => err.fmt(fmt),
            Error::Glob(ref err) => err.fmt(fmt),
            Error::GlobPattern(ref err) => err.fmt(fmt),
            Error::Iter(kind) => write!(fmt, "{}", kind.as_str()),
            Error::Path(kind) => write!(fmt, "{}", kind.as_str()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "single not found"
    }
    fn cause(&self) -> Option<&(dyn error::Error)> {
        None
    }
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<glob::GlobError> for Error {
    fn from(err: glob::GlobError) -> Error {
        Error::Glob(err)
    }
}

impl From<glob::PatternError> for Error {
    fn from(err: glob::PatternError) -> Error {
        Error::GlobPattern(err)
    }
}

impl From<env::VarError> for Error {
    fn from(err: env::VarError) -> Error {
        Error::Env(err)
    }
}

// Converts an `IterError` into an `Error`.
impl From<PathError> for Error {
    fn from(err: PathError) -> Error {
        Error::Path(err)
    }
}

// Converts a `PathError` into an `Error`.
impl From<IterError> for Error {
    fn from(err: IterError) -> Error {
        Error::Iter(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::ffi::OsStr;
    use std::path::{Component, PathBuf};

    #[test]
    fn test_single() {
        let result = func_with_result_convert();
    }

    fn func_with_result_convert() -> Result<PathBuf> {
        let os_str = env::var("HOME")?;
        let dir = PathBuf::from(os_str);
        Ok(dir)
    }
}
