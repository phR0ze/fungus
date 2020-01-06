use colored::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

use crate::core::*;
use crate::sys::{self, user};

/// Return the path in an absolute clean form
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let home = env::var("HOME").unwrap();
/// assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
/// ```
pub fn abs<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    let path = path.as_ref();

    // Check for empty string
    if path.empty() {
        return Err(PathError::Empty.into());
    }

    // Expand home directory
    let mut path_buf = path.expand()?;

    // Trim protocol prefix if needed
    path_buf = path_buf.trim_protocol();

    // Clean the resulting path
    path_buf = path_buf.clean()?;

    // Expand relative directories if needed
    if !path_buf.is_absolute() {
        let mut curr = env::current_dir()?;
        while let Ok(path) = path_buf.first() {
            match path {
                Component::CurDir => {
                    path_buf = path_buf.trim_first();
                }
                Component::ParentDir => {
                    curr = curr.dir()?;
                    path_buf = path_buf.trim_first();
                }
                _ => return Ok(curr.mash(path_buf)),
            }
        }
        return Ok(curr);
    }

    Ok(path_buf)
}

/// Returns all directories for the given path recurisely, sorted by filename. Handles path
/// expansion. Paths are returned as abs paths. Doesn't include the path itself. Paths are
/// guaranteed to be distinct.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_all_dirs");
/// let dir1 = tmpdir.mash("dir1");
/// let dir2 = dir1.mash("dir2");
/// assert!(sys::mkdir(&dir2).is_ok());
/// assert_iter_eq(sys::all_dirs(&tmpdir).unwrap(), vec![dir1, dir2]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn all_dirs<T: AsRef<Path>>(path: T) -> Result<Vec<PathBuf>> {
    let abs = path.as_ref().abs()?;
    if abs.exists() {
        let mut paths: Vec<PathBuf> = Vec::new();
        let mut distinct = HashMap::<PathBuf, bool>::new();
        if abs.is_dir() {
            for entry in WalkDir::new(&abs).min_depth(1).follow_links(false).sort_by(|x, y| x.file_name().cmp(y.file_name())) {
                let path = entry?.into_path();

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
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_all_files");
/// let file1 = tmpdir.mash("file1");
/// let dir1 = tmpdir.mash("dir1");
/// let file2 = dir1.mash("file2");
/// assert!(sys::mkdir(&dir1).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::touch(&file2).is_ok());
/// assert_iter_eq(sys::all_files(&tmpdir).unwrap(), vec![file2, file1]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn all_files<T: AsRef<Path>>(path: T) -> Result<Vec<PathBuf>> {
    let abs = path.as_ref().abs()?;
    if abs.exists() {
        let mut paths: Vec<PathBuf> = Vec::new();
        let mut distinct = HashMap::<PathBuf, bool>::new();
        if abs.is_dir() {
            for entry in WalkDir::new(&abs).min_depth(1).follow_links(false).sort_by(|x, y| x.file_name().cmp(y.file_name())) {
                let path = entry?.into_path();

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
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_all_paths");
/// let file1 = tmpdir.mash("file1");
/// let dir1 = tmpdir.mash("dir1");
/// let file2 = dir1.mash("file2");
/// let file3 = dir1.mash("file3");
/// assert!(sys::mkdir(&dir1).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::touch(&file2).is_ok());
/// assert!(sys::touch(&file3).is_ok());
/// assert_iter_eq(sys::all_paths(&tmpdir).unwrap(), vec![dir1, file2, file3, file1]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn all_paths<T: AsRef<Path>>(path: T) -> Result<Vec<PathBuf>> {
    let abs = path.as_ref().abs()?;
    if abs.exists() {
        let mut paths: Vec<PathBuf> = Vec::new();
        let mut distinct = HashMap::<PathBuf, bool>::new();
        if abs.is_dir() {
            for entry in WalkDir::new(&abs).min_depth(1).follow_links(false).sort_by(|x, y| x.file_name().cmp(y.file_name())) {
                let path = entry?.into_path();

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

/// Returns the current working directory. Just wraps the Rust env function but I kept forgetting
/// where it was located.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// println!("current working directory: {:?}", sys::cwd().unwrap());
/// ```
pub fn cwd() -> Result<PathBuf> {
    match env::current_dir() {
        Ok(cwd) => Ok(cwd),
        Err(err) => Err(err.into()),
    }
}

/// Returns all directories for the given path, sorted by filename. Handles path expansion.
/// Paths are returned as abs paths. Doesn't include the path itself only its children nor
/// is this recursive.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_dirs");
/// let dir1 = tmpdir.mash("dir1");
/// let dir2 = tmpdir.mash("dir2");
/// assert!(sys::mkdir(&dir1).is_ok());
/// assert!(sys::mkdir(&dir2).is_ok());
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

/// Returns true if the given path exists. Handles path expansion.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
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
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_files");
/// let file1 = tmpdir.mash("file1");
/// let file2 = tmpdir.mash("file2");
/// assert!(sys::mkdir(&tmpdir).is_ok());
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
/// use fungus::prelude::*;
///
/// assert_eq!(sys::is_dir("/etc"), true);
/// ```
pub fn is_dir<T: AsRef<Path>>(path: T) -> bool {
    match metadata(path) {
        Ok(x) => x.is_dir(),
        Err(_) => false,
    }
}

/// Returns true if the given path exists and is an executable. Handles path expansion
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_is_exec");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// assert!(sys::touch_p(&file1, 0o644).is_ok());
/// assert_eq!(sys::is_exec(&file1), false);
/// assert!(sys::chmod_p(&file1).unwrap().add_x().chmod().is_ok());
/// assert_eq!(file1.mode().unwrap(), 0o100755);
/// assert_eq!(file1.is_exec(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn is_exec<T: AsRef<Path>>(path: T) -> bool {
    match metadata(path) {
        Ok(x) => x.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

/// Returns true if the given path exists and is a file. Handles path expansion
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::is_file("/etc/hosts"), true);
/// ```
pub fn is_file<T: AsRef<Path>>(path: T) -> bool {
    match metadata(path) {
        Ok(x) => x.is_file(),
        Err(_) => false,
    }
}

/// Returns true if the given path exists and is readonly. Handles path expansion
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_is_readonly");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// assert!(sys::touch_p(&file1, 0o644).is_ok());
/// assert_eq!(file1.is_readonly(), false);
/// assert!(sys::chmod_p(&file1).unwrap().readonly().chmod().is_ok());
/// assert_eq!(file1.mode().unwrap(), 0o100444);
/// assert_eq!(sys::is_readonly(&file1), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn is_readonly<T: AsRef<Path>>(path: T) -> bool {
    match metadata(path) {
        Ok(x) => x.permissions().readonly(),
        Err(_) => false,
    }
}

/// Returns true if the given path exists and is a symlink. Handles path expansion
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_is_symlink");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let link1 = tmpdir.mash("link1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::symlink(&link1, &file1).is_ok());
/// assert_eq!(sys::is_symlink(link1), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn is_symlink<T: AsRef<Path>>(path: T) -> bool {
    match path.as_ref().abs() {
        Ok(abs) => readlink(abs).is_ok(),
        Err(_) => false,
    }
}

/// Returns true if the given path exists and is a symlinked directory. Handles path
/// expansion
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_is_symlink_dir");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let dir1 = tmpdir.mash("dir1");
/// let link1 = tmpdir.mash("link1");
/// assert!(sys::mkdir(&dir1).is_ok());
/// assert!(sys::symlink(&link1, &dir1).is_ok());
/// assert_eq!(sys::is_symlink_dir(link1), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn is_symlink_dir<T: AsRef<Path>>(path: T) -> bool {
    match path.as_ref().abs() {
        Ok(abs) => match readlink(&abs) {
            Ok(target) => match target.abs_from(&abs) {
                Ok(x) => x.is_dir(),
                Err(_) => false,
            },
            Err(_) => false,
        },
        Err(_) => false,
    }
}

/// Returns true if the given path exists and is a symlinked file. Handles path
/// expansion
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_is_symlink_file");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let link1 = tmpdir.mash("link1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::symlink(&link1, &file1).is_ok());
/// assert_eq!(sys::is_symlink_file(link1), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn is_symlink_file<T: AsRef<Path>>(path: T) -> bool {
    match path.as_ref().abs() {
        Ok(abs) => match readlink(&abs) {
            Ok(target) => match target.abs_from(&abs) {
                Ok(x) => x.is_file(),
                Err(_) => false,
            },
            Err(_) => false,
        },
        Err(_) => false,
    }
}

/// Returns the group ID of the owner of this file. Handles path expansion.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::gid("/etc").unwrap(), 0);
/// ```
pub fn gid<T: AsRef<Path>>(path: T) -> Result<u32> {
    Ok(metadata(path)?.gid())
}

/// Returns a vector of all paths from the given target glob with path expansion and sorted by
/// name. Doesn't include the target itself only its children nor is this recursive.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_glob");
/// let dir1 = tmpdir.mash("dir1");
/// let dir2 = tmpdir.mash("dir2");
/// let file1 = tmpdir.mash("file1");
/// assert!(sys::mkdir(&dir1).is_ok());
/// assert!(sys::mkdir(&dir2).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert_iter_eq(sys::glob(tmpdir.mash("*")).unwrap(), vec![dir1, dir2, file1]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn glob<T: AsRef<Path>>(src: T) -> Result<Vec<PathBuf>> {
    let abs = src.as_ref().abs()?.to_string()?;
    let mut paths: Vec<PathBuf> = Vec::new();
    for x in glob::glob(&abs)? {
        paths.push(x?.abs()?);
    }
    Ok(paths)
}

/// Returns a new owned [`PathBuf`] from `dir` mashed together with `base`.
/// Differs from the `mash` implementation as `mash` drops root prefix of the given `path` if
/// it exists and also drops any trailing '/' on the new resulting path. More closely aligns
/// with the Golang implementation of join.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::mash("/foo", "/bar"), PathBuf::from("/foo/bar"));
/// ```
pub fn mash<T: AsRef<Path>, U: AsRef<Path>>(dir: T, base: U) -> PathBuf {
    dir.as_ref().join(base.as_ref().trim_prefix("/")).components().collect::<PathBuf>()
}

/// Returns the Metadata object for the `Path` if it exists else an error. Handles path
/// expansion.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let meta = sys::metadata(Path::new("/etc")).unwrap();
/// assert_eq!(meta.is_dir(), true);
/// ```
pub fn metadata<T: AsRef<Path>>(path: T) -> Result<fs::Metadata> {
    let abs = path.as_ref().abs()?;
    let meta = fs::metadata(abs)?;
    Ok(meta)
}

/// Parse unix shell pathing e.g. $PATH, $XDG_DATA_DIRS or $XDG_CONFIG_DIRS.
/// List of directories seperated by :
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let paths = vec![PathBuf::from("/foo1"), PathBuf::from("/foo2/bar")];
/// assert_iter_eq(sys::parse_paths("/foo1:/foo2/bar").unwrap(), paths);
/// ```
pub fn parse_paths<T: AsRef<str>>(value: T) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for dir in value.as_ref().split(':') {
        // Unix shell semantics: path element "" means "."
        let path = match dir == "" {
            true => sys::cwd()?,
            false => PathBuf::from(dir),
        };
        paths.push(path);
    }
    Ok(paths)
}

/// Returns all directories/files for the given path, sorted by filename. Handles path
/// expansion. Paths are returned as abs paths. Doesn't include the path itself only
/// its children nor is this recursive.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_paths");
/// let dir1 = tmpdir.mash("dir1");
/// let dir2 = tmpdir.mash("dir2");
/// let file1 = tmpdir.mash("file1");
/// assert!(sys::mkdir(&dir1).is_ok());
/// assert!(sys::mkdir(&dir2).is_ok());
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
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("path_doc_readlink");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let link1 = tmpdir.mash("link1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
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

/// Returns the user ID of the owner of this file. Handles path expansion.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::uid("/etc").unwrap(), 0);
/// ```
pub fn uid<T: AsRef<Path>>(path: T) -> Result<u32> {
    Ok(metadata(path)?.uid())
}

// Path extensions
// -------------------------------------------------------------------------------------------------
pub trait PathExt {
    /// Return the path in an absolute clean form
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let home = env::var("HOME").unwrap();
    /// assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
    /// ```
    fn abs(&self) -> Result<PathBuf>;

    /// Returns a new absolute [`PathBuf`] based on the given absolute `Path`. The last element of
    /// the given path will be assumed to be a file name.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let home = PathBuf::from("~").abs().unwrap();
    /// assert_eq!(PathBuf::from("foo2").abs_from(home.mash("foo1").abs().unwrap()).unwrap(), home.mash("foo2"));
    /// ```
    fn abs_from<T: AsRef<Path>>(&self, path: T) -> Result<PathBuf>;

    /// Returns the final component of the `Path`, if there is one.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!("bar", PathBuf::from("/foo/bar").base().unwrap());
    /// ```
    fn base(&self) -> Result<String>;

    /// Set the given mode for the `Path` and return the `Path`
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("pathbuf_doc_chmod");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.mash("file1");
    /// assert!(sys::mkdir(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(file1.chmod(0o644).is_ok());
    /// assert_eq!(file1.mode().unwrap(), 0o100644);
    /// assert!(file1.chmod(0o555).is_ok());
    /// assert_eq!(file1.mode().unwrap(), 0o100555);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn chmod(&self, mode: u32) -> Result<()>;

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

    /// Returns the `Path` with the given string concatenated on.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(Path::new("/foo/bar").concat(".rs").unwrap(), PathBuf::from("/foo/bar.rs"));
    /// ```
    fn concat<T: AsRef<str>>(&self, val: T) -> Result<PathBuf>;

    /// Returns the `Path` without its final component, if there is one.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let dir = PathBuf::from("/foo/bar").dir().unwrap();
    /// assert_eq!(PathBuf::from("/foo").as_path(), dir);
    /// ```
    fn dir(&self) -> Result<PathBuf>;

    /// Returns true if the `Path` is empty.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(PathBuf::from("").empty(), true);
    /// ```
    fn empty(&self) -> bool;

    /// Returns true if the `Path` exists. Handles path expansion.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(Path::new("/etc").exists(), true);
    /// ```
    fn exists(&self) -> bool;

    /// Expand the path to include the home prefix if necessary
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let home = env::var("HOME").unwrap();
    /// assert_eq!(PathBuf::from(&home).mash("foo"), PathBuf::from("~/foo").expand().unwrap());
    /// ```
    fn expand(&self) -> Result<PathBuf>;

    /// Returns the extension of the path or an error.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(Path::new("foo.bar").ext().unwrap(), "bar");
    /// ```
    fn ext(&self) -> Result<String>;

    /// Returns the first path component.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let first = Component::Normal(OsStr::new("foo"));
    /// assert_eq!(PathBuf::from("foo/bar").first().unwrap(), first);
    /// ```
    fn first(&self) -> Result<Component>;

    /// Returns the group ID of the owner of this file.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(Path::new("/etc").gid().unwrap(), 0);
    /// ```
    fn gid(&self) -> Result<u32>;

    /// Returns true if the `Path` contains the given path or string.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let path = PathBuf::from("/foo/bar");
    /// assert_eq!(path.has("foo"), true);
    /// assert_eq!(path.has("/foo"), true);
    /// ```
    fn has<T: AsRef<Path>>(&self, path: T) -> bool;

    /// Returns true if the `Path` as a String has the given prefix
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let path = PathBuf::from("/foo/bar");
    /// assert_eq!(path.has_prefix("/foo"), true);
    /// assert_eq!(path.has_prefix("foo"), false);
    /// ```
    fn has_prefix<T: AsRef<Path>>(&self, prefix: T) -> bool;

    /// Returns true if the `Path` as a String has the given suffix
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let path = PathBuf::from("/foo/bar");
    /// assert_eq!(path.has_suffix("/bar"), true);
    /// assert_eq!(path.has_suffix("foo"), false);
    /// ```
    fn has_suffix<T: AsRef<Path>>(&self, suffix: T) -> bool;

    /// Returns true if the `Path` exists and is a directory. Handles path expansion.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(Path::new("/etc").is_dir(), true);
    /// ```
    fn is_dir(&self) -> bool;

    /// Returns true if the `Path` exists and is an executable. Handles path expansion.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_is_exec");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// assert!(sys::mkdir(&tmpdir).is_ok());
    /// let file1 = tmpdir.mash("file1");
    /// assert!(sys::touch_p(&file1, 0o644).is_ok());
    /// assert_eq!(file1.is_exec(), false);
    /// assert!(sys::chmod_p(&file1).unwrap().add_x().chmod().is_ok());
    /// assert_eq!(file1.mode().unwrap(), 0o100755);
    /// assert_eq!(file1.is_exec(), true);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn is_exec(&self) -> bool;

    /// Returns true if the `Path` exists and is a file. Handles path expansion
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(Path::new("/etc/hosts").is_file(), true);
    /// ```
    fn is_file(&self) -> bool;

    /// Returns true if the `Path` exists and is readonly. Handles path expansion.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_is_readonly");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// assert!(sys::mkdir(&tmpdir).is_ok());
    /// let file1 = tmpdir.mash("file1");
    /// assert!(sys::touch_p(&file1, 0o644).is_ok());
    /// assert_eq!(file1.is_readonly(), false);
    /// assert!(sys::chmod_p(&file1).unwrap().readonly().chmod().is_ok());
    /// assert_eq!(file1.mode().unwrap(), 0o100444);
    /// assert_eq!(file1.is_readonly(), true);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn is_readonly(&self) -> bool;

    /// Returns true if the `Path` exists and is a symlink. Handles path expansion
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("pathbuf_doc_is_symlink");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.mash("file1");
    /// let link1 = tmpdir.mash("link1");
    /// assert!(sys::mkdir(&tmpdir).is_ok());
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
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("pathbuf_doc_is_symlink_dir");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let dir1 = tmpdir.mash("dir1");
    /// let link1 = tmpdir.mash("link1");
    /// assert!(sys::mkdir(&dir1).is_ok());
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
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("pathbuf_doc_is_symlink_file");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.mash("file1");
    /// let link1 = tmpdir.mash("link1");
    /// assert!(sys::mkdir(&tmpdir).is_ok());
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
    /// use fungus::prelude::*;
    ///
    /// let first = Component::Normal(OsStr::new("bar"));
    /// assert_eq!(PathBuf::from("foo/bar").last().unwrap(), first);
    /// ```
    fn last(&self) -> Result<Component>;

    /// Returns a new owned [`PathBuf`] from `self` mashed together with `path`.
    /// Differs from the `mash` implementation as `mash` drops root prefix of the given `path` if
    /// it exists and also drops any trailing '/' on the new resulting path. More closely aligns
    /// with the Golang implementation of join.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(Path::new("/foo").mash("/bar"), PathBuf::from("/foo/bar"));
    /// ```
    fn mash<T: AsRef<Path>>(&self, path: T) -> PathBuf;

    /// Returns the Metadata object for the `Path` if it exists else and error
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let meta = Path::new("/etc").metadata().unwrap();
    /// assert_eq!(meta.is_dir(), true);
    /// ```
    fn metadata(&self) -> Result<fs::Metadata>;

    /// Returns the Metadata object for the `Path` if it exists else and error
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("pathbuf_doc_mode");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.mash("file1");
    /// assert!(sys::mkdir(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(file1.chmod(0o644).is_ok());
    /// assert_eq!(file1.mode().unwrap(), 0o100644);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn mode(&self) -> Result<u32>;

    /// Returns the final component of the `Path` without an extension if there is one
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(PathBuf::from("/foo/bar.foo").name().unwrap(), "bar");
    /// ```
    fn name(&self) -> Result<String>;

    /// Return the permissions for the `Path`
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("pathbuf_doc_perms");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.mash("file1");
    /// assert!(sys::mkdir(&tmpdir).is_ok());
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
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("pathbuf_doc_readlink");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.mash("file1");
    /// let link1 = tmpdir.mash("link1");
    /// assert!(sys::mkdir(&tmpdir).is_ok());
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
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(PathBuf::from("foo/bar1").relative_from("foo/bar2").unwrap(), PathBuf::from("bar1"));
    /// ```
    fn relative_from<T: AsRef<Path>>(&self, path: T) -> Result<PathBuf>;

    /// Set the given [`Permissions`] on the `Path` and return the `Path`
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("pathbuf_doc_setperms");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let file1 = tmpdir.mash("file1");
    /// assert!(sys::mkdir(&tmpdir).is_ok());
    /// assert!(sys::touch(&file1).is_ok());
    /// assert!(file1.chmod(0o644).is_ok());
    /// assert_eq!(file1.perms().unwrap().mode(), 0o100644);
    /// assert!(file1.setperms(fs::Permissions::from_mode(0o555)).is_ok());
    /// assert_eq!(file1.perms().unwrap().mode(), 0o100555);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    fn setperms(&self, perms: fs::Permissions) -> Result<PathBuf>;

    /// Returns a new [`String`] from the `path`.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(PathBuf::from("/foo").to_string().unwrap(), "/foo".to_string());
    /// ```
    fn to_string(&self) -> Result<String>;

    /// Returns a new [`PathBuf`] with the file extension trimmed off.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(Path::new("foo.exe").trim_ext().unwrap(), PathBuf::from("foo"));
    /// ```
    fn trim_ext(&self) -> Result<PathBuf>;

    /// Returns a new [`PathBuf`] with first [`Component`] trimmed off.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(PathBuf::from("/foo").trim_first(), PathBuf::from("foo"));
    /// ```
    fn trim_first(&self) -> PathBuf;

    /// Returns a new [`PathBuf`] with last [`Component`] trimmed off.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(PathBuf::from("/foo").trim_last(), PathBuf::from("/"));
    /// ```
    fn trim_last(&self) -> PathBuf;

    /// Returns a new [`PathBuf`] with the given prefix trimmed off else the original `path`.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(Path::new("/foo/bar").trim_prefix("/foo"), PathBuf::from("/bar"));
    /// ```
    fn trim_prefix<T: AsRef<Path>>(&self, prefix: T) -> PathBuf;

    /// Returns a new [`PathBuf`] with well known protocol prefixes trimmed off else the original
    /// `path`.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(PathBuf::from("ftp://foo").trim_protocol(), PathBuf::from("foo"));
    /// ```
    fn trim_protocol(&self) -> PathBuf;

    /// Returns a new [`PathBuf`] with the given `suffix` trimmed off else the original `path`.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(PathBuf::from("/foo/bar").trim_suffix("/bar"), PathBuf::from("/foo"));
    /// ```
    fn trim_suffix<T: AsRef<Path>>(&self, suffix: T) -> PathBuf;

    /// Returns the user ID of the owner of this file.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(Path::new("/etc").uid().unwrap(), 0);
    /// ```
    fn uid(&self) -> Result<u32>;
}

impl PathExt for Path {
    fn abs(&self) -> Result<PathBuf> {
        abs(self)
    }

    fn abs_from<T: AsRef<Path>>(&self, base: T) -> Result<PathBuf> {
        let base = base.as_ref().abs()?;
        if !self.is_absolute() && self != base {
            let mut path = base.trim_last();
            let mut components = self.components();
            loop {
                match components.next() {
                    Some(component) => match component {
                        Component::ParentDir => path = path.trim_last(),
                        Component::Normal(x) => return Ok(path.mash(x).mash(components.collect::<PathBuf>()).clean()?),
                        _ => (),
                    },
                    None => return Err(PathError::Empty.into()),
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

    fn chmod(&self, mode: u32) -> Result<()> {
        sys::chmod(self, mode)?;
        Ok(())
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

    fn concat<T: AsRef<str>>(&self, val: T) -> Result<PathBuf> {
        Ok(PathBuf::from(format!("{}{}", self.to_string()?, val.as_ref())))
    }

    fn dir(&self) -> Result<PathBuf> {
        let dir = self.parent().ok_or_else(|| PathError::parent_not_found(self))?;
        Ok(dir.to_path_buf())
    }

    fn empty(&self) -> bool {
        self == PathBuf::new()
    }

    fn exists(&self) -> bool {
        exists(&self)
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
                expanded = user::home_dir()?;
            }

            // Replace prefix with home directory
            1 => expanded = user::home_dir()?.mash(&path_str[2..]),
            _ => (),
        }

        Ok(expanded)
    }

    fn ext(&self) -> Result<String> {
        match self.extension() {
            Some(val) => Ok(val.to_str().ok_or_else(|| PathError::failed_to_string(self))?.to_string()),
            None => Err(PathError::extension_not_found(self).into()),
        }
    }

    fn first(&self) -> Result<Component> {
        self.components().first_result()
    }

    fn gid(&self) -> Result<u32> {
        gid(&self)
    }

    fn has<T: AsRef<Path>>(&self, path: T) -> bool {
        match (self.to_string(), path.as_ref().to_string()) {
            (Ok(base), Ok(path)) => base.contains(&path),
            _ => false,
        }
    }

    fn has_prefix<T: AsRef<Path>>(&self, prefix: T) -> bool {
        match (self.to_string(), prefix.as_ref().to_string()) {
            (Ok(base), Ok(prefix)) => base.starts_with(&prefix),
            _ => false,
        }
    }

    fn has_suffix<T: AsRef<Path>>(&self, suffix: T) -> bool {
        match (self.to_string(), suffix.as_ref().to_string()) {
            (Ok(base), Ok(suffix)) => base.ends_with(&suffix),
            _ => false,
        }
    }

    fn is_dir(&self) -> bool {
        is_dir(self)
    }

    fn is_exec(&self) -> bool {
        is_exec(self)
    }

    fn is_file(&self) -> bool {
        is_file(self)
    }

    fn is_readonly(&self) -> bool {
        is_readonly(self)
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

    fn mash<T: AsRef<Path>>(&self, path: T) -> PathBuf {
        mash(self, path)
    }

    fn metadata(&self) -> Result<fs::Metadata> {
        let meta = fs::metadata(self)?;
        Ok(meta)
    }

    fn mode(&self) -> Result<u32> {
        let perms = self.perms()?;
        Ok(perms.mode())
    }

    fn name(&self) -> Result<String> {
        self.trim_ext()?.base()
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
        Ok(match self.extension() {
            Some(val) => {
                let _str = val.to_str().ok_or_else(|| PathError::failed_to_string(self))?;
                self.trim_suffix(format!(".{}", _str.to_string()))
            }
            None => self.to_path_buf(),
        })
    }

    fn trim_first(&self) -> PathBuf {
        self.components().drop(1).as_path().to_path_buf()
    }

    fn trim_last(&self) -> PathBuf {
        self.components().drop(-1).as_path().to_path_buf()
    }

    fn trim_prefix<T: AsRef<Path>>(&self, prefix: T) -> PathBuf {
        match (self.to_string(), prefix.as_ref().to_string()) {
            (Ok(base), Ok(prefix)) if base.starts_with(&prefix) => PathBuf::from(&base[prefix.len()..]),
            _ => self.to_path_buf(),
        }
    }

    fn trim_protocol(&self) -> PathBuf {
        match self.to_string() {
            Ok(base) => match base.find("//") {
                Some(i) => {
                    let (prefix, suffix) = base.split_at(i + 2);
                    let lower = prefix.to_lowercase();
                    let lower = lower.trim_start_matches("file://");
                    let lower = lower.trim_start_matches("ftp://");
                    let lower = lower.trim_start_matches("http://");
                    let lower = lower.trim_start_matches("https://");
                    if lower != "" {
                        PathBuf::from(format!("{}{}", prefix, suffix))
                    } else {
                        PathBuf::from(suffix)
                    }
                }
                _ => PathBuf::from(base),
            },
            _ => self.to_path_buf(),
        }
    }

    fn trim_suffix<T: AsRef<Path>>(&self, suffix: T) -> PathBuf {
        match (self.to_string(), suffix.as_ref().to_string()) {
            (Ok(base), Ok(suffix)) if base.ends_with(&suffix) => PathBuf::from(&base[..base.len() - suffix.len()]),
            _ => self.to_path_buf(),
        }
    }

    fn uid(&self) -> Result<u32> {
        uid(&self)
    }
}

pub trait PathColorExt {
    fn black(&self) -> Result<ColoredString>;
    fn red(&self) -> Result<ColoredString>;
    fn green(&self) -> Result<ColoredString>;
    fn yellow(&self) -> Result<ColoredString>;
    fn blue(&self) -> Result<ColoredString>;
    fn magenta(&self) -> Result<ColoredString>;
    fn purple(&self) -> Result<ColoredString>;
    fn cyan(&self) -> Result<ColoredString>;
    fn white(&self) -> Result<ColoredString>;
    fn bright_black(&self) -> Result<ColoredString>;
    fn bright_red(&self) -> Result<ColoredString>;
    fn bright_green(&self) -> Result<ColoredString>;
    fn bright_yellow(&self) -> Result<ColoredString>;
    fn bright_blue(&self) -> Result<ColoredString>;
    fn bright_magenta(&self) -> Result<ColoredString>;
    fn bright_purple(&self) -> Result<ColoredString>;
    fn bright_cyan(&self) -> Result<ColoredString>;
    fn bright_white(&self) -> Result<ColoredString>;
    fn color<S: Into<Color>>(&self, color: S) -> Result<ColoredString>;
    fn on_black(&self) -> Result<ColoredString>;
    fn on_red(&self) -> Result<ColoredString>;
    fn on_green(&self) -> Result<ColoredString>;
    fn on_yellow(&self) -> Result<ColoredString>;
    fn on_blue(&self) -> Result<ColoredString>;
    fn on_magenta(&self) -> Result<ColoredString>;
    fn on_purple(&self) -> Result<ColoredString>;
    fn on_cyan(&self) -> Result<ColoredString>;
    fn on_white(&self) -> Result<ColoredString>;
    fn on_bright_black(&self) -> Result<ColoredString>;
    fn on_bright_red(&self) -> Result<ColoredString>;
    fn on_bright_green(&self) -> Result<ColoredString>;
    fn on_bright_yellow(&self) -> Result<ColoredString>;
    fn on_bright_blue(&self) -> Result<ColoredString>;
    fn on_bright_magenta(&self) -> Result<ColoredString>;
    fn on_bright_purple(&self) -> Result<ColoredString>;
    fn on_bright_cyan(&self) -> Result<ColoredString>;
    fn on_bright_white(&self) -> Result<ColoredString>;
    fn on_color<S: Into<Color>>(&self, color: S) -> Result<ColoredString>;
    fn normal(&self) -> Result<ColoredString>;
    fn bold(&self) -> Result<ColoredString>;
    fn dimmed(&self) -> Result<ColoredString>;
    fn italic(&self) -> Result<ColoredString>;
    fn underline(&self) -> Result<ColoredString>;
    fn blink(&self) -> Result<ColoredString>;
    fn reverse(&self) -> Result<ColoredString>;
    fn reversed(&self) -> Result<ColoredString>;
    fn hidden(&self) -> Result<ColoredString>;
    fn strikethrough(&self) -> Result<ColoredString>;
}
impl PathColorExt for Path {
    fn black(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::Black))
    }
    fn red(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::Red))
    }
    fn green(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::Green))
    }
    fn yellow(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::Yellow))
    }
    fn blue(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::Blue))
    }
    fn magenta(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::Magenta))
    }
    fn purple(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::Magenta))
    }
    fn cyan(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::Cyan))
    }
    fn white(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::White))
    }
    fn bright_black(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::BrightBlack))
    }
    fn bright_red(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::BrightRed))
    }
    fn bright_green(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::BrightGreen))
    }
    fn bright_yellow(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::BrightYellow))
    }
    fn bright_blue(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::BrightBlue))
    }
    fn bright_magenta(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::BrightMagenta))
    }
    fn bright_purple(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::BrightMagenta))
    }
    fn bright_cyan(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::BrightCyan))
    }
    fn bright_white(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.color(Color::BrightWhite))
    }
    fn color<S: Into<Color>>(&self, color: S) -> Result<ColoredString> {
        Ok(self.to_string()?.color(color.into()))
    }
    fn on_black(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::Black))
    }
    fn on_red(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::Red))
    }
    fn on_green(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::Green))
    }
    fn on_yellow(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::Yellow))
    }
    fn on_blue(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::Blue))
    }
    fn on_magenta(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::Magenta))
    }
    fn on_purple(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::Magenta))
    }
    fn on_cyan(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::Cyan))
    }
    fn on_white(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::White))
    }
    fn on_bright_black(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::BrightBlack))
    }
    fn on_bright_red(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::BrightRed))
    }
    fn on_bright_green(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::BrightGreen))
    }
    fn on_bright_yellow(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::BrightYellow))
    }
    fn on_bright_blue(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::BrightBlue))
    }
    fn on_bright_magenta(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::BrightMagenta))
    }
    fn on_bright_purple(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::BrightMagenta))
    }
    fn on_bright_cyan(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::BrightCyan))
    }
    fn on_bright_white(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(Color::BrightWhite))
    }
    fn on_color<S: Into<Color>>(&self, color: S) -> Result<ColoredString> {
        Ok(self.to_string()?.on_color(color))
    }
    fn normal(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.normal())
    }
    fn bold(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.bold())
    }
    fn dimmed(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.dimmed())
    }
    fn italic(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.italic())
    }
    fn underline(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.underline())
    }
    fn blink(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.blink())
    }
    fn reverse(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.reverse())
    }
    fn reversed(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.reversed())
    }
    fn hidden(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.hidden())
    }
    fn strikethrough(&self) -> Result<ColoredString> {
        Ok(self.to_string()?.strikethrough())
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // Reusable teset setup
    struct Setup {
        temp: PathBuf,
    }
    impl Setup {
        fn init() -> Self {
            let setup = Self { temp: PathBuf::from("tests/temp").abs().unwrap() };
            sys::mkdir(&setup.temp).unwrap();
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
        assert_eq!(sys::abs("foo").unwrap(), cwd.mash("foo"));

        // expand home path
        assert_eq!(sys::abs("~/foo").unwrap(), home.mash("foo"));

        // More complicated
        assert_eq!(sys::abs("~/foo/bar/../.").unwrap(), home.mash("foo"));
        assert_eq!(sys::abs("~/foo/bar/../").unwrap(), home.mash("foo"));
        assert_eq!(sys::abs("~/foo/bar/../blah").unwrap(), home.mash("foo/blah"));

        // Move up the path multiple levels
        assert_eq!(sys::abs("./../../foo").unwrap(), home.mash("foo"));
        assert_eq!(sys::abs("../../foo").unwrap(), home.mash("foo"));

        // Move up until invalid
        assert!(sys::abs("../../../../../foo").is_err());
    }

    #[test]
    fn test_all_dirs() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_all_dirs");
        let tmpdir1 = tmpdir.mash("dir1");
        let tmpdir2 = tmpdir1.mash("dir2");
        let tmpfile1 = tmpdir.mash("file1");
        let tmpfile2 = tmpdir.mash("file2");

        // invalid target
        assert!(sys::all_dirs("").is_err());

        // Create the dirs and files
        assert!(sys::mkdir(&tmpdir1).is_ok());
        assert!(sys::mkdir(&tmpdir2).is_ok());
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

        // invalid target
        assert!(sys::all_dirs(&tmpfile1).is_err());

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
        let tmpdir = setup.temp.mash("path_all_files");
        let tmpdir1 = tmpdir.mash("dir1");
        let tmpdir2 = tmpdir1.mash("dir2");
        let tmpfile1 = tmpdir1.mash("file1");
        let tmpfile2 = tmpdir2.mash("file2");

        // invalid target
        assert!(sys::all_files("").is_err());

        // Create the dirs and files
        assert!(sys::mkdir(&tmpdir1).is_ok());
        assert!(sys::mkdir(&tmpdir2).is_ok());
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

        // invalid target
        assert!(sys::all_files(&tmpfile1).is_err());

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
        let tmpdir = setup.temp.mash("path_all_paths");
        let tmpdir1 = tmpdir.mash("dir1");
        let tmpdir2 = tmpdir1.mash("dir2");
        let tmpfile1 = tmpdir1.mash("file1");
        let tmpfile2 = tmpdir2.mash("file2");

        // invalid target
        assert!(sys::all_paths("").is_err());

        // Create the dirs and files
        assert!(sys::mkdir(&tmpdir1).is_ok());
        assert!(sys::mkdir(&tmpdir2).is_ok());
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

        // invalid target
        assert!(sys::all_paths(&tmpfile1).is_err());

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
        let tmpdir = setup.temp.mash("path_dirs");
        let tmpdir1 = tmpdir.mash("dir1");
        let tmpdir2 = tmpdir.mash("dir2");
        let tmpfile1 = tmpdir.mash("file1");
        let tmpfile2 = tmpdir.mash("file2");

        // invalid target
        assert!(sys::dirs("").is_err());

        // Create the dirs and files
        assert!(sys::mkdir(&tmpdir1).is_ok());
        assert!(sys::mkdir(&tmpdir2).is_ok());
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

        // invalid target
        assert!(sys::dirs(&tmpfile1).is_err());

        // Validate the the dirs function gives me the correct dirs without the files and in order
        let dirs = sys::dirs(&tmpdir).unwrap();
        assert_iter_eq(dirs, vec![tmpdir1, tmpdir2]);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_files() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_files");
        let tmpdir1 = tmpdir.mash("dir1");
        let tmpdir2 = tmpdir.mash("dir2");
        let tmpfile1 = tmpdir.mash("file1");
        let tmpfile2 = tmpdir.mash("file2");

        // invalid target
        assert!(sys::files("").is_err());

        // Create the dirs and files
        assert!(sys::mkdir(&tmpdir1).is_ok());
        assert!(sys::mkdir(&tmpdir2).is_ok());
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

        // invalid target
        assert!(sys::files(&tmpfile1).is_err());

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
    fn test_is_exec() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_is_exec");
        let file1 = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert!(sys::touch_p(&file1, 0o644).is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100644);
        assert_eq!(file1.is_exec(), false);

        // add_x
        assert!(sys::chmod_p(&file1).unwrap().add_x().chmod().is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100755);
        assert_eq!(file1.is_exec(), true);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_is_file() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_is_file");
        let tmpfile = tmpdir.mash("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert!(sys::touch(&tmpfile).is_ok());
        assert_eq!(sys::is_file(tmpfile), true);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_is_symlink() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_is_symlink");
        let file1 = tmpdir.mash("file1");
        let link1 = tmpdir.mash("link1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(sys::symlink(&link1, &file1).is_ok());
        assert_eq!(sys::is_symlink(link1), true);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_is_symlink_dir() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_is_symlink_dir");
        let dir1 = tmpdir.mash("dir1");
        let link1 = tmpdir.mash("link1");
        let link2 = tmpdir.mash("link2");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&dir1).is_ok());

        // test absolute
        assert!(sys::symlink(&link1, &dir1).is_ok());
        assert_eq!(sys::is_symlink_dir(&link1), true);
        assert_eq!(sys::is_symlink_file(&link1), false);

        // test relative
        assert!(sys::symlink(&link2, "dir1").is_ok());
        assert_eq!(sys::is_symlink_dir(&link2), true);
        assert_eq!(sys::is_symlink_file(&link2), false);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_is_symlink_file() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_is_symlink_file");
        let file1 = tmpdir.mash("file1");
        let link1 = tmpdir.mash("link1");
        let link2 = tmpdir.mash("link2");

        // invalid
        assert_eq!(sys::is_symlink_file(""), false);

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());

        // test absolute
        assert!(sys::symlink(&link1, &file1).is_ok());
        assert_eq!(sys::is_symlink_file(&link1), true);
        assert_eq!(sys::is_symlink_dir(&link1), false);

        // test relative
        assert!(sys::symlink(&link2, "file1").is_ok());
        assert_eq!(sys::is_symlink_file(&link2), true);
        assert_eq!(sys::is_symlink_dir(&link2), false);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_glob() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_glob");
        let tmpdir1 = tmpdir.mash("dir1");
        let tmpdir2 = tmpdir.mash("dir2");
        let tmpfile1 = tmpdir.mash("file1");
        let tmpfile2 = tmpdir.mash("file2");

        // Create the dirs and files
        assert!(sys::mkdir(&tmpdir1).is_ok());
        assert!(sys::mkdir(&tmpdir2).is_ok());
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
        let paths = sys::glob(tmpdir.mash("*")).unwrap();
        assert_iter_eq(paths, vec![tmpdir1, tmpdir2, tmpfile1, tmpfile2]);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_metadata() {
        let setup = Setup::init();
        let meta = sys::metadata(setup.temp).unwrap();
        assert_eq!(meta.is_dir(), true);
    }

    #[test]
    fn test_paths() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_paths");
        let tmpdir1 = tmpdir.mash("dir1");
        let tmpdir2 = tmpdir.mash("dir2");
        let tmpfile1 = tmpdir.mash("file1");
        let tmpfile2 = tmpdir.mash("file2");

        // invalid target
        assert!(sys::paths("").is_err());

        // Create the dirs and files
        assert!(sys::mkdir(&tmpdir1).is_ok());
        assert!(sys::mkdir(&tmpdir2).is_ok());
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

        // invalid target
        assert!(sys::paths(&tmpfile1).is_err());

        // Validate the the paths function gives me all the dirs/files in order
        let paths = sys::paths(&tmpdir).unwrap();
        assert_iter_eq(paths, vec![tmpdir1, tmpdir2, tmpfile1, tmpfile2]);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_parse_paths() {
        let paths = vec![PathBuf::from("/foo1"), PathBuf::from("/foo2/bar")];
        assert_iter_eq(sys::parse_paths("/foo1:/foo2/bar").unwrap(), paths);

        let paths = vec![sys::cwd().unwrap(), PathBuf::from("/foo1"), PathBuf::from("/foo2/bar")];
        assert_iter_eq(sys::parse_paths(":/foo1:/foo2/bar").unwrap(), paths);
    }

    #[test]
    fn test_readlink() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_readlink");
        let file1 = tmpdir.mash("file1");
        let link1 = tmpdir.mash("link1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
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
    fn test_pathext_abs_from() {
        let home = PathBuf::from("~").abs().unwrap();

        // invalid
        assert!(PathBuf::from("foo").abs_from("").is_err());

        // already absolute
        assert_eq!(PathBuf::from("/foo").abs_from("foo1").unwrap(), PathBuf::from("/foo"));

        // share the same directory
        assert_eq!(PathBuf::from("foo2").abs_from(home.mash("foo1").abs().unwrap()).unwrap(), home.mash("foo2"));
        assert_eq!(PathBuf::from("./foo2").abs_from(home.mash("foo1").abs().unwrap()).unwrap(), home.mash("foo2"));

        // share parent directory
        assert_eq!(PathBuf::from("../foo2").abs_from(home.mash("bar1/foo1").abs().unwrap()).unwrap(), home.mash("foo2"));
        assert_eq!(PathBuf::from("bar2/foo2").abs_from(home.mash("bar1/foo1").abs().unwrap()).unwrap(), home.mash("bar1/bar2/foo2"));
        assert_eq!(PathBuf::from("../../foo2").abs_from(home.mash("bar1/foo1").abs().unwrap()).unwrap(), home.trim_last().mash("foo2"));

        // share grandparent directory
        assert_eq!(PathBuf::from("blah1/bar2/foo2").abs_from(home.mash("bar1/foo1").abs().unwrap()).unwrap(), home.mash("bar1/blah1/bar2/foo2"));
    }

    #[test]
    fn test_pathext_base() {
        assert_eq!("bar", PathBuf::from("/foo/bar").base().unwrap());
    }

    #[test]
    fn test_pathext_chmod() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_pathbuf_chmod");
        let file1 = tmpdir.mash("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
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
    fn test_pathext_concat() {
        assert_eq!(Path::new("").concat(".rs").unwrap(), PathBuf::from(".rs"));
        assert_eq!(Path::new("foo").concat(".rs").unwrap(), PathBuf::from("foo.rs"));
        assert_eq!(Path::new("foo.exe").concat(".rs").unwrap(), PathBuf::from("foo.exe.rs"));
        assert_eq!(Path::new("/foo/bar").concat(".rs").unwrap(), PathBuf::from("/foo/bar.rs"));
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
    fn test_pathext_ext() {
        assert!(PathBuf::from("").ext().is_err());
        assert!(PathBuf::from("foo").ext().is_err());
        assert_eq!(PathBuf::from("foo.exe").ext().unwrap(), "exe");
        assert_eq!(PathBuf::from("/foo/bar.exe").ext().unwrap(), "exe");
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
    fn test_pathext_is_dir() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_pathext_is_dir");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.is_dir(), false);
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert_eq!(tmpdir.is_dir(), true);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_pathext_is_file() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_pathext_is_file");
        let tmpfile = tmpdir.mash("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert!(sys::touch(&tmpfile).is_ok());
        assert_eq!(tmpfile.is_file(), true);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_pathext_is_symlink_file() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_pathext_is_symlink_file");
        let file1 = tmpdir.mash("file1");
        let link1 = tmpdir.mash("link1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert_eq!(link1.is_symlink_file(), false);
        assert!(sys::touch(&file1).is_ok());
        assert!(sys::symlink(&link1, &file1).is_ok());
        assert_eq!(link1.is_symlink_file(), true);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
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
    fn test_pathext_metadata() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_pathext_metadata");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(tmpdir.metadata().is_err());
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert!(tmpdir.metadata().is_ok());

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_pathext_mash() {
        // strips off root on path
        assert_eq!(Path::new("/foo").mash("/bar"), PathBuf::from("/foo/bar"));

        // strips off trailing slashes
        assert_eq!(Path::new("/foo").mash("bar/"), PathBuf::from("/foo/bar"));
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
        let tmpdir = setup.temp.mash("path_pathbuf_mode");
        let file1 = tmpdir.mash("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(file1.chmod(0o644).is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100644);
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_pathext_name() {
        assert!(PathBuf::from("").name().is_err());
        assert_eq!(PathBuf::from("foo").name().unwrap(), "foo");
        assert_eq!(PathBuf::from("foo.exe").name().unwrap(), "foo");
        assert_eq!(PathBuf::from("/foo/bar.exe").name().unwrap(), "bar");
    }

    #[test]
    fn test_pathext_perms() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_pathbuf_perms");
        let file1 = tmpdir.mash("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(file1.chmod(0o644).is_ok());
        assert_eq!(file1.perms().unwrap().mode(), 0o100644);
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_pathext_setperms() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("path_pathbuf_setperms");
        let file1 = tmpdir.mash("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());
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
    fn test_pathext_relative_from() {
        let cwd = env::current_dir().unwrap();

        // same directory
        assert_eq!(PathBuf::from("bar1").relative_from("bar1").unwrap(), cwd.mash("bar1"));

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
        assert_eq!(PathBuf::from("").trim_ext().unwrap(), PathBuf::new());
        assert_eq!(PathBuf::from("foo").trim_ext().unwrap(), PathBuf::from("foo"));
        assert_eq!(PathBuf::from("foo.exe").trim_ext().unwrap(), PathBuf::from("foo"));
        assert_eq!(PathBuf::from("/foo/bar.exe").trim_ext().unwrap(), PathBuf::from("/foo/bar"));
    }

    #[test]
    fn test_pathext_trim_last() {
        assert_eq!(PathBuf::new(), PathBuf::from("/").trim_last());
        assert_eq!(PathBuf::from("/"), PathBuf::from("/foo").trim_last());
    }

    #[test]
    fn test_pathext_trim_first() {
        assert_eq!(PathBuf::new(), PathBuf::from("/").trim_first());
        assert_eq!(PathBuf::from("foo"), PathBuf::from("/foo").trim_first());
    }

    #[test]
    fn test_pathext_trim_prefix() {
        // drop root
        assert_eq!(PathBuf::from("/").trim_prefix("/"), PathBuf::new());

        // drop start
        assert_eq!(Path::new("/foo/bar").trim_prefix("/foo"), PathBuf::from("/bar"));

        // no change
        assert_eq!(PathBuf::from("/").trim_prefix(""), PathBuf::from("/"));
        assert_eq!(PathBuf::from("/foo").trim_prefix("blah"), PathBuf::from("/foo"));
    }

    #[test]
    fn test_pathext_trim_protocol() {
        // no change
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo").trim_protocol());

        // file://
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("file:///foo").trim_protocol());

        // ftp://
        assert_eq!(PathBuf::from("foo"), PathBuf::from("ftp://foo").trim_protocol());

        // http://
        assert_eq!(PathBuf::from("foo"), PathBuf::from("http://foo").trim_protocol());

        // https://
        assert_eq!(PathBuf::from("foo"), PathBuf::from("https://foo").trim_protocol());

        // Check case is being considered
        assert_eq!(PathBuf::from("Foo"), PathBuf::from("HTTPS://Foo").trim_protocol());
        assert_eq!(PathBuf::from("Foo"), PathBuf::from("Https://Foo").trim_protocol());
        assert_eq!(PathBuf::from("FoO"), PathBuf::from("HttpS://FoO").trim_protocol());

        // Check non protocol matches are ignored
        assert_eq!(PathBuf::from("foo"), PathBuf::from("foo").trim_protocol());
        assert_eq!(PathBuf::from("foo/bar"), PathBuf::from("foo/bar").trim_protocol());
        assert_eq!(PathBuf::from("foo//bar"), PathBuf::from("foo//bar").trim_protocol());
        assert_eq!(PathBuf::from("ntp:://foo"), PathBuf::from("ntp:://foo").trim_protocol());
    }

    #[test]
    fn test_pathext_trim_suffix() {
        // drop root
        assert_eq!(PathBuf::new(), PathBuf::from("/").trim_suffix("/"));

        // drop end
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo/").trim_suffix("/"));

        // no change
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo").trim_suffix("/"));
    }

    #[test]
    fn test_pathcoloredext() {
        assert!(PathBuf::from("foo").black().is_ok());
        assert!(PathBuf::from("foo").red().is_ok());
        assert!(PathBuf::from("foo").green().is_ok());
        assert!(PathBuf::from("foo").yellow().is_ok());
        assert!(PathBuf::from("foo").blue().is_ok());
        assert!(PathBuf::from("foo").magenta().is_ok());
        assert!(PathBuf::from("foo").purple().is_ok());
        assert!(PathBuf::from("foo").cyan().is_ok());
        assert!(PathBuf::from("foo").white().is_ok());
        assert!(PathBuf::from("foo").bright_black().is_ok());
        assert!(PathBuf::from("foo").bright_red().is_ok());
        assert!(PathBuf::from("foo").bright_green().is_ok());
        assert!(PathBuf::from("foo").bright_yellow().is_ok());
        assert!(PathBuf::from("foo").bright_blue().is_ok());
        assert!(PathBuf::from("foo").bright_magenta().is_ok());
        assert!(PathBuf::from("foo").bright_purple().is_ok());
        assert!(PathBuf::from("foo").bright_cyan().is_ok());
        assert!(PathBuf::from("foo").bright_white().is_ok());
        assert!(PathBuf::from("foo").color(Color::Blue).is_ok());
        assert!(PathBuf::from("foo").on_black().is_ok());
        assert!(PathBuf::from("foo").on_red().is_ok());
        assert!(PathBuf::from("foo").on_green().is_ok());
        assert!(PathBuf::from("foo").on_yellow().is_ok());
        assert!(PathBuf::from("foo").on_blue().is_ok());
        assert!(PathBuf::from("foo").on_magenta().is_ok());
        assert!(PathBuf::from("foo").on_purple().is_ok());
        assert!(PathBuf::from("foo").on_cyan().is_ok());
        assert!(PathBuf::from("foo").on_white().is_ok());
        assert!(PathBuf::from("foo").on_bright_black().is_ok());
        assert!(PathBuf::from("foo").on_bright_red().is_ok());
        assert!(PathBuf::from("foo").on_bright_green().is_ok());
        assert!(PathBuf::from("foo").on_bright_yellow().is_ok());
        assert!(PathBuf::from("foo").on_bright_blue().is_ok());
        assert!(PathBuf::from("foo").on_bright_magenta().is_ok());
        assert!(PathBuf::from("foo").on_bright_purple().is_ok());
        assert!(PathBuf::from("foo").on_bright_cyan().is_ok());
        assert!(PathBuf::from("foo").on_bright_white().is_ok());
        assert!(PathBuf::from("foo").on_color(Color::Blue).is_ok());
        assert!(PathBuf::from("foo").normal().is_ok());
        assert!(PathBuf::from("foo").bold().is_ok());
        assert!(PathBuf::from("foo").dimmed().is_ok());
        assert!(PathBuf::from("foo").italic().is_ok());
        assert!(PathBuf::from("foo").underline().is_ok());
        assert!(PathBuf::from("foo").blink().is_ok());
        assert!(PathBuf::from("foo").reverse().is_ok());
        assert!(PathBuf::from("foo").reversed().is_ok());
        assert!(PathBuf::from("foo").hidden().is_ok());
        assert!(PathBuf::from("foo").strikethrough().is_ok());
    }
}
