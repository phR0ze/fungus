// File utilities
// -------------------------------------------------------------------------------------------------
pub mod files {
    use std::fs;
    use std::fs::File;
    use std::os::unix;
    use std::path::{Path, PathBuf};

    use crate::path::PathExt;
    use core::preamble::*;

    /// Copies a single file from src to dst, creating destination directories as needed and
    /// handling path expansion returning an absolute path of the destination.
    ///
    /// The dst will be copied to if it is an existing directory.
    /// The dst will be a clone of the src if it doesn't exist.
    ///
    /// ### Examples
    /// ```
    /// ```
    pub fn copy_file<T: AsRef<Path>, U: AsRef<Path>>(src: T, dst: U) -> Result<PathBuf> {
        // Check the source for issues
        let src_abs = src.as_ref().abs()?;
        if !src_abs.exists() {
            return Err(PathError::does_not_exist(src).into());
        }
        // if src_abs.is_dir() || src_s

        // Configure and check the destination
        let dst_abs = src.as_ref().abs()?;

        Ok(dst_abs)
    }

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

    /// Creates a new symbolic link. Handles path expansion and returns an absolute path to the
    /// link while still creating the symbolic link as a relative path to the target.
    ///
    /// ### Examples
    /// ```
    /// use std::path::PathBuf;
    /// use sys::preamble::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_symlink");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.join("file1");
    /// let link1 = tmpdir.join("link1");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(sys::symlink(&link1, &file1).is_ok());
    /// assert_eq!(link1.exists(), true);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    pub fn symlink<T: AsRef<Path>, U: AsRef<Path>>(link: T, target: U) -> Result<PathBuf> {
        let link_abs = link.as_ref().abs()?;
        if link_abs.exists() {
            return Err(PathError::exists_already(link_abs).into());
        }
        unix::fs::symlink(target, &link_abs)?;
        Ok(link_abs)
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
        temp: PathBuf,
    }
    impl Setup {
        fn init() -> Self {
            let setup = Self { temp: PathBuf::from("tests/temp").abs().unwrap() };
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
    fn test_symlink() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("symlink");
        let file1 = tmpdir.join("file1");
        let link1 = tmpdir.join("link1");
        assert!(crate::remove_all(&tmpdir).is_ok());

        assert!(crate::mkdir_p(&tmpdir).is_ok());
        assert!(crate::touch(&file1).is_ok());
        assert!(crate::symlink(&link1, &file1).is_ok());
        assert_eq!(link1.exists(), true);

        // Clean up
        assert!(crate::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_touch() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("touch");
        let tmpfile = tmpdir.join("file1");
        assert!(crate::remove_all(&tmpdir).is_ok());

        assert!(crate::mkdir_p(&tmpdir).is_ok());
        assert!(crate::touch(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), true);

        // Clean up
        assert!(crate::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }
}
