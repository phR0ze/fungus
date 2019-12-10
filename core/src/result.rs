use std::result;

/// The canonical `Result` type.
pub type Result<T> = result::Result<T, failure::Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::path::PathBuf;

    fn parent_not_found() -> Result<PathBuf> {
        Err(PathError::parent_not_found("foo").into())
    }

    #[test]
    fn test_eq_err() {
        assert_ne!(parent_not_found().unwrap_err().downcast_ref::<PathError>(), Some(&PathError::parent_not_found("bar")));
        assert_eq!(parent_not_found().unwrap_err().downcast_ref::<PathError>(), Some(&PathError::parent_not_found("foo")));
    }
}
