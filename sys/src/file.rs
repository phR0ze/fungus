use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::path::PathExt;
use core::preamble::*;

// File utilities
// -------------------------------------------------------------------------------------------------
pub mod files {
    use super::*;

    /// Creates the given directory and any parent directories needed, handling path expansion and
    /// returning an absolute path created.
    ///
    /// ### Examples
    /// ```
    /// use rs::sys::preamble::*;
    ///
    /// assert_eq!(sys::mkdir_p("~/foo").unwrap(), sys::abs("~/foo"));
    /// ```
    pub fn mkdir_p<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
        let abs = path.as_ref().abs()?;
        fs::create_dir_all(&abs)?;
        Ok(abs)
    }

    /// Removes the given empty directory or file. Handles path expansion. Does
    /// not follow symbolic links but rather removes the links themselves.
    ///
    /// ### Examples
    /// ```
    /// use rs::sys::preamble::*;
    ///
    /// assert_eq!(sys::remove("~/foo").is_ok(), true);
    /// ```
    pub fn remove<T: AsRef<Path>>(path: T) -> Result<()> {
        let abs = path.as_ref().abs()?;
        let wrapped_meta = fs::metadata(&abs);
        if wrapped_meta.is_ok() {
            let meta = wrapped_meta.unwrap();
            if meta.is_file() {
                fs::remove_file(abs)?;
            } else if meta.is_dir() {
                fs::remove_dir(abs)?;
            }
        }
        Ok(())
    }

    /// Removes the given directory after removing all of its contents. Handles path expansion. Does
    /// not follow symbolic links but rather removes the links themselves.
    ///
    /// ### Examples
    /// ```
    /// use rs::sys::preamble::*;
    ///
    /// assert_eq!(sys::mkdir_p("~/foo").unwrap(), sys::abs("~/foo"));
    /// ```
    pub fn remove_all<T: AsRef<Path>>(path: T) -> Result<()> {
        let abs = path.as_ref().abs()?;
        if abs.exists() {
            fs::remove_dir_all(abs)?;
        }
        Ok(())
    }

    /// Create an empty file similar to the linux touch command. Handles path expansion.
    ///
    /// ### Examples
    /// ```
    /// use rs::sys::preamble::*;
    ///
    /// assert_eq!(sys::touch("~/foo").unwrap(), sys::abs("~/foo"));
    /// ```
    pub fn touch<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
        let abs = path.as_ref().abs()?;

        // create if the file doesn't exist
        if !abs.exists() {
            File::create(&abs)?;
        }

        // update the access and modification times for the file

        Ok(abs)
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::preamble::*;

    // Reusable teset setup
    struct Setup {
        root: PathBuf,
        temp: PathBuf,
    }
    impl Setup {
        fn init() -> Self {
            let setup = Self { root: PathBuf::from("test").abs().unwrap(), temp: PathBuf::from("test/temp").abs().unwrap() };
            crate::mkdir_p(&setup.temp).unwrap();
            setup
        }
    }

    #[test]
    fn test_mkdir_p() {
        let setup = Setup::init();
        assert!(setup.temp.is_dir());
    }

    #[test]
    fn test_remove() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("remove_dir");
        let tmpfile = setup.temp.join("remove_file");

        // Remove empty directory
        assert!(crate::mkdir_p(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), true);
        assert!(crate::remove(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);

        // Remove file
        assert!(crate::touch(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), true);
        assert!(crate::remove(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), false);
    }

    #[test]
    fn test_remove_all() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("remove_all");

        assert!(crate::mkdir_p(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), true);
        assert!(crate::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_touch() {
        let setup = Setup::init();
        let tmpfile = setup.temp.join("touch");

        assert!(crate::remove(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), false);
        assert!(crate::touch(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), true);
    }
}
