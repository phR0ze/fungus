use std::result;

use crate::error::*;

/// The canonical `Result` type.
pub type Result<T> = result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::path::PathBuf;

    fn return_path_parent_not_found_error() -> Result<PathBuf> {
        Err(PathError::parent_not_found("foo"))
    }

    #[test]
    fn test_eq_err() {
        assert!(return_path_parent_not_found_error().unwrap_err().eq(&PathError::parent_not_found("foo")));
        assert!(!return_path_parent_not_found_error().unwrap_err().eq(&PathError::parent_not_found("bar")));
    }
}
