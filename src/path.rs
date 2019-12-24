use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

use crate::core::*;

/// Return the path in an absolute clean form
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let home = env::var("HOME").unwrap();
/// assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
/// ```
pub fn abs<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    let _path = path.as_ref();

    // Check for empty string
    if _path.empty() {
        return Err(PathError::empty().into());
    }

    // Expand home directory
    let mut path_buf = _path.expand()?;

    // Trim protocol prefix if needed
    path_buf = path_buf.trim_protocol()?;

    // Clean the resulting path
    path_buf = path_buf.clean()?;

    // Expand relative directories if needed
    if !path_buf.is_absolute() {
        let curr = env::current_dir()?;

        // Unwrap works here as there will always be Some
        path_buf = match path_buf.first()? {
            Component::CurDir => curr.join(path_buf),
            Component::ParentDir => curr.dir()?.join(path_buf.trim_first()?),
            _ => curr.join(path_buf),
        }
    }

    Ok(path_buf)
}

/// Returns all directories for the given path recurisely, sorted by filename. Handles path
/// expansion. Paths are returned as abs paths. Doesn't include the path itself. Paths are
/// guaranteed to be distinct.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
/// use fungus::core::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_all_dirs");
/// let dir1 = tmpdir.join("dir1");
/// let dir2 = dir1.join("dir2");
/// assert!(sys::mkdir_p(&dir2).is_ok());
/// assert_iter_eq(sys::all_dirs(&tmpdir).unwrap(), vec![dir1, dir2]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn all_dirs<T: AsRef<Path>>(path: T) -> Result<Vec<PathBuf>> {
    let abs = path.as_ref().abs()?;
    if abs.exists() {
        let mut paths: Vec<PathBuf> = Vec::new();
        let mut distinct = HashMap::<PathBuf, bool>::new();
        if abs.is_dir() {
            let mut first = true;
            for entry in WalkDir::new(&abs).follow_links(true) {
                // Skip the directory itself
                if first {
                    first = false;
                    continue;
                }
                let entry = entry?;
                let path = entry.path().abs()?;

                // Ensure the path is a directory and distinct
                if path.is_dir() {
                    if !distinct.contains_key(&path) {
                        distinct.insert(path.clone(), true);
                        paths.push(path);
                    }
                }
            }
            return Ok(paths);
        }
        return Err(PathError::is_not_dir(abs).into());
    }
    Err(PathError::does_not_exist(abs).into())
}

/// Returns all files for the given path recursively, sorted by filename. Handles path
/// expansion. Paths are returned as abs paths. Doesn't include the path itself. Paths are
/// guaranteed to be distinct.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
/// use fungus::core::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_all_files");
/// let file1 = tmpdir.join("file1");
/// let dir1 = tmpdir.join("dir1");
/// let file2 = dir1.join("file2");
/// assert!(sys::mkdir_p(&dir1).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::touch(&file2).is_ok());
/// assert_iter_eq(sys::all_files(&tmpdir).unwrap(), vec![file1, file2]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn all_files<T: AsRef<Path>>(path: T) -> Result<Vec<PathBuf>> {
    let abs = path.as_ref().abs()?;
    if abs.exists() {
        let mut paths: Vec<PathBuf> = Vec::new();
        let mut distinct = HashMap::<PathBuf, bool>::new();
        if abs.is_dir() {
            let mut first = true;
            for entry in WalkDir::new(&abs).follow_links(true) {
                // Skip the directory itself
                if first {
                    first = false;
                    continue;
                }
                let entry = entry?;
                let path = entry.path().abs()?;

                // Ensure the path is a directory and distinct
                if path.is_file() {
                    if !distinct.contains_key(&path) {
                        distinct.insert(path.clone(), true);
                        paths.push(path);
                    }
                }
            }
            return Ok(paths);
        }
        return Err(PathError::is_not_dir(abs).into());
    }
    Err(PathError::does_not_exist(abs).into())
}

/// Returns all paths for the given path recursively, sorted by filename. Handles path
/// expansion. Paths are returned as abs paths. Doesn't include the path itself. Paths are
/// guaranteed to be distinct.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
/// use fungus::core::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_all_paths");
/// let file1 = tmpdir.join("file1");
/// let dir1 = tmpdir.join("dir1");
/// let file2 = dir1.join("file2");
/// let file3 = dir1.join("file3");
/// assert!(sys::mkdir_p(&dir1).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::touch(&file2).is_ok());
/// assert!(sys::touch(&file3).is_ok());
/// assert_iter_eq(sys::all_paths(&tmpdir).unwrap(), vec![file1, dir1, file2, file3]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn all_paths<T: AsRef<Path>>(path: T) -> Result<Vec<PathBuf>> {
    let abs = path.as_ref().abs()?;
    if abs.exists() {
        let mut paths: Vec<PathBuf> = Vec::new();
        let mut distinct = HashMap::<PathBuf, bool>::new();
        if abs.is_dir() {
            let mut first = true;
            for entry in WalkDir::new(&abs).follow_links(true) {
                // Skip the directory itself
                if first {
                    first = false;
                    continue;
                }
                let entry = entry?;
                let path = entry.path().abs()?;

                // Ensure the path is a directory and distinct
                if !distinct.contains_key(&path) {
                    distinct.insert(path.clone(), true);
                    paths.push(path);
                }
            }
            return Ok(paths);
        }
        return Err(PathError::is_not_dir(abs).into());
    }
    Err(PathError::does_not_exist(abs).into())
}

/// Change the `Path` mode to the given mode and return the `Path`
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_chmod");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.join("file1");
/// assert!(sys::mkdir_p(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert_eq!(file1.mode().unwrap(), 0o100644);
/// assert!(sys::chmod(&file1, 0o555).is_ok());
/// assert_eq!(file1.mode().unwrap(), 0o100555);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn chmod<T: AsRef<Path>>(path: T, mode: u32) -> Result<PathBuf> {
    let abs = path.as_ref().abs()?;
    let perms = fs::Permissions::from_mode(mode);
    Ok(abs.setperms(perms)?)
}

/// Returns all directories for the given path, sorted by filename. Handles path expansion.
/// Paths are returned as abs paths. Doesn't include the path itself only its children nor
/// is this recursive.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
/// use fungus::core::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_dirs");
/// let dir1 = tmpdir.join("dir1");
/// let dir2 = tmpdir.join("dir2");
/// assert!(sys::mkdir_p(&dir1).is_ok());
/// assert!(sys::mkdir_p(&dir2).is_ok());
/// assert_iter_eq(sys::dirs(&tmpdir).unwrap(), vec![dir1, dir2]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn dirs<T: AsRef<Path>>(path: T) -> Result<Vec<PathBuf>> {
    let abs = path.as_ref().abs()?;
    if abs.exists() {
        if abs.is_dir() {
            let mut paths: Vec<PathBuf> = Vec::new();
            for entry in fs::read_dir(abs)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    paths.push(path.abs()?);
                }
            }
            paths.sort();
            return Ok(paths);
        }
        return Err(PathError::is_not_dir(abs).into());
    }
    Err(PathError::does_not_exist(abs).into())
}

/// Returns the full path to the directory of the current running executable.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let dir = env::current_exe().unwrap().dir().unwrap();
/// assert_eq!(sys::exec_dir().unwrap(), dir);
/// ```
pub fn exec_dir() -> Result<PathBuf> {
    Ok(env::current_exe()?.dir()?)
}

/// Returns the current running executable's name.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let base = env::current_exe().unwrap().base().unwrap();
/// assert_eq!(sys::exec_name().unwrap(), base);
/// ```
pub fn exec_name() -> Result<String> {
    Ok(env::current_exe()?.base()?)
}

/// Returns true if the given path exists. Handles path expansion.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// assert_eq!(sys::exists("/etc"), true);
/// ```
pub fn exists<T: AsRef<Path>>(path: T) -> bool {
    match metadata(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Returns all files for the given path, sorted by filename. Handles path expansion.
/// Paths are returned as abs paths. Doesn't include the path itself only its children nor
/// is this recursive.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
/// use fungus::core::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_files");
/// let file1 = tmpdir.join("file1");
/// let file2 = tmpdir.join("file2");
/// assert!(sys::mkdir_p(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::touch(&file2).is_ok());
/// assert_iter_eq(sys::files(&tmpdir).unwrap(), vec![file1, file2]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn files<T: AsRef<Path>>(path: T) -> Result<Vec<PathBuf>> {
    let abs = path.as_ref().abs()?;
    if abs.exists() {
        if abs.is_dir() {
            let mut paths: Vec<PathBuf> = Vec::new();
            for entry in fs::read_dir(abs)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    paths.push(path.abs()?);
                }
            }
            paths.sort();
            return Ok(paths);
        }
        return Err(PathError::is_not_dir(abs).into());
    }
    Err(PathError::does_not_exist(abs).into())
}

/// Returns true if the given path exists and is a directory. Handles path expansion.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// assert_eq!(sys::is_dir("/etc"), true);
/// ```
pub fn is_dir<T: AsRef<Path>>(path: T) -> bool {
    match metadata(path) {
        Ok(x) => x.is_dir(),
        Err(_) => false,
    }
}

/// Returns true if the given path exists and is a file. Handles path expansion
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// assert_eq!(sys::is_file("/etc/hosts"), true);
/// ```
pub fn is_file<T: AsRef<Path>>(path: T) -> bool {
    match metadata(path) {
        Ok(x) => x.is_file(),
        Err(_) => false,
    }
}

/// Returns true if the given path exists and is a symlink. Handles path expansion
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_is_symlink");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.join("file1");
/// let link1 = tmpdir.join("link1");
/// assert!(sys::mkdir_p(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::symlink(&link1, &file1).is_ok());
/// assert_eq!(sys::is_symlink(link1), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn is_symlink<T: AsRef<Path>>(path: T) -> bool {
    readlink(path).is_ok()
}

/// Returns true if the given path exists and is a symlinked directory. Handles path
/// expansion
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_is_symlink_dir");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let dir1 = tmpdir.join("dir1");
/// let link1 = tmpdir.join("link1");
/// assert!(sys::mkdir_p(&dir1).is_ok());
/// assert!(sys::symlink(&link1, &dir1).is_ok());
/// assert_eq!(sys::is_symlink_dir(link1), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn is_symlink_dir<T: AsRef<Path>>(path: T) -> bool {
    match readlink(path) {
        Ok(x) => x.is_dir(),
        Err(_) => false,
    }
}

/// Returns true if the given path exists and is a symlinked file. Handles path
/// expansion
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_is_symlink_file");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.join("file1");
/// let link1 = tmpdir.join("link1");
/// assert!(sys::mkdir_p(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::symlink(&link1, &file1).is_ok());
/// assert_eq!(sys::is_symlink_file(link1), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn is_symlink_file<T: AsRef<Path>>(path: T) -> bool {
    match readlink(path) {
        Ok(x) => x.is_file(),
        Err(_) => false,
    }
}

/// Returns a vector of all paths from the given target glob with path expansion and sorted by
/// name. Doesn't include the target itself only its children nor is this recursive.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
/// use fungus::core::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_glob");
/// let dir1 = tmpdir.join("dir1");
/// let dir2 = tmpdir.join("dir2");
/// let file1 = tmpdir.join("file1");
/// assert!(sys::mkdir_p(&dir1).is_ok());
/// assert!(sys::mkdir_p(&dir2).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert_iter_eq(sys::glob(tmpdir.join("*")).unwrap(), vec![dir1, dir2, file1]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn glob<T: AsRef<Path>>(pattern: T) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let _str = pattern.as_ref().to_string()?;
    for x in glob::glob(&_str)? {
        paths.push(x?.abs()?);
    }
    Ok(paths)
}

/// Returns the Metadata object for the `Path` if it exists else an error. Handls path
/// expansion.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let meta = sys::metadata(Path::new("/etc")).unwrap();
/// assert_eq!(meta.is_dir(), true);
/// ```
pub fn metadata<T: AsRef<Path>>(path: T) -> Result<fs::Metadata> {
    //let abs = path.as_ref().abs()?;
    let abs = path.as_ref();
    let meta = fs::metadata(abs)?;
    Ok(meta)
}

/// Returns all directories/files for the given path, sorted by filename. Handles path
/// expansion. Paths are returned as abs paths. Doesn't include the path itself only
/// its children nor is this recursive.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
/// use fungus::core::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_paths");
/// let dir1 = tmpdir.join("dir1");
/// let dir2 = tmpdir.join("dir2");
/// let file1 = tmpdir.join("file1");
/// assert!(sys::mkdir_p(&dir1).is_ok());
/// assert!(sys::mkdir_p(&dir2).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert_iter_eq(sys::paths(&tmpdir).unwrap(), vec![dir1, dir2, file1]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn paths<T: AsRef<Path>>(path: T) -> Result<Vec<PathBuf>> {
    let abs = path.as_ref().abs()?;
    if abs.exists() {
        if abs.is_dir() {
            let mut paths: Vec<PathBuf> = Vec::new();
            for entry in fs::read_dir(abs)? {
                let entry = entry?;
                let path = entry.path();
                paths.push(path.abs()?);
            }
            paths.sort();
            return Ok(paths);
        }
        return Err(PathError::is_not_dir(abs).into());
    }
    Err(PathError::does_not_exist(abs).into())
}

/// Returns the absolute path for the given link target. Handles path expansion
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_readlink");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.join("file1");
/// let link1 = tmpdir.join("link1");
/// assert!(sys::mkdir_p(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::symlink(&link1, &file1).is_ok());
/// assert_eq!(sys::readlink(link1).unwrap(), file1);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn readlink<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    let abs = path.as_ref().abs()?;
    let abs = fs::read_link(abs)?;
    Ok(abs)
}

// Path extensions
// -------------------------------------------------------------------------------------------------
pub trait PathExt {
    /// Return the path in an absolute clean form
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let home = env::var("HOME").unwrap();
    /// assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
    /// ```
    fn abs(&self) -> Result<PathBuf>;

    /// Returns the absolute `Path` based on the given absolute `Path`. The last element of the
    /// given path will be assumed to be a file name.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let home = PathBuf::from("~").abs().unwrap();
    /// assert_eq!(PathBuf::from("foo2").abs_from(home.join("foo1").abs().unwrap()).unwrap(), home.join("foo2"));
    /// ```
    fn abs_from<T: AsRef<Path>>(&self, path: T) -> Result<PathBuf>;

    /// Returns the final component of the `Path`, if there is one.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!("bar", PathBuf::from("/foo/bar").base().unwrap());
    /// ```
    fn base(&self) -> Result<String>;

    /// Set the given mode for the `Path` and return the `Path`
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("pathbuf_doc_chmod");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.join("file1");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(file1.chmod(0o644).is_ok());
    /// assert_eq!(file1.mode().unwrap(), 0o100644);
    /// assert!(file1.chmod(0o555).is_ok());
    /// assert_eq!(file1.mode().unwrap(), 0o100555);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn chmod(&self, mode: u32) -> Result<PathBuf>;

    /// Return the shortest path equivalent to the path by purely lexical processing and thus does not handle
    /// links correctly in some cases, use canonicalize in those cases. It applies the following rules
    /// interatively until no further processing can be done.
    ///
    ///	1. Replace multiple slashes with a single
    ///	2. Eliminate each . path name element (the current directory)
    ///	3. Eliminate each inner .. path name element (the parent directory)
    ///	   along with the non-.. element that precedes it.
    ///	4. Eliminate .. elements that begin a rooted path:
    ///	   that is, replace "/.." by "/" at the beginning of a path.
    /// 5. Leave intact ".." elements that begin a non-rooted path.
    /// 6. Drop trailing '/' unless it is the root
    ///
    /// If the result of this process is an empty string, return the string `.`, representing the current directory.
    fn clean(&self) -> Result<PathBuf>;

    /// Returns the `Path` without its final component, if there is one.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let dir = PathBuf::from("/foo/bar").dir().unwrap();
    /// assert_eq!(PathBuf::from("/foo").as_path(), dir);
    /// ```
    fn dir(&self) -> Result<PathBuf>;

    /// Returns true if the `Path` is empty.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(PathBuf::from("").empty(), true);
    /// ```
    fn empty(&self) -> bool;

    /// Returns true if the `Path` exists. Handles path expansion.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(Path::new("/etc").exists(), true);
    /// ```
    fn exists<T: AsRef<Path>>(path: T) -> bool;

    /// Expand the path to include the home prefix if necessary
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let home = env::var("HOME").unwrap();
    /// assert_eq!(PathBuf::from(&home).join("foo"), PathBuf::from("~/foo").expand().unwrap());
    /// ```
    fn expand(&self) -> Result<PathBuf>;

    /// Returns the first path component.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let first = Component::Normal(OsStr::new("foo"));
    /// assert_eq!(PathBuf::from("foo/bar").first().unwrap(), first);
    /// ```
    fn first(&self) -> Result<Component>;

    /// Returns true if the `Path` as a String contains the given string
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let path = PathBuf::from("/foo/bar");
    /// assert_eq!(path.has("foo"), true);
    /// assert_eq!(path.has("/foo"), true);
    /// ```
    fn has<T: AsRef<str>>(&self, value: T) -> bool;

    /// Returns true if the `Path` as a String has the given string prefix
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let path = PathBuf::from("/foo/bar");
    /// assert_eq!(path.has_prefix("/foo"), true);
    /// assert_eq!(path.has_prefix("foo"), false);
    /// ```
    fn has_prefix<T: AsRef<str>>(&self, value: T) -> bool;

    /// Returns true if the `Path` as a String has the given string suffix
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let path = PathBuf::from("/foo/bar");
    /// assert_eq!(path.has_suffix("/bar"), true);
    /// assert_eq!(path.has_suffix("foo"), false);
    /// ```
    fn has_suffix<T: AsRef<str>>(&self, value: T) -> bool;

    /// Returns true if the `Path` exists and is a directory. Handles path expansion.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(Path::new("/etc").is_dir(), true);
    /// ```
    fn is_dir(&self) -> bool;

    /// Returns true if the `Path` exists and is a file. Handles path expansion
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(Path::new("/etc/hosts").is_file(), true);
    /// ```
    fn is_file(&self) -> bool;

    /// Returns true if the `Path` exists and is a symlink. Handles path expansion
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("pathbuf_doc_is_symlink");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.join("file1");
    /// let link1 = tmpdir.join("link1");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(sys::symlink(&link1, &file1).is_ok());
    /// assert_eq!(link1.is_symlink(), true);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn is_symlink(&self) -> bool;

    /// Returns true if the `Path` exists and is a symlinked directory. Handles path expansion
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("pathbuf_doc_is_symlink_dir");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let dir1 = tmpdir.join("dir1");
    /// let link1 = tmpdir.join("link1");
    /// assert!(sys::mkdir_p(&dir1).is_ok());
    /// assert!(sys::symlink(&link1, &dir1).is_ok());
    /// assert_eq!(link1.is_symlink_dir(), true);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn is_symlink_dir(&self) -> bool;

    /// Returns true if the given `Path` exists and is a symlinked file. Handles path
    /// expansion
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("pathbuf_doc_is_symlink_file");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.join("file1");
    /// let link1 = tmpdir.join("link1");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(sys::symlink(&link1, &file1).is_ok());
    /// assert_eq!(link1.is_symlink_file(), true);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn is_symlink_file(&self) -> bool;

    /// Returns the last path component.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let first = Component::Normal(OsStr::new("bar"));
    /// assert_eq!(PathBuf::from("foo/bar").last().unwrap(), first);
    /// ```
    fn last(&self) -> Result<Component>;

    /// Returns the Metadata object for the `Path` if it exists else and error
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let meta = Path::new("/etc").metadata().unwrap();
    /// assert_eq!(meta.is_dir(), true);
    /// ```
    fn metadata(&self) -> Result<fs::Metadata>;

    /// Returns the Metadata object for the `Path` if it exists else and error
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("pathbuf_doc_mode");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.join("file1");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(file1.chmod(0o644).is_ok());
    /// assert_eq!(file1.mode().unwrap(), 0o100644);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn mode(&self) -> Result<u32>;

    /// Return the permissions for the `Path`
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("pathbuf_doc_perms");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.join("file1");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(file1.chmod(0o644).is_ok());
    /// assert_eq!(file1.perms().unwrap().mode(), 0o100644);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn perms(&self) -> Result<fs::Permissions>;

    /// Returns the absolute path for the link target. Handles path expansion
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("pathbuf_doc_readlink");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.join("file1");
    /// let link1 = tmpdir.join("link1");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(sys::symlink(&link1, &file1).is_ok());
    /// assert_eq!(link1.readlink().unwrap(), file1);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn readlink(&self) -> Result<PathBuf>;

    /// Returns the `Path` relative to the given `Path`
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(PathBuf::from("foo/bar1").relative_from("foo/bar2").unwrap(), PathBuf::from("bar1"));
    /// ```
    fn relative_from<T: AsRef<Path>>(&self, path: T) -> Result<PathBuf>;

    /// Set the given permissions on the `Path` and return the `Path`
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("pathbuf_doc_setperms");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.join("file1");
    /// assert!(sys::mkdir_p(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(file1.chmod(0o644).is_ok());
    /// assert_eq!(file1.perms().unwrap().mode(), 0o100644);
    /// assert!(file1.setperms(fs::Permissions::from_mode(0o555)).is_ok());
    /// assert_eq!(file1.perms().unwrap().mode(), 0o100555);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn setperms(&self, perms: fs::Permissions) -> Result<PathBuf>;

    /// Returns the `Path` as a String
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(PathBuf::from("/foo").to_string().unwrap(), "/foo".to_string());
    /// ```
    fn to_string(&self) -> Result<String>;

    /// Returns the `Path` with the file extension removed
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(PathBuf::from("foo.exe").trim_ext().unwrap(), PathBuf::from("foo"));
    /// ```
    fn trim_ext(&self) -> Result<PathBuf>;

    /// Returns the `Path` with the first component trimmed off
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(PathBuf::from("/foo").trim_first().unwrap(), PathBuf::from("foo"));
    /// ```
    fn trim_first(&self) -> Result<PathBuf>;

    /// Returns the `Path` with the last component trimmed off
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(PathBuf::from("/foo").trim_last().unwrap(), PathBuf::from("/"));
    /// ```
    fn trim_last(&self) -> Result<PathBuf>;

    /// Returns the `Path` with the given prefix trimmed off else the original `Path`.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(Path::new("/foo/bar").trim_prefix("/foo").unwrap(), PathBuf::from("/bar"));
    /// ```
    fn trim_prefix<T: AsRef<Path>>(&self, prefix: T) -> Result<PathBuf>;

    /// Returns the `Path` with well known protocol prefixes removed.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(PathBuf::from("ftp://foo").trim_protocol().unwrap(), PathBuf::from("foo"));
    /// ```
    fn trim_protocol(&self) -> Result<PathBuf>;

    /// Returns the `Path` with the given suffix trimmed off else the original `Path`.
    ///
    /// ### Examples
    /// ```
    /// use fungus::presys::*;
    ///
    /// assert_eq!(PathBuf::from("/foo/bar").trim_suffix("/bar").unwrap(), PathBuf::from("/foo"));
    /// ```
    fn trim_suffix<T: AsRef<Path>>(&self, value: T) -> Result<PathBuf>;
}

impl PathExt for Path {
    fn abs(&self) -> Result<PathBuf> {
        abs(self)
    }

    fn abs_from<T: AsRef<Path>>(&self, base: T) -> Result<PathBuf> {
        let base = base.as_ref().abs()?;
        if !self.is_absolute() && self != base {
            let mut path = base.trim_last()?;
            let mut components = self.components();
            loop {
                match components.next() {
                    Some(component) => match component {
                        Component::ParentDir => path = path.trim_last()?,
                        Component::Normal(x) => return Ok(path.join(x).join(components.collect::<PathBuf>()).clean()?),
                        _ => (),
                    },
                    None => return Err(PathError::empty().into()),
                }
            }
        }
        Ok(self.to_path_buf())
    }

    fn base(&self) -> Result<String> {
        let os_str = self.file_name().ok_or_else(|| PathError::filename_not_found(self))?;
        let filename = os_str.to_str().ok_or_else(|| PathError::failed_to_string(self))?;
        Ok(String::from(filename))
    }

    fn chmod(&self, mode: u32) -> Result<PathBuf> {
        Ok(chmod(self, mode)?)
    }

    fn clean(&self) -> Result<PathBuf> {
        // Components already handles the following cases:
        // 1. Repeated separators are ignored, so a/b and a//b both have a and b as components.
        // 2. Occurrences of . are normalized away, except if they are at the beginning of the path.
        //    e.g. a/./b, a/b/, a/b/. and a/b all have a and b as components, but ./a/b starts with an additional CurDir component.
        // 6. A trailing slash is normalized away, /a/b and /a/b/ are equivalent.
        let mut cnt = 0;
        let mut prev = None;
        let mut path_buf = PathBuf::new();
        for component in self.components() {
            match component {
                // 2. Eliminate . path name at begining of path for simplicity
                x if x == Component::CurDir && cnt == 0 => continue,

                // 5. Leave .. begining non rooted path
                x if x == Component::ParentDir && cnt > 0 && !prev.has(&Component::ParentDir) => {
                    match prev.unwrap() {
                        // 4. Eliminate .. elements that begin a root path
                        Component::RootDir => (),

                        // 3. Eliminate inner .. path name elements
                        Component::Normal(_) => {
                            cnt -= 1;
                            path_buf.pop();
                            prev = path_buf.components().last();
                        }
                        _ => (),
                    }
                    continue;
                }

                // Normal
                _ => {
                    cnt += 1;
                    path_buf.push(component);
                    prev = Some(component);
                }
            };
        }

        // Ensure if empty the current dir is returned
        if path_buf.empty() {
            path_buf.push(".");
        }
        Ok(path_buf)
    }

    fn dir(&self) -> Result<PathBuf> {
        let dir = self.parent().ok_or_else(|| PathError::parent_not_found(self))?;
        Ok(dir.to_path_buf())
    }

    fn empty(&self) -> bool {
        match self.to_string() {
            Ok(s) => s == "",
            Err(_) => false,
        }
    }

    fn exists<T: AsRef<Path>>(path: T) -> bool {
        exists(path)
    }

    fn expand(&self) -> Result<PathBuf> {
        let path_str = self.to_string()?;
        let mut expanded = self.to_path_buf();

        // Check for invalid home expansion
        match path_str.matches("~").count() {
            // Only home expansion at the begining of the path is allowed
            cnt if cnt > 1 => return Err(PathError::multiple_home_symbols(self).into()),

            // Invalid home expansion requested
            cnt if cnt == 1 && !self.has_prefix("~/") && path_str != "~" => {
                return Err(PathError::invalid_expansion(self).into());
            }

            // Single tilda only
            cnt if cnt == 1 && path_str == "~" => {
                expanded = crate::user::home()?;
            }

            // Replace prefix with home directory
            1 => expanded = crate::user::home()?.join(&path_str[2..]),
            _ => (),
        }

        Ok(expanded)
    }

    fn first(&self) -> Result<Component> {
        self.components().first_result()
    }

    fn has<T: AsRef<str>>(&self, value: T) -> bool {
        match self.to_string() {
            Ok(s) => s.contains(value.as_ref()),
            Err(_) => false,
        }
    }

    fn has_prefix<T: AsRef<str>>(&self, value: T) -> bool {
        match self.to_string() {
            Ok(s) => s.starts_with(value.as_ref()),
            Err(_) => false,
        }
    }

    fn has_suffix<T: AsRef<str>>(&self, value: T) -> bool {
        match self.to_string() {
            Ok(s) => s.ends_with(value.as_ref()),
            Err(_) => false,
        }
    }

    fn is_dir(&self) -> bool {
        is_dir(self)
    }

    fn is_file(&self) -> bool {
        is_file(self)
    }

    fn is_symlink(&self) -> bool {
        is_symlink(self)
    }

    fn is_symlink_dir(&self) -> bool {
        is_symlink_dir(self)
    }

    fn is_symlink_file(&self) -> bool {
        is_symlink_file(self)
    }

    fn last(&self) -> Result<Component> {
        self.components().last_result()
    }
    fn metadata(&self) -> Result<fs::Metadata> {
        let meta = fs::metadata(self)?;
        Ok(meta)
    }

    fn mode(&self) -> Result<u32> {
        let perms = self.perms()?;
        Ok(perms.mode())
    }

    fn perms(&self) -> Result<fs::Permissions> {
        Ok(self.metadata()?.permissions())
    }

    fn readlink(&self) -> Result<PathBuf> {
        readlink(self)
    }

    fn relative_from<T: AsRef<Path>>(&self, base: T) -> Result<PathBuf> {
        let path = self.abs()?;
        let base = base.as_ref().abs()?;
        if path != base {
            let mut x = path.components();
            let mut y = base.components();
            let mut comps: Vec<Component> = vec![];
            loop {
                match (x.next(), y.next()) {
                    (None, None) => break,
                    (Some(a), None) => {
                        comps.push(a);
                        comps.extend(x.by_ref());
                        break;
                    }
                    (None, _) => comps.push(Component::ParentDir),
                    (Some(a), Some(b)) if comps.is_empty() && a == b => (),
                    (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
                    (Some(_), Some(b)) if b == Component::ParentDir => return Ok(path),
                    (Some(a), Some(_)) => {
                        for _ in y {
                            comps.push(Component::ParentDir);
                        }
                        comps.push(a);
                        comps.extend(x.by_ref());
                        break;
                    }
                }
            }
            return Ok(comps.iter().collect::<PathBuf>());
        }
        Ok(path)
    }

    fn setperms(&self, perms: fs::Permissions) -> Result<PathBuf> {
        fs::set_permissions(&self, perms)?;
        Ok(self.to_path_buf())
    }

    fn to_string(&self) -> Result<String> {
        let _str = self.to_str().ok_or_else(|| PathError::failed_to_string(self))?;
        Ok(String::from(_str))
    }

    fn trim_ext(&self) -> Result<PathBuf> {
        match self.file_stem() {
            Some(val) => Ok(self.trim_last()?.join(val)),
            None => Ok(self.to_path_buf()),
        }
    }

    fn trim_first(&self) -> Result<PathBuf> {
        Ok(self.components().drop(1).as_path().to_path_buf())
    }

    fn trim_last(&self) -> Result<PathBuf> {
        Ok(self.components().drop(-1).as_path().to_path_buf())
    }

    fn trim_prefix<T: AsRef<Path>>(&self, prefix: T) -> Result<PathBuf> {
        let base = self.to_string()?;
        let prefix = prefix.as_ref().to_string()?;
        if base.starts_with(&prefix) {
            if prefix.len() < base.len() {
                let new = &base[prefix.len()..];
                return Ok(PathBuf::from(new));
            }
            return Ok(PathBuf::new());
        }
        Ok(self.to_path_buf())
    }

    fn trim_protocol(&self) -> Result<PathBuf> {
        let mut base = self.to_string()?;
        if let Some(i) = base.find("//") {
            let (prefix, suffix) = base.split_at(i + 2);
            let lower = prefix.to_lowercase();
            let lower = lower.trim_start_matches("file://");
            let lower = lower.trim_start_matches("ftp://");
            let lower = lower.trim_start_matches("http://");
            let lower = lower.trim_start_matches("https://");
            if lower != "" {
                base = format!("{}{}", prefix, suffix);
            } else {
                return Ok(PathBuf::from(suffix));
            }
        }
        Ok(PathBuf::from(base))
    }

    fn trim_suffix<T: AsRef<Path>>(&self, suffix: T) -> Result<PathBuf> {
        let base = self.to_string()?;
        let suffix = suffix.as_ref().to_string()?;
        if base.ends_with(&suffix) {
            let new = &base[..base.len() - suffix.len()];
            return Ok(PathBuf::from(new));
        }
        Ok(self.to_path_buf())
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::presys::*;

    // Reusable teset setup
    struct Setup {
        temp: PathBuf,
    }
    impl Setup {
        fn init() -> Self {
            let setup = Self { temp: PathBuf::from("tests/temp").abs().unwrap() };
            sys::mkdir_p(&setup.temp).unwrap();
            setup
        }
    }

    #[test]
    fn test_abs() {
        let home = PathBuf::from(env::var("HOME").unwrap());
        let cwd = env::current_dir().unwrap();
        let prev = cwd.dir().unwrap();

        // expand previous directory and drop trailing slashes
        assert_eq!(sys::abs("..//").unwrap(), prev);
        assert_eq!(sys::abs("../").unwrap(), prev);
        assert_eq!(sys::abs("..").unwrap(), prev);

        // expand current directory and drop trailing slashes
        assert_eq!(sys::abs(".//").unwrap(), cwd);
        assert_eq!(sys::abs("./").unwrap(), cwd);
        assert_eq!(sys::abs(".").unwrap(), cwd);

        // home dir
        assert_eq!(sys::abs("~").unwrap(), home);
        assert_eq!(sys::abs("~/").unwrap(), home);

        // expand relative directory
        assert_eq!(sys::abs("foo").unwrap(), cwd.join("foo"));

        // expand home path
        assert_eq!(sys::abs("~/foo").unwrap(), home.join("foo"));

        // More complicated
        assert_eq!(sys::abs("~/foo/bar/../.").unwrap(), home.join("foo"));
        assert_eq!(sys::abs("~/foo/bar/../").unwrap(), home.join("foo"));
        assert_eq!(sys::abs("~/foo/bar/../blah").unwrap(), home.join("foo/blah"));
    }

    #[test]
    fn test_abs_from() {
        let home = PathBuf::from("~").abs().unwrap();

        // share the same directory
        assert_eq!(PathBuf::from("foo2").abs_from(home.join("foo1").abs().unwrap()).unwrap(), home.join("foo2"));
        assert_eq!(PathBuf::from("./foo2").abs_from(home.join("foo1").abs().unwrap()).unwrap(), home.join("foo2"));

        // share parent directory
        assert_eq!(PathBuf::from("../foo2").abs_from(home.join("bar1/foo1").abs().unwrap()).unwrap(), home.join("foo2"));
        assert_eq!(PathBuf::from("bar2/foo2").abs_from(home.join("bar1/foo1").abs().unwrap()).unwrap(), home.join("bar1/bar2/foo2"));
        assert_eq!(PathBuf::from("../../foo2").abs_from(home.join("bar1/foo1").abs().unwrap()).unwrap(), home.trim_last().unwrap().join("foo2"));

        // share grandparent directory
        assert_eq!(PathBuf::from("blah1/bar2/foo2").abs_from(home.join("bar1/foo1").abs().unwrap()).unwrap(), home.join("bar1/blah1/bar2/foo2"));
    }

    #[test]
    fn test_all_dirs() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("all_dirs");
        let tmpdir1 = tmpdir.join("dir1");
        let tmpdir2 = tmpdir1.join("dir2");
        let tmpfile1 = tmpdir.join("file1");
        let tmpfile2 = tmpdir.join("file2");

        // Create the dirs and files
        assert!(sys::mkdir_p(&tmpdir1).is_ok());
        assert!(sys::mkdir_p(&tmpdir2).is_ok());
        assert_eq!(tmpdir.is_dir(), true);
        assert_eq!(tmpdir.is_file(), false);
        assert_eq!(tmpdir1.is_dir(), true);
        assert_eq!(tmpdir2.is_dir(), true);
        assert!(sys::touch(&tmpfile1).is_ok());
        assert_eq!(tmpfile1.is_dir(), false);
        assert_eq!(tmpfile1.is_file(), true);
        assert!(sys::touch(&tmpfile2).is_ok());
        assert_eq!(tmpfile2.is_dir(), false);
        assert_eq!(tmpfile2.is_file(), true);

        // Validate the the all_dirs function gives me the correct dirs in order
        let dirs = sys::all_dirs(&tmpdir).unwrap();
        assert_iter_eq(dirs, vec![tmpdir1, tmpdir2]);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_all_files() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("all_files");
        let tmpdir1 = tmpdir.join("dir1");
        let tmpdir2 = tmpdir1.join("dir2");
        let tmpfile1 = tmpdir1.join("file1");
        let tmpfile2 = tmpdir2.join("file2");

        // Create the dirs and files
        assert!(sys::mkdir_p(&tmpdir1).is_ok());
        assert!(sys::mkdir_p(&tmpdir2).is_ok());
        assert_eq!(tmpdir.is_dir(), true);
        assert_eq!(tmpdir.is_file(), false);
        assert_eq!(tmpdir1.is_dir(), true);
        assert_eq!(tmpdir2.is_dir(), true);
        assert!(sys::touch(&tmpfile1).is_ok());
        assert_eq!(tmpfile1.is_dir(), false);
        assert_eq!(tmpfile1.is_file(), true);
        assert!(sys::touch(&tmpfile2).is_ok());
        assert_eq!(tmpfile2.is_dir(), false);
        assert_eq!(tmpfile2.is_file(), true);

        // Validate the the all_files function gives me the correct files in order
        let files = sys::all_files(&tmpdir).unwrap();
        assert_iter_eq(files, vec![tmpfile2, tmpfile1]);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_all_paths() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("all_paths");
        let tmpdir1 = tmpdir.join("dir1");
        let tmpdir2 = tmpdir1.join("dir2");
        let tmpfile1 = tmpdir1.join("file1");
        let tmpfile2 = tmpdir2.join("file2");

        // Create the dirs and files
        assert!(sys::mkdir_p(&tmpdir1).is_ok());
        assert!(sys::mkdir_p(&tmpdir2).is_ok());
        assert_eq!(tmpdir.is_dir(), true);
        assert_eq!(tmpdir.is_file(), false);
        assert_eq!(tmpdir1.is_dir(), true);
        assert_eq!(tmpdir2.is_dir(), true);
        assert!(sys::touch(&tmpfile1).is_ok());
        assert_eq!(tmpfile1.is_dir(), false);
        assert_eq!(tmpfile1.is_file(), true);
        assert!(sys::touch(&tmpfile2).is_ok());
        assert_eq!(tmpfile2.is_dir(), false);
        assert_eq!(tmpfile2.is_file(), true);

        // Validate the the all_paths function gives me the correct paths in order
        let paths = sys::all_paths(&tmpdir).unwrap();
        assert_iter_eq(paths, vec![tmpdir1, tmpdir2, tmpfile2, tmpfile1]);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_dirs() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("dirs");
        let tmpdir1 = tmpdir.join("dir1");
        let tmpdir2 = tmpdir.join("dir2");
        let tmpfile1 = tmpdir.join("file1");
        let tmpfile2 = tmpdir.join("file2");

        // Create the dirs and files
        assert!(sys::mkdir_p(&tmpdir1).is_ok());
        assert!(sys::mkdir_p(&tmpdir2).is_ok());
        assert_eq!(tmpdir.is_dir(), true);
        assert_eq!(tmpdir.is_file(), false);
        assert_eq!(tmpdir1.is_dir(), true);
        assert_eq!(tmpdir2.is_dir(), true);
        assert!(sys::touch(&tmpfile1).is_ok());
        assert_eq!(tmpfile1.is_dir(), false);
        assert_eq!(tmpfile1.is_file(), true);
        assert!(sys::touch(&tmpfile2).is_ok());
        assert_eq!(tmpfile2.is_dir(), false);
        assert_eq!(tmpfile2.is_file(), true);

        // Validate the the dirs function gives me the correct dirs without the files and in order
        let dirs = sys::dirs(&tmpdir).unwrap();
        assert_iter_eq(dirs, vec![tmpdir1, tmpdir2]);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_exec_dir() {
        let cwd = env::current_dir().unwrap();
        let dir = cwd.join("target/debug/deps");
        assert_eq!(sys::exec_dir().unwrap(), dir);
    }

    #[test]
    fn test_exec_name() {
        let exec_path = env::current_exe().unwrap();
        let name = exec_path.base().unwrap();
        assert_eq!(name, sys::exec_name().unwrap());
    }

    #[test]
    fn test_files() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("files");
        let tmpdir1 = tmpdir.join("dir1");
        let tmpdir2 = tmpdir.join("dir2");
        let tmpfile1 = tmpdir.join("file1");
        let tmpfile2 = tmpdir.join("file2");

        // Create the dirs and files
        assert!(sys::mkdir_p(&tmpdir1).is_ok());
        assert!(sys::mkdir_p(&tmpdir2).is_ok());
        assert_eq!(tmpdir.is_dir(), true);
        assert_eq!(tmpdir.is_file(), false);
        assert_eq!(tmpdir1.is_dir(), true);
        assert_eq!(tmpdir2.is_dir(), true);
        assert!(sys::touch(&tmpfile1).is_ok());
        assert_eq!(tmpfile1.is_dir(), false);
        assert_eq!(tmpfile1.is_file(), true);
        assert!(sys::touch(&tmpfile2).is_ok());
        assert_eq!(tmpfile2.is_dir(), false);
        assert_eq!(tmpfile2.is_file(), true);

        // Validate the the files function gives me the correct files without the dirs and in order
        let files = sys::files(&tmpdir).unwrap();
        assert_iter_eq(files, vec![tmpfile1, tmpfile2]);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_is_dir() {
        let setup = Setup::init();
        assert_eq!(sys::is_dir("."), true);
        assert_eq!(sys::is_dir(setup.temp), true);
        assert_eq!(sys::is_dir("/foobar"), false);
    }

    #[test]
    fn test_is_file() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("is_file");
        let tmpfile = tmpdir.join("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert!(sys::touch(&tmpfile).is_ok());
        assert_eq!(tmpfile.is_file(), true);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_is_symlink() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("is_symlink");
        let file1 = tmpdir.join("file1");
        let link1 = tmpdir.join("link1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(sys::symlink(&link1, &file1).is_ok());
        assert_eq!(sys::is_symlink(link1), true);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_is_symlink_dir() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("is_symlink_dir");
        let dir1 = tmpdir.join("dir1");
        let link1 = tmpdir.join("link1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir_p(&dir1).is_ok());
        assert!(sys::symlink(&link1, &dir1).is_ok());
        assert_eq!(sys::is_symlink_dir(&link1), true);
        assert_eq!(sys::is_symlink_file(&link1), false);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_is_symlink_file() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("is_symlink_file");
        let file1 = tmpdir.join("file1");
        let link1 = tmpdir.join("link1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(sys::symlink(&link1, &file1).is_ok());
        assert_eq!(sys::is_symlink_file(&link1), true);
        assert_eq!(sys::is_symlink_dir(&link1), false);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_metadata() {
        let setup = Setup::init();
        let meta = sys::metadata(setup.temp).unwrap();
        assert_eq!(meta.is_dir(), true);
    }

    #[test]
    fn test_glob() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("glob");
        let tmpdir1 = tmpdir.join("dir1");
        let tmpdir2 = tmpdir.join("dir2");
        let tmpfile1 = tmpdir.join("file1");
        let tmpfile2 = tmpdir.join("file2");

        // Create the dirs and files
        assert!(sys::mkdir_p(&tmpdir1).is_ok());
        assert!(sys::mkdir_p(&tmpdir2).is_ok());
        assert_eq!(tmpdir.is_dir(), true);
        assert_eq!(tmpdir.is_file(), false);
        assert_eq!(tmpdir1.is_dir(), true);
        assert_eq!(tmpdir2.is_dir(), true);
        assert!(sys::touch(&tmpfile1).is_ok());
        assert_eq!(tmpfile1.is_dir(), false);
        assert_eq!(tmpfile1.is_file(), true);
        assert!(sys::touch(&tmpfile2).is_ok());
        assert_eq!(tmpfile2.is_dir(), false);
        assert_eq!(tmpfile2.is_file(), true);

        // Validate the the files function gives me the correct files without the dirs and in order
        let paths = sys::glob(tmpdir.join("*")).unwrap();
        assert_iter_eq(paths, vec![tmpdir1, tmpdir2, tmpfile1, tmpfile2]);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_paths() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("paths");
        let tmpdir1 = tmpdir.join("dir1");
        let tmpdir2 = tmpdir.join("dir2");
        let tmpfile1 = tmpdir.join("file1");
        let tmpfile2 = tmpdir.join("file2");

        // Create the dirs and files
        assert!(sys::mkdir_p(&tmpdir1).is_ok());
        assert!(sys::mkdir_p(&tmpdir2).is_ok());
        assert_eq!(tmpdir.is_dir(), true);
        assert_eq!(tmpdir.is_file(), false);
        assert_eq!(tmpdir1.is_dir(), true);
        assert_eq!(tmpdir2.is_dir(), true);
        assert!(sys::touch(&tmpfile1).is_ok());
        assert_eq!(tmpfile1.is_dir(), false);
        assert_eq!(tmpfile1.is_file(), true);
        assert!(sys::touch(&tmpfile2).is_ok());
        assert_eq!(tmpfile2.is_dir(), false);
        assert_eq!(tmpfile2.is_file(), true);

        // Validate the the paths function gives me all the dirs/files in order
        let paths = sys::paths(&tmpdir).unwrap();
        assert_iter_eq(paths, vec![tmpdir1, tmpdir2, tmpfile1, tmpfile2]);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_readlink() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("readlink");
        let file1 = tmpdir.join("file1");
        let link1 = tmpdir.join("link1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(sys::symlink(&link1, &file1).is_ok());
        assert_eq!(sys::is_symlink_file(&link1), true);
        assert_eq!(sys::is_symlink_dir(&link1), false);
        assert_eq!(sys::readlink(&link1).unwrap(), file1);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    // Path tests
    // ---------------------------------------------------------------------------------------------

    #[test]
    fn test_pathext_chmod() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("pathbuf_chmod");
        let file1 = tmpdir.join("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(file1.chmod(0o644).is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100644);
        assert!(file1.chmod(0o555).is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100555);
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_pathext_clean() {
        let tests = vec![
            // Root
            ("/", "/"),
            // Remove trailing slashes
            ("/", "//"),
            ("/", "///"),
            (".", ".//"),
            // Remove duplicates and handle rooted parent ref
            ("/", "//.."),
            ("..", "..//"),
            ("/", "/..//"),
            ("foo/bar/blah", "foo//bar///blah"),
            ("/foo/bar/blah", "/foo//bar///blah"),
            // Unneeded current dirs and duplicates
            ("/", "/.//./"),
            (".", "././/./"),
            (".", "./"),
            ("/", "/./"),
            ("foo", "./foo"),
            ("foo/bar", "./foo/./bar"),
            ("/foo/bar", "/foo/./bar"),
            ("foo/bar", "foo/bar/."),
            // Handle parent references
            ("/", "/.."),
            ("/foo", "/../foo"),
            (".", "foo/.."),
            ("../foo", "../foo"),
            ("/bar", "/foo/../bar"),
            ("foo", "foo/bar/.."),
            ("bar", "foo/../bar"),
            ("/bar", "/foo/../bar"),
            (".", "foo/bar/../../"),
            ("..", "foo/bar/../../.."),
            ("/", "/foo/bar/../../.."),
            ("/", "/foo/bar/../../../.."),
            ("../..", "foo/bar/../../../.."),
            ("blah/bar", "foo/bar/../../blah/bar"),
            ("blah", "foo/bar/../../blah/bar/.."),
            ("../foo", "../foo"),
            ("../foo", "../foo/"),
            ("../foo/bar", "../foo/bar"),
            ("..", "../foo/.."),
            ("~/foo", "~/foo"),
        ];
        for test in tests {
            assert_eq!(PathBuf::from(test.0), PathBuf::from(test.1).clean().unwrap());
        }
    }

    #[test]
    fn test_pathext_dirname() {
        assert_eq!(PathBuf::from("/").as_path(), PathBuf::from("/foo/").dir().unwrap());
        assert_eq!(PathBuf::from("/foo").as_path(), PathBuf::from("/foo/bar").dir().unwrap());
    }

    #[test]
    fn test_pathext_empty() {
        // empty string
        assert_eq!(PathBuf::from("").empty(), true);

        // false
        assert_eq!(PathBuf::from("/foo").empty(), false);
    }

    #[test]
    fn test_pathext_exists() {
        let setup = Setup::init();
        assert_eq!(setup.temp.exists(), true);
    }

    #[test]
    fn test_pathext_expand() {
        let home = PathBuf::from(env::var("HOME").unwrap());

        // happy path
        assert_eq!(PathBuf::from("~/").expand().unwrap(), home);
        assert_eq!(PathBuf::from("~").expand().unwrap(), home);

        // More than one ~
        assert!(PathBuf::from("~/foo~").expand().is_err());

        // invalid path
        assert!(PathBuf::from("~foo").expand().is_err());

        // empty path - nothing to do but no error
        assert_eq!(PathBuf::from(""), PathBuf::from("").expand().unwrap());

        // can't safely do this without locking as test are run in parallel
        // // home not set
        // {
        //     env::remove_var("HOME");
        //     assert!(PathBuf::from("~/foo").expand().is_err());
        //     env::set_var("HOME", &home);
        // }
    }

    #[test]
    fn test_pathext_first() {
        assert_eq!(Component::RootDir, PathBuf::from("/").first().unwrap());
        assert_eq!(Component::CurDir, PathBuf::from(".").first().unwrap());
        assert_eq!(Component::ParentDir, PathBuf::from("..").first().unwrap());
        assert_eq!(Component::Normal(OsStr::new("foo")), PathBuf::from("foo").first().unwrap());
        assert_eq!(Component::Normal(OsStr::new("foo")), PathBuf::from("foo/bar").first().unwrap());
    }

    #[test]
    fn test_pathext_has() {
        let path = PathBuf::from("/foo/bar");
        assert!(path.has("foo"));
        assert!(path.has("/foo"));
        assert!(path.has("/"));
        assert!(path.has("/ba"));
        assert!(!path.has("bob"));
    }

    #[test]
    fn test_pathext_has_prefix() {
        let path = PathBuf::from("/foo/bar");
        assert_eq!(path.has_prefix("/foo"), true);
        assert_eq!(path.has_prefix("foo"), false);
    }

    #[test]
    fn test_pathext_has_suffix() {
        let path = PathBuf::from("/foo/bar");
        assert_eq!(path.has_suffix("/foo"), false);
        assert_eq!(path.has_suffix("/bar"), true);
    }

    #[test]
    fn test_pathext_last() {
        assert_eq!(Component::RootDir, PathBuf::from("/").last().unwrap());
        assert_eq!(Component::CurDir, PathBuf::from(".").last().unwrap());
        assert_eq!(Component::ParentDir, PathBuf::from("..").last().unwrap());
        assert_eq!(Component::Normal(OsStr::new("foo")), PathBuf::from("foo").last().unwrap());
        assert_eq!(Component::Normal(OsStr::new("bar")), PathBuf::from("/foo/bar").last().unwrap());
    }

    #[test]
    fn test_pathext_name() {
        assert_eq!("bar", PathBuf::from("/foo/bar").base().unwrap());
    }

    #[test]
    fn test_pathext_meta() {
        let setup = Setup::init();
        let meta = setup.temp.metadata().unwrap();
        assert_eq!(meta.is_dir(), true);
    }

    #[test]
    fn test_pathext_mode() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("pathbuf_mode");
        let file1 = tmpdir.join("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(file1.chmod(0o644).is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100644);
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_pathext_perms() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("pathbuf_perms");
        let file1 = tmpdir.join("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(file1.chmod(0o644).is_ok());
        assert_eq!(file1.perms().unwrap().mode(), 0o100644);
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_pathext_setperms() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("pathbuf_setperms");
        let file1 = tmpdir.join("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(file1.chmod(0o644).is_ok());
        let mut perms = file1.perms().unwrap();
        assert_eq!(perms.mode(), 0o100644);
        perms.set_mode(0o555);
        assert!(file1.setperms(perms).is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100555);
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_pathext_relative() {
        // share same directory
        assert_eq!(PathBuf::from("bar1").relative_from("bar2").unwrap(), PathBuf::from("bar1"));
        assert_eq!(PathBuf::from("foo/bar1").relative_from("foo/bar2").unwrap(), PathBuf::from("bar1"));
        assert_eq!(PathBuf::from("~/foo/bar1").relative_from("~/foo/bar2").unwrap(), PathBuf::from("bar1"));
        assert_eq!(PathBuf::from("../foo/bar1").relative_from("../foo/bar2").unwrap(), PathBuf::from("bar1"));

        // share parent directory
        assert_eq!(PathBuf::from("foo1/bar1").relative_from("foo2/bar2").unwrap(), PathBuf::from("../foo1/bar1"));

        // share grandparent directory
        assert_eq!(PathBuf::from("blah1/foo1/bar1").relative_from("blah2/foo2/bar2").unwrap(), PathBuf::from("../../blah1/foo1/bar1"));
    }

    #[test]
    fn test_pathext_to_string() {
        assert_eq!("/foo".to_string(), PathBuf::from("/foo").to_string().unwrap());
    }

    #[test]
    fn test_pathext_trim_ext() {
        assert_eq!(PathBuf::new(), PathBuf::from("").trim_ext().unwrap());
        assert_eq!(PathBuf::from("foo"), PathBuf::from("foo.exe").trim_ext().unwrap());
        assert_eq!(PathBuf::from("/foo/bar"), PathBuf::from("/foo/bar.exe").trim_ext().unwrap());
    }

    #[test]
    fn test_pathext_trim_last() {
        assert_eq!(PathBuf::new(), PathBuf::from("/").trim_last().unwrap());
        assert_eq!(PathBuf::from("/"), PathBuf::from("/foo").trim_last().unwrap());
    }

    #[test]
    fn test_pathext_trim_first() {
        assert_eq!(PathBuf::new(), PathBuf::from("/").trim_first().unwrap());
        assert_eq!(PathBuf::from("foo"), PathBuf::from("/foo").trim_first().unwrap());
    }

    #[test]
    fn test_pathext_trim_prefix() {
        // drop root
        assert_eq!(PathBuf::from("/").trim_prefix("/").unwrap(), PathBuf::new());

        // drop start
        assert_eq!(Path::new("/foo/bar").trim_prefix("/foo").unwrap(), PathBuf::from("/bar"));

        // no change
        assert_eq!(PathBuf::from("/").trim_prefix("").unwrap(), PathBuf::from("/"));
        assert_eq!(PathBuf::from("/foo").trim_prefix("blah").unwrap(), PathBuf::from("/foo"));
    }

    #[test]
    fn test_pathext_trim_protocol() {
        // no change
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo").trim_protocol().unwrap());

        // file://
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("file:///foo").trim_protocol().unwrap());

        // ftp://
        assert_eq!(PathBuf::from("foo"), PathBuf::from("ftp://foo").trim_protocol().unwrap());

        // http://
        assert_eq!(PathBuf::from("foo"), PathBuf::from("http://foo").trim_protocol().unwrap());

        // https://
        assert_eq!(PathBuf::from("foo"), PathBuf::from("https://foo").trim_protocol().unwrap());

        // Check case is being considered
        assert_eq!(PathBuf::from("Foo"), PathBuf::from("HTTPS://Foo").trim_protocol().unwrap());
        assert_eq!(PathBuf::from("Foo"), PathBuf::from("Https://Foo").trim_protocol().unwrap());
        assert_eq!(PathBuf::from("FoO"), PathBuf::from("HttpS://FoO").trim_protocol().unwrap());

        // Check non protocol matches are ignored
        assert_eq!(PathBuf::from("foo"), PathBuf::from("foo").trim_protocol().unwrap());
        assert_eq!(PathBuf::from("foo/bar"), PathBuf::from("foo/bar").trim_protocol().unwrap());
        assert_eq!(PathBuf::from("foo//bar"), PathBuf::from("foo//bar").trim_protocol().unwrap());
        assert_eq!(PathBuf::from("ntp:://foo"), PathBuf::from("ntp:://foo").trim_protocol().unwrap());
    }

    #[test]
    fn test_pathext_trim_suffix() {
        // drop root
        assert_eq!(PathBuf::new(), PathBuf::from("/").trim_suffix("/").unwrap());

        // drop end
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo/").trim_suffix("/").unwrap());

        // no change
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo").trim_suffix("/").unwrap());
    }
}
