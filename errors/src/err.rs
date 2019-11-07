use std::error;

/// The canonical `Result` type.
pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
