use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::iter_error::*;
use crate::path_error::*;

/// An error aggregate for common errors in rust
#[derive(Debug)]
pub struct Error {
    ctx: Context<ErrorKind>,
}
impl Error {
    /// Return the kind of this error.
    pub fn kind(&self) -> &ErrorKind {
        self.ctx.get_context()
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.ctx.cause()
    }
    fn backtrace(&self) -> Option<&Backtrace> {
        self.ctx.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.ctx.fmt(f)
    }
}

/// The specific kind of error that can occur.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// An IO error
    // Io(io::Error),

    /// An iterator error
    Iter(IterError),

    /// A path error
    Path(PathError),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // ErrorKind::Io(ref err) => err.fmt(f),
            ErrorKind::Iter(ref err) => err.fmt(f),
            ErrorKind::Path(ref err) => err.fmt(f),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error::from(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(ctx: Context<ErrorKind>) -> Error {
        Error { ctx }
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

    // #[test]
    // fn test_backtrace() {
    //     let err = path_empty().unwrap_err();
    //     println!("{:?}", err);
    // }

    #[test]
    fn test_error_eq() {
        assert_eq!(PathError::empty(), PathError::empty());
        assert_eq!(PathError::parent_not_found("foo"), PathError::parent_not_found("foo"));
        assert_ne!(PathError::parent_not_found("foo"), PathError::parent_not_found("bar"));
    }
}

// // use glob;
// // use std::env;
// // use std::error;
// // use std::io;

// // use crate::*;

// // // Error for the single extension method.
// // //--------------------------------------------------------------------------------------------------
// // #[derive(Debug)]
// // pub enum Error {
// //     Io(io::Error),
// //     Env(env::VarError),
// //     Glob(glob::GlobError),
// //     GlobPattern(glob::PatternError),
// //     Path(PathError),
// //     Iter(IterError),
// // }

// // impl fmt::Display for Error {
// //     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
// //         match self {
// //             Error::Io(ref err) => err.fmt(fmt),
// //             Error::Env(ref err) => err.fmt(fmt),
// //             Error::Glob(ref err) => err.fmt(fmt),
// //             Error::GlobPattern(ref err) => err.fmt(fmt),
// //             Error::Iter(ref err) => err.fmt(fmt),
// //             Error::Path(ref err) => err.fmt(fmt),
// //         }
// //     }
// // }

// // impl error::Error for Error {
// //     fn description(&self) -> &str {
// //         match self {
// //             Error::Io(ref err) => err.description(),
// //             Error::Env(ref err) => err.description(),
// //             Error::Glob(ref err) => err.description(),
// //             Error::GlobPattern(ref err) => err.description(),
// //             Error::Iter(ref err) => err.description(),
// //             Error::Path(ref err) => err.description(),
// //         }
// //     }
// //     fn cause(&self) -> Option<&(dyn error::Error)> {
// //         None
// //     }
// //     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
// //         None
// //     }
// // }

// // impl From<io::Error> for Error {
// //     fn from(err: io::Error) -> Error {
// //         Error::Io(err)
// //     }
// // }

// // impl From<glob::GlobError> for Error {
// //     fn from(err: glob::GlobError) -> Error {
// //         Error::Glob(err)
// //     }
// // }

// // impl From<glob::PatternError> for Error {
// //     fn from(err: glob::PatternError) -> Error {
// //         Error::GlobPattern(err)
// //     }
// // }

// // impl From<env::VarError> for Error {
// //     fn from(err: env::VarError) -> Error {
// //         Error::Env(err)
// //     }
// // }

// // // Converts an `IterError` into an `Error`.
// // impl From<PathError> for Error {
// //     fn from(err: PathError) -> Error {
// //         Error::Path(err)
// //     }
// // }

// // // Converts a `PathError` into an `Error`.
// // impl From<IterError> for Error {
// //     fn from(err: IterError) -> Error {
// //         Error::Iter(err)
// //     }
// // }

// // #[cfg(test)]
// // mod tests {
// //     use super::*;
// //     use std::env;
// //     use std::error::Error;
// //     use std::ffi::OsStr;
// //     use std::path::{Component, PathBuf};

// //     #[test]
// //     fn test_description() {
// //         let err = PathError::ParentNotFound;
// //         assert_eq!("parent not found", err.description());
// //         //let err = Error::from(PathError::ParentNotFound);
// //     }
// // }
