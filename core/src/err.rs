// use glob;
// use std::env;
// use std::error;
// use std::fmt;
// use std::io;

// use crate::path_error::*;

// // Error for the single extension method.
// //--------------------------------------------------------------------------------------------------
// #[derive(Debug)]
// pub enum OldError {
//     Io(io::Error),
//     Env(env::VarError),
//     Glob(glob::GlobError),
//     GlobPattern(glob::PatternError),
//     Path(PathError),
// }

// impl OldError {
//     // Creates a new Error from a known kind of error as well as an
//     // arbitrary error payload.
//     pub fn new() -> OldError {
//         OldError::Io(io::Error::new(io::ErrorKind::Other, "test"))
//     }
// }

// impl fmt::Display for OldError {
//     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             OldError::Io(ref err) => err.fmt(fmt),
//             OldError::Env(ref err) => err.fmt(fmt),
//             OldError::Glob(ref err) => err.fmt(fmt),
//             OldError::GlobPattern(ref err) => err.fmt(fmt),
//             OldError::Path(kind) => write!(fmt, "{}", kind.as_str()),
//         }
//     }
// }

// impl error::Error for OldError {
//     fn description(&self) -> &str {
//         "single not found"
//     }
//     fn cause(&self) -> Option<&(dyn error::Error)> {
//         None
//     }
//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         None
//     }
// }

// impl From<io::Error> for OldError {
//     fn from(err: io::Error) -> OldError {
//         OldError::Io(err)
//     }
// }

// impl From<glob::GlobError> for OldError {
//     fn from(err: glob::GlobError) -> OldError {
//         OldError::Glob(err)
//     }
// }

// impl From<glob::PatternError> for OldError {
//     fn from(err: glob::PatternError) -> OldError {
//         OldError::GlobPattern(err)
//     }
// }

// impl From<env::VarError> for OldError {
//     fn from(err: env::VarError) -> OldError {
//         OldError::Env(err)
//     }
// }

// // Converts an `IterError` into an `Error`.
// impl From<PathError> for OldError {
//     fn from(err: PathError) -> OldError {
//         OldError::Path(err)
//     }
// }
