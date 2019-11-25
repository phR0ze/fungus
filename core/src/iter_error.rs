use std::error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IterError {
    ItemNotFound,
    MultipleFound,
}

impl IterError {
    pub(crate) fn as_str(&self) -> &'static str {
        match *self {
            IterError::ItemNotFound => "item not found",
            IterError::MultipleFound => "multiple found",
        }
    }
}

impl fmt::Display for IterError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl error::Error for IterError {
    fn description(&self) -> &str {
        self.as_str()
    }
    fn cause(&self) -> Option<&(dyn error::Error)> {
        None
    }
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
