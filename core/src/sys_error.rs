use glob;
use std::env;
use std::fmt;
use std::io;

use crate::error::*;

/// An error indicating that something went wrong with an IO operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SysError {
    /// The specified environment variable was not found in the current process's environment.
    EnvNotFound,

    /// The specified environment variable was found, but it did not contain valid unicode data.
    EnvNotUnicode,

    /// An error from the glob package
    Glob { kind: io::ErrorKind, desc: String },

    /// A glob pattern error from the glob package
    GlobPattern(String),

    /// An error from the std::io package.
    Io { kind: io::ErrorKind, desc: String },
}

impl fmt::Display for SysError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SysError::EnvNotFound => write!(f, "std::env::VarError environment variable not found"),
            SysError::EnvNotUnicode => write!(f, "std::env::VarError environment variable not valid unicode"),
            SysError::Glob { ref kind, ref desc } => write!(f, "glob::GlobError {}", desc),
            SysError::GlobPattern(ref desc) => write!(f, "glob::PatternError {}", desc),
            SysError::Io { ref kind, ref desc } => write!(f, "std::io::Error {}", desc),
        }
    }
}

// SysError => Error
// -------------------------------------------------------------------------------------------------
impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        Error::from(ErrorKind::Sys(err))
    }
}

// std::env::VarError
// -------------------------------------------------------------------------------------------------
impl From<env::VarError> for SysError {
    fn from(err: env::VarError) -> Self {
        match err {
            env::VarError::NotPresent => SysError::EnvNotFound,
            env::VarError::NotUnicode(_) => SysError::EnvNotUnicode,
        }
    }
}

impl From<env::VarError> for Error {
    fn from(err: env::VarError) -> Self {
        Error::from(SysError::from(err))
    }
}

// glob errors
// -------------------------------------------------------------------------------------------------
impl From<glob::GlobError> for SysError {
    fn from(err: glob::GlobError) -> Self {
        let io_err = err.into_error();
        SysError::Glob { kind: io_err.kind(), desc: format!("{}", io_err) }
    }
}

impl From<glob::GlobError> for Error {
    fn from(err: glob::GlobError) -> Self {
        Error::from(SysError::from(err))
    }
}

impl From<glob::PatternError> for SysError {
    fn from(err: glob::PatternError) -> Self {
        SysError::GlobPattern(format!("{}", err))
    }
}

impl From<glob::PatternError> for Error {
    fn from(err: glob::PatternError) -> Self {
        Error::from(SysError::from(err))
    }
}

// std::io::Error => Error
// -------------------------------------------------------------------------------------------------
impl From<io::Error> for SysError {
    fn from(err: io::Error) -> Self {
        SysError::Io { kind: err.kind(), desc: format!("{}", err) }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::from(SysError::from(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::path::PathBuf;

    // io::Error
    // --------------------------------------------------------------------------------------------
    fn io_result() -> io::Result<PathBuf> {
        Err(io::Error::from(io::ErrorKind::AlreadyExists))
    }

    fn io_error_from_result() -> Result<PathBuf> {
        let val = io_result()?;
        Ok(val)
    }

    #[test]
    fn test_error_from_io() {
        let err = io_error_from_result().unwrap_err();
        assert_eq!("std::io::Error entity already exists", format!("{}", err));
    }

    #[test]
    fn test_sys_from_io() {
        let err = SysError::from(io::Error::from(io::ErrorKind::AlreadyExists));
        assert_eq!("std::io::Error entity already exists", format!("{}", err));
    }

    // env::VarError
    // --------------------------------------------------------------------------------------------
    fn env_result() -> std::result::Result<String, env::VarError> {
        Err(env::VarError::NotPresent)
    }

    fn env_error_from_result() -> Result<String> {
        let val = env_result()?;
        Ok(val)
    }

    #[test]
    fn test_error_from_env() {
        let err = env_error_from_result().unwrap_err();
        assert_eq!("std::env::VarError environment variable not found", format!("{}", err));
    }

    #[test]
    fn test_sys_from_env() {
        let err = SysError::from(env::VarError::NotPresent);
        assert_eq!("std::env::VarError environment variable not found", format!("{}", err));
    }

    // other
    // --------------------------------------------------------------------------------------------
    #[test]
    fn test_backtrace() {
        let err = io_error_from_result().unwrap_err();
        println!("{:?}", err)
    }
}
