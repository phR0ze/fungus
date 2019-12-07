use std::env;
use std::path::PathBuf;

use core::*;

pub mod user {
    use super::*;

    // Returns the full path to the current user's home directory.
    pub fn home() -> Result<PathBuf> {
        let os_str = env::var("HOME")?;
        let dir = PathBuf::from(os_str);
        Ok(dir)
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    use crate::*;

    #[test]
    fn test_user_home() {
        let home_str = env::var("HOME").unwrap();
        let home_path = PathBuf::from(home_str);
        let home_dir = home_path.parent().unwrap();
        assert_eq!(home_dir.to_path_buf(), user::home().unwrap().dirname().unwrap());
    }
}
