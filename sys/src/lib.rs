use std::env;
use std::io;
use std::path::{Path, PathBuf};

// Returns the full path to the directory of the current running executable.
pub fn exec_dir() -> io::Result<PathBuf> {
    let path = env::current_exe()?;
    let dir = dirname(&path)?;
    Ok(dir)
}

// Returns the `Path` without its final component, if there is one.
pub fn dirname<T: AsRef<Path>>(path: &T) -> io::Result<PathBuf> {
    let parent = path.as_ref().parent().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Parent directory not found"))?;
    let dir = parent.to_path_buf();
    Ok(dir)
}

// Returns the final component of the `Path`, if there is one.
pub fn filename<T: AsRef<Path>>(path: &T) -> io::Result<&str> {
    let os_str = path.as_ref().file_name().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Filename not found"))?;
    let filename = os_str.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Unable to convert filename into String"))?;
    Ok(filename)
}

// PathBuf extensions
pub trait PathBufExt {
    fn dirname(&self) -> io::Result<PathBuf>;
    fn filename(&self) -> io::Result<&str>;
}
impl PathBufExt for PathBuf {
    // Returns the `Path` without its final component, if there is one.
    fn dirname(&self) -> io::Result<PathBuf> {
        return crate::dirname(self);
    }

    // Returns the final component of the `Path`, if there is one.
    fn filename(&self) -> io::Result<&str> {
        return crate::filename(self);
    }
}

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
    fn test_dirname() {
        let dir = PathBuf::from("/foo/bar");
        assert_eq!(PathBuf::from("/foo"), dirname(&dir).unwrap());
    }

    #[test]
    fn test_filename() {
        let path = PathBuf::from("/foo/bar");
        assert_eq!("bar", filename(&path).unwrap());
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
