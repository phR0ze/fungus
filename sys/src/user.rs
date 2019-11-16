use std::env;
use std::path::PathBuf;

use errors::Result;

// Returns the full path to the current user's home directory.
pub fn user_home() -> Result<PathBuf> {
    let os_str = env::var("HOME")?;
    let dir = PathBuf::from(os_str);
    Ok(dir)
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::*;

    #[test]
    fn test_user_home() {
        assert_eq!(PathBuf::from("/home"), user_home().unwrap().dirname().unwrap());
    }
}
