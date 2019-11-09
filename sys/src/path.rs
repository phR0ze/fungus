use glob::glob;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

use errors::Result;

// Path utilities
// -------------------------------------------------------------------------------------------------

// Returns the full path to the directory of the current running executable.
pub fn exec_dir() -> Result<PathBuf> {
    let path = env::current_exe()?;
    let dir = dirname(&path)?;
    Ok(dir)
}

// Returns the full path to the current user's home directory.
pub fn home_dir() -> Result<PathBuf> {
    let os_str = env::var_os("HOME").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "User home directory not found"))?;
    let dir = PathBuf::from(os_str);
    Ok(dir)
}

// Returns the `Path` without its final component, if there is one.
pub fn dirname<T: AsRef<Path>>(path: &T) -> Result<PathBuf> {
    let parent = path.as_ref().parent().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Parent directory not found"))?;
    let dir = parent.to_path_buf();
    Ok(dir)
}

// Returns the final component of the `Path`, if there is one.
pub fn filename<T: AsRef<Path>>(path: &T) -> Result<&str> {
    let os_str = path.as_ref().file_name().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Filename not found"))?;
    let filename = os_str.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Unable to convert filename into str"))?;
    Ok(filename)
}

// Returns a vector of PathBuf or the first error it encountered.
pub fn getpaths<T: AsRef<Path>>(pattern: &T) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let _str = pattern.as_ref().to_str().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Unable to convert path into str"))?;
    for x in glob(_str)? {
        let path = x?;
        paths.push(path);
    }
    Ok(paths)
}

// PathBuf extensions
// -------------------------------------------------------------------------------------------------
pub trait PathBufExt {
    fn dirname(&self) -> Result<PathBuf>;
    fn filename(&self) -> Result<&str>;
}
impl PathBufExt for PathBuf {
    // Returns the `Path` without its final component, if there is one.
    fn dirname(&self) -> Result<PathBuf> {
        crate::dirname(self)
    }

    // Returns the final component of the `Path`, if there is one.
    fn filename(&self) -> Result<&str> {
        crate::filename(self)
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_dir() {
        let cwd = env::current_dir().unwrap();
        let dir = cwd.parent().unwrap().join("target/debug/deps");
        assert_eq!(dir, exec_dir().unwrap());
    }

    #[test]
    fn test_home_dir() {
        assert_eq!(PathBuf::from("/home"), dirname(&home_dir().unwrap()).unwrap());
    }

    #[test]
    fn test_dirname() {
        // test from PathBuf
        assert_eq!(PathBuf::from("/foo"), dirname(&PathBuf::from("/foo/bar")).unwrap());

        // test from string slice
        assert_eq!(PathBuf::from("/foo"), dirname(&"/foo/bar").unwrap());

        // test from String
        assert_eq!(PathBuf::from("/foo"), dirname(&String::from("/foo/bar")).unwrap());
    }

    #[test]
    fn test_filename() {
        let path = PathBuf::from("/foo/bar");
        assert_eq!("bar", filename(&path).unwrap());
    }

    #[test]
    fn test_getpaths() {
        let paths = getpaths(&"*").unwrap();
        assert_eq!(&PathBuf::from("Cargo.toml"), paths.first().unwrap());
        assert_eq!(&PathBuf::from("src"), paths.last().unwrap());
    }

    #[test]
    fn test_pathbufext_dirname() {
        let dir = PathBuf::from("/foo/bar");
        assert_eq!(PathBuf::from("/foo"), dir.dirname().unwrap());
    }

    #[test]
    fn test_pathbufext_filename() {
        let path = PathBuf::from("/foo/bar");
        assert_eq!("bar", path.filename().unwrap());
    }
}
