use failure::{Backtrace, Context, Fail};
use std::fmt;

use crate::iter_error::*;
use crate::path_error::*;
use crate::sys_error::*;

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
    /// An iterator error
    Iter(IterError),

    /// A path error
    Path(PathError),

    /// An IO error
    Sys(SysError),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Iter(ref err) => err.fmt(f),
            ErrorKind::Path(ref err) => err.fmt(f),
            ErrorKind::Sys(ref err) => err.fmt(f),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error::from(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(ctx: Context<ErrorKind>) -> Self {
        Error { ctx }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::path::PathBuf;

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
