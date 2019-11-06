// use std::error;
// use std::fmt;

// // New general purpose ok or error result
// pub type Result<T> = std::result::Result<T, Error>;

// /// A list of potential errors.
// #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
// pub enum ErrorKind {
//     /// An entity was not found.
//     NotFound,

//     /// Any error not part of this list.
//     Other,
// }
// impl ErrorKind {
//     pub(crate) fn as_str(&self) -> &'static str {
//         match *self {
//             ErrorKind::NotFound => "entity not found",
//             ErrorKind::Other => "other os error",
//         }
//     }
// }
// impl From<ErrorKind> for Error {
//     /// Converts an [`ErrorKind`] into an [`Error`].
//     ///
//     /// This conversion allocates a new error with a simple representation of error kind.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use std::io::{Error, ErrorKind};
//     ///
//     /// let not_found = ErrorKind::NotFound;
//     /// let error = Error::from(not_found);
//     /// assert_eq!("entity not found", format!("{}", error));
//     /// ```
//     ///
//     /// [`ErrorKind`]: ../../std/io/enum.ErrorKind.html
//     /// [`Error`]: ../../std/io/struct.Error.html
//     #[inline]
//     fn from(kind: ErrorKind) -> Error {
//         Error { repr: Repr::Simple(kind) }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Error;

// impl fmt::Display for Error {
//     fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(fmt, "example error")
//     }
// }

// impl error::Error for Error {
//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         None
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn error() {
//         let strings = vec!["tofu"];
//         let result = return_result_with_error(strings);
//         assert_eq!(true, result.is_err())
//         // assert_eq!(Error{}, result.err().unwrap())
//     }
//     fn return_result_with_error(vec: Vec<&str>) -> Result<i32> {
//         vec.first().ok_or(Error).and_then(|s| s.parse::<i32>().map_err(|_| Error))
//     }
// }
