use std::error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathError {
    ComponentNotFound,
    Empty,
    FailedToString,
    FileNameNotFound,
    InvalidExpansion,
    MultipleHomeSymbols,
    ParentNotFound,
    Other,
}

impl PathError {
    pub(crate) fn as_str(&self) -> &'static str {
        match *self {
            PathError::ComponentNotFound => "component not found",
            PathError::Empty => "empty path",
            PathError::FailedToString => "failed to_string",
            PathError::FileNameNotFound => "filename not found",
            PathError::InvalidExpansion => "invalid expansion",
            PathError::MultipleHomeSymbols => "multiple home symbols",
            PathError::ParentNotFound => "parent not found",
            PathError::Other => "other path error",
        }
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl error::Error for PathError {
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
