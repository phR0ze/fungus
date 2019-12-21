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
    /// use std::path::PathBuf;
    /// use sys::preamble::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_mkdir_p");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// assert_eq!(tmpdir.exists(), false);
    /// ```
    pub fn mkdir_p<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
        let abs = path.as_ref().abs()?;
        if !abs.exists() {
            fs::create_dir_all(&abs)?;
        }
        Ok(abs)
    }

    /// Removes the given empty directory or file. Handles path expansion. Does
    /// not follow symbolic links but rather removes the links themselves.
    ///
    /// ### Examples
    /// ```
    /// use std::path::PathBuf;
    /// use sys::preamble::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_remove");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::remove(&tmpdir).is_ok());
    /// assert_eq!(tmpdir.exists(), false);
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
    /// use std::path::PathBuf;
    /// use sys::preamble::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_remove_all");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// assert_eq!(tmpdir.exists(), false);
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
    /// use std::path::PathBuf;
    /// use sys::preamble::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_touch");
    /// let tmpfile = tmpdir.join("file1");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::touch(&tmpfile).is_ok());
    /// assert_eq!(tmpfile.exists(), true);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// assert_eq!(tmpdir.exists(), false);
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
            let setup = Self { root: PathBuf::from("tests").abs().unwrap(), temp: PathBuf::from("tests/temp").abs().unwrap() };
            crate::mkdir_p(&setup.temp).unwrap();
            setup
        }
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

        assert!(crate::touch(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), true);

        // Clean up
        assert!(crate::remove(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), false);
    }
}
