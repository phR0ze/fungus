use std::fs;
use std::fs::File;
use std::os::unix;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::core::*;
use crate::path::PathExt;

/// Chmod provides flexible options for changing file permission with optional configuration.
#[derive(Debug, Clone)]
pub struct Chmod {
    path: PathBuf,     // path to chmod
    only_dirs: bool,   // chmod only dirs
    only_files: bool,  // chmod only files
    recurse_set: bool, // chmod recursively
}

impl Chmod {
    /// Update the `recurse` option. Default is enabled.
    /// When `yes` is `true`, source is recursively walked.
    pub fn recurse(mut self, yes: bool) -> Self {
        self.recurse_set = yes;
        self
    }

    /// Execute chmod all dirs/files using the given `mode`.
    pub fn all(mut self, mode: u32) -> Result<Self> {
        self.only_dirs = false;
        self.only_files = false;
        self.e(mode)
    }

    /// Execute chmod only dirs using the given `mode`.
    pub fn dirs(mut self, mode: u32) -> Result<Self> {
        self.only_dirs = true;
        self.only_files = false;
        self.e(mode)
    }

    /// Execute chmod only files using the given `mode`.
    pub fn files(mut self, mode: u32) -> Result<Self> {
        self.only_dirs = false;
        self.only_files = true;
        self.e(mode)
    }

    /// Internal implementation that the helper functions call
    fn e(self, mode: u32) -> Result<Self> {
        let (recurse, dirs, files) = ((&self).recurse_set, (&self).only_dirs, (&self).only_files);

        // Handle globbing
        let sources = crate::path::glob(&self.path)?;
        if sources.len() == 0 {
            return Err(PathError::does_not_exist(&self.path).into());
        }

        // Execute the chmod for all sources
        for source in sources {
            let (is_dir, old_mode) = match dirs || files || recurse {
                true => (source.is_dir(), source.mode()?),
                false => (false, 0),
            };

            // Grant permissions on the way in
            if (!dirs && !files) || (dirs && is_dir) || (files && !is_dir) {
                if !recurse || !is_dir || (recurse && !revoking_mode(old_mode, mode)) {
                    source.setperms(fs::Permissions::from_mode(mode))?;
                }
            }

            // Handle recursion
            if recurse && is_dir {
                for path in crate::path::paths(&source)? {
                    let modder = chmod_p(path)?.recurse(recurse);
                    let result = match (dirs, files) {
                        (true, false) => modder.dirs(mode),
                        (false, true) => modder.files(mode),
                        _ => modder.all(mode),
                    };
                    result?;
                }
            }

            // Revoke permissions on the way out
            if (!dirs && !files) || (dirs && is_dir) || (files && !is_dir) {
                if recurse && is_dir && revoking_mode(old_mode, mode) {
                    source.setperms(fs::Permissions::from_mode(mode))?;
                }
            }
        }

        Ok(self)
    }
}

/// Wraps `chmod_p` to apply the given `mode` to all files/dirs using recursion and invoking
/// the mode change when yes: bool
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_chmod");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::chmod(&file1, 0o644).is_ok());
/// assert_eq!(file1.mode().unwrap(), 0o100644);
/// assert!(sys::chmod(&file1, 0o555).is_ok());
/// assert_eq!(file1.mode().unwrap(), 0o100555);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn chmod<T: AsRef<Path>>(path: T, mode: u32) -> Result<Chmod> {
    chmod_p(path)?.all(mode)
}

/// Change the mode of the `path` providing path expansion, globbing, recursion and error
/// tracing. Provides more control over options than the `chmod` function. Changes are not
/// invoked until one of the helper functions are called e.g. `all`, `dirs` or `files`.
/// Symbolic links will have the target mode changed.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_chmod_p");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::chmod_p(&file1).unwrap().all(0o644).is_ok());
/// assert_eq!(file1.mode().unwrap(), 0o100644);
/// assert!(sys::chmod_p(&file1).unwrap().all(0o555).is_ok());
/// assert_eq!(file1.mode().unwrap(), 0o100555);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn chmod_p<T: AsRef<Path>>(path: T) -> Result<Chmod> {
    Ok(Chmod { path: path.as_ref().abs()?, only_dirs: false, only_files: false, recurse_set: true })
}

/// Change the ownership of the `path` providing path expansion, globbing, recursion and error
/// tracing.
///
//// ### Examples
/// ```
/// use fungus::presys::*;
/// ```
pub fn chown<T: AsRef<Path>>(path: T, uid: u32, gid: u32) -> Result<()> {
    let abs = path.as_ref().abs()?;

    Ok(())
}

/// Copies src to dst recursively creating destination directories as needed and handling path
/// expansion and globbing e.g. copy("./*", "../") and returning an absolute path of the
/// destination.
///
/// The dst will be copied to if it is an existing directory.
/// The dst will be a clone of the src if it doesn't exist.
/// Doesn't follow links
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_copy");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let file2 = tmpdir.mash("file2");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::copy(&file1, &file2).is_ok());
/// assert_eq!(file2.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn copy<T: AsRef<Path>, U: AsRef<Path>>(src: T, dst: U) -> Result<PathBuf> {
    let mut clone = true;
    let dstabs = dst.as_ref().abs()?;

    // Handle globbing
    let sources = crate::path::glob(&src)?;
    if sources.len() == 0 {
        return Err(PathError::does_not_exist(&src).into());
    }

    // Copy into destination vs clone as destination
    if dstabs.is_dir() || sources.len() > 1 {
        clone = false;
    }

    // Recurse on sources
    for srcroot in sources {
        for entry in WalkDir::new(&srcroot).follow_links(false).sort_by(|x, y| x.file_name().cmp(y.file_name())) {
            let entry = entry?;
            let srcpath = entry.path().abs()?;

            // Set proper dst path
            let dstpath = match clone {
                true => dstabs.mash(srcpath.trim_prefix(&srcroot)),
                false => dstabs.mash(srcpath.trim_prefix(srcroot.dir()?)),
            };
            match &srcpath {
                // Copy dir links needs to be first as is_dir follows links
                x if x.is_symlink_dir() => {
                    symlink(&dstpath, srcpath.readlink()?)?;
                }

                // Create destination directories as needed
                x if x.is_dir() => {
                    mkdir(dstpath)?;
                }

                // Copy file
                _ => {
                    copyfile(&srcpath, &dstpath)?;
                }
            }
        }
    }

    Ok(dstabs)
}

/// Copyfile provides a flexible options for copying files
#[derive(Debug)]
pub struct Copyfile {
    src: PathBuf,       // source file
    dst: PathBuf,       // destination path
    mode: u32,          // mode to chmod the file to
    mode_set: bool,     // track if the mode was set
    follow_links: bool, // follow links when copying files
}

impl Copyfile {
    /// Update the `follow_links` option. Default is disabled.
    /// When `yes` is `true`, links are followed.
    pub fn follow_links(mut self, yes: bool) -> Self {
        self.follow_links = yes;
        self
    }

    /// Update the `mode` option. Default is disabled.
    pub fn mode(mut self, mode: u32) -> Self {
        self.mode = mode;
        self.mode_set = true;
        self
    }

    /// Execute the copyfile operation with the current options.
    pub fn e(mut self) -> Result<Self> {
        // Configure and check source
        if !self.src.exists() {
            return Err(PathError::does_not_exist(&self.src).into());
        }
        if self.src.is_dir() || self.src.is_symlink_dir() {
            return Err(PathError::is_not_file_or_symlink_to_file(&self.src).into());
        }

        // Configure and check the destination
        match self.dst.exists() {
            // Exists so dst is either a file to overwrite or a dir to copy into
            true => {
                if self.dst.is_dir() {
                    self.dst = self.dst.mash(self.src.base()?)
                }
            }

            // Doesn't exist so dst is a new destination name, ensure all paths exist
            false => {
                let srcdir = self.src.dir()?;
                let dstdir = self.dst.dir()?;
                if srcdir != dstdir {
                    mkdir(dstdir)?.chmod(srcdir.mode()?)?;
                }
            }
        }

        // Check for same file
        if self.src == self.dst {
            return Ok(self);
        }

        // Recreate link or copy file including permissions
        if self.src.is_symlink() {
            symlink(&self.dst, self.src.readlink()?)?;
        } else {
            fs::copy(&self.src, &self.dst)?;
            if self.mode_set {
                chmod_p(&self.dst)?.recurse(false).all(self.mode)?;
            }
        }

        Ok(self)
    }
}

/// Wraps `copyfile_p` to copy the given `src` to the given `dst`. Disables follow_links
/// and uses the source mode rather than an optionally set one.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_copyfile");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let file2 = tmpdir.mash("file2");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::copyfile(&file1, &file2).is_ok());
/// assert_eq!(file2.exists(), true);
/// assert_eq!(file1.mode().unwrap(), file2.mode().unwrap());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn copyfile<T: AsRef<Path>, U: AsRef<Path>>(src: T, dst: U) -> Result<Copyfile> {
    copyfile_p(src, dst)?.e()
}

/// Copies a single file from src to dst, creating destination directories as needed and handling
/// path expansion and optionally following links.
///
/// The dst will be copied to if it is an existing directory.
/// The dst will be a clone of the src if it doesn't exist.
/// Doesn't follow links
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_copyfile");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let file2 = tmpdir.mash("file2");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch_p(&file1, 0o644).is_ok());
/// assert!(sys::copyfile_p(&file1, &file2).unwrap().mode(0o555).e().is_ok());
/// assert_eq!(file2.mode().unwrap(), 0o100555);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn copyfile_p<T: AsRef<Path>, U: AsRef<Path>>(src: T, dst: U) -> Result<Copyfile> {
    Ok(Copyfile { src: src.as_ref().abs()?, dst: dst.as_ref().abs()?, mode: 0, mode_set: false, follow_links: false })
}

/// Creates the given directory and any parent directories needed, handling path expansion and
/// returning an absolute path created.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_mkdir");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert_eq!(tmpdir.exists(), false);
/// ```
pub fn mkdir<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    let abs = path.as_ref().abs()?;
    if !abs.exists() {
        fs::create_dir_all(&abs)?;
    }
    Ok(abs)
}

/// Wraps `mkdir` allowing for setting the directory's mode.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_mkdir");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir_p(&tmpdir, 0o555).is_ok());
/// assert_eq!(tmpdir.mode().unwrap(), 0o40555);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn mkdir_p<T: AsRef<Path>>(path: T, mode: u32) -> Result<PathBuf> {
    let path = mkdir(path)?;
    chmod_p(&path)?.recurse(false).all(mode)?;
    Ok(path)
}

/// Removes the given empty directory or file. Handles path expansion. Does
/// not follow symbolic links but rather removes the links themselves.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_remove");
/// assert!(sys::mkdir(&tmpdir).is_ok());
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
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_remove_all");
/// assert!(sys::mkdir(&tmpdir).is_ok());
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

/// Returns true if the new mode is revoking permissions as compared to the old mode as pertains
/// directory read/execute permissions. This is useful when recursively modifying file permissions.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// assert_eq!(sys::revoking_mode(0o0777, 0o0777), false);
/// ```
pub fn revoking_mode(old: u32, new: u32) -> bool {
    old & 0o0500 > new & 0o0500 || old & 0o0050 > new & 0o0050 || old & 0o0005 > new & 0o0005
}

/// Creates a new symbolic link. Handles path expansion and returns an absolute path to the
/// link while still creating the symbolic link as a relative path to the target.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_symlink");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let link1 = tmpdir.mash("link1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
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
/// Uses default file creation permissions 0o666 - umask usually ends up being 0o644.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_touch");
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch(&tmpfile).is_ok());
/// assert_eq!(tmpfile.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn touch<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    let abs = path.as_ref().abs()?;
    if !abs.exists() {
        File::create(&abs)?;
    }
    Ok(abs)
}

/// Wraps `touch` allowing for setting the file's mode.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_touch");
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch_p(&tmpfile, 0o555).is_ok());
/// assert_eq!(tmpfile.mode().unwrap(), 0o100555);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn touch_p<T: AsRef<Path>>(path: T, mode: u32) -> Result<PathBuf> {
    let abs = touch(path)?;
    chmod_p(&abs)?.recurse(false).all(mode)?;
    Ok(abs)
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
            sys::mkdir(&setup.temp).unwrap();
            setup
        }
    }

    #[test]
    fn test_chmod() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("chmod");
        let file1 = tmpdir.mash("file1");

        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        assert!(sys::touch(&file1).is_ok());
        assert!(sys::chmod(&file1, 0o644).is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100644);
        assert!(sys::chmod(&file1, 0o555).is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100555);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_chmod_p() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("chmod_p");
        let dir1 = tmpdir.mash("dir1");
        let file1 = dir1.mash("file1");
        let dir2 = dir1.mash("dir2");
        let file2 = dir2.mash("file2");
        let file3 = tmpdir.mash("file3");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&dir2).is_ok());
        assert!(sys::touch_p(&file1, 0o644).is_ok());
        assert!(sys::touch_p(&file2, 0o644).is_ok());

        // all files
        assert!(sys::chmod_p(&dir1).unwrap().all(0o600).is_ok());
        assert_eq!(dir1.mode().unwrap(), 0o40600);

        // now fix dirs only to allow for listing directries
        assert!(sys::chmod_p(&dir1).unwrap().dirs(0o755).is_ok());
        assert_eq!(dir1.mode().unwrap(), 0o40755);
        assert_eq!(file1.mode().unwrap(), 0o100600);
        assert_eq!(dir2.mode().unwrap(), 0o40755);
        assert_eq!(file2.mode().unwrap(), 0o100600);

        // now change just the files back to 644
        assert!(sys::chmod_p(&dir1).unwrap().files(0o644).is_ok());
        assert_eq!(dir1.mode().unwrap(), 0o40755);
        assert_eq!(file1.mode().unwrap(), 0o100644);
        assert_eq!(dir2.mode().unwrap(), 0o40755);
        assert_eq!(file2.mode().unwrap(), 0o100644);

        // try globbing
        assert!(sys::touch_p(&file3, 0o644).is_ok());
        assert!(sys::chmod_p(tmpdir.mash("*3")).unwrap().files(0o555).is_ok());
        assert_eq!(dir1.mode().unwrap(), 0o40755);
        assert_eq!(file1.mode().unwrap(), 0o100644);
        assert_eq!(dir2.mode().unwrap(), 0o40755);
        assert_eq!(file2.mode().unwrap(), 0o100644);
        assert_eq!(file2.mode().unwrap(), 0o100644);
        assert_eq!(file3.mode().unwrap(), 0o100555);

        // doesn't exist
        assert!(sys::chmod_p("bogus").unwrap().all(0o644).is_err());

        // no path given
        assert!(sys::chmod_p("").is_err());

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_copy_empty() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("copy_empty");
        let file1 = tmpdir.mash("file1");

        // source doesn't exist
        assert!(sys::copy("", &file1).is_err());
        assert_eq!(file1.exists(), false);
    }

    #[test]
    fn test_copy_link_dir() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("copy_link_dir");
        let dirlink = tmpdir.mash("dirlink");
        let dir1 = tmpdir.mash("dir1");
        let dir1file = dir1.mash("file");
        let dir1link = dir1.mash("link");
        let dir2 = tmpdir.mash("dir2");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // copy directory with files
        assert!(sys::mkdir(&dirlink).is_ok());
        assert!(sys::mkdir(&dir1).is_ok());
        assert!(sys::touch(&dir1file).is_ok());
        assert!(sys::symlink(&dir1link, "../dirlink").is_ok());
        assert!(sys::copy(&dir1, &dir2).is_ok());

        let paths = vec![
            tmpdir.mash("dir1"),
            tmpdir.mash("dir1/file"),
            tmpdir.mash("dir1/link"),
            tmpdir.mash("dir2"),
            tmpdir.mash("dir2/file"),
            tmpdir.mash("dir2/link"),
            tmpdir.mash("dirlink"),
        ];
        assert_iter_eq(sys::all_paths(&tmpdir).unwrap(), paths);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_copy_dir_copy() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("copy_dir_copy");
        let dir1 = tmpdir.mash("dir1");
        let dir1file = dir1.mash("file");
        let dir2 = tmpdir.mash("dir2");
        let dir3file = tmpdir.mash("dir2/dir1/file");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // copy directory with files
        assert!(sys::mkdir(&dir1).is_ok());
        assert!(sys::touch(&dir1file).is_ok());
        assert!(sys::mkdir(&dir2).is_ok());
        assert_eq!(dir3file.exists(), false);
        assert!(sys::copy(&dir1, &dir2).is_ok());

        let paths = vec![tmpdir.mash("dir1"), tmpdir.mash("dir1/file"), tmpdir.mash("dir2"), tmpdir.mash("dir2/dir1"), tmpdir.mash("dir2/dir1/file")];
        assert_iter_eq(sys::all_paths(&tmpdir).unwrap(), paths);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_copy_dir_clone() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("copy_dir_clone");
        let dir1 = tmpdir.mash("dir1");
        let dir1file = dir1.mash("file");
        let dir2 = tmpdir.mash("dir2");
        let dir2file = dir2.mash("file");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // copy directory with files
        assert!(sys::mkdir(&dir1).is_ok());
        assert!(sys::touch(&dir1file).is_ok());
        assert_eq!(dir2file.exists(), false);
        assert!(sys::copy(&dir1, &dir2).is_ok());

        let paths = vec![tmpdir.mash("dir1"), tmpdir.mash("dir1/file"), tmpdir.mash("dir2"), tmpdir.mash("dir2/file")];
        assert_iter_eq(sys::all_paths(&tmpdir).unwrap(), paths);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_copy_single_file() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("copy_single_file");
        let file1 = tmpdir.mash("file1");
        let file2 = tmpdir.mash("file2");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // copy single file
        assert!(sys::touch(&file1).is_ok());
        assert_eq!(file1.exists(), true);
        assert_eq!(file2.exists(), false);
        assert!(sys::copy(&file1, &file2).is_ok());

        let paths = vec![tmpdir.mash("file1"), tmpdir.mash("file2")];
        assert_iter_eq(sys::all_paths(&tmpdir).unwrap(), paths);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_copyfile() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("copyfile");
        let file1 = tmpdir.mash("file1");
        let file2 = tmpdir.mash("file2");
        let link1 = tmpdir.mash("link1");
        let link2 = tmpdir.mash("link2");
        let file3 = tmpdir.mash("dir1/file3");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // copy to same dir
        assert!(sys::touch(&file1).is_ok());
        assert_eq!(file1.exists(), true);
        assert_eq!(file2.exists(), false);
        assert!(sys::copyfile(&file1, &file2).is_ok());
        assert_eq!(file2.exists(), true);
        assert_eq!(file1.mode().unwrap(), file2.mode().unwrap());

        // copy a link
        assert!(sys::symlink(&link1, &file1).is_ok());
        assert_eq!(link2.exists(), false);
        assert!(sys::copyfile(&link1, &link2).is_ok());
        assert_eq!(link2.exists(), true);

        // copy to dir the doesn't exist
        assert_eq!(file3.exists(), false);
        assert!(sys::copyfile(&file1, &file3).is_ok());
        assert_eq!(file3.exists(), true);
        assert_eq!(tmpdir.mode().unwrap(), file3.dir().unwrap().mode().unwrap());
        assert_eq!(file1.mode().unwrap(), file3.mode().unwrap());

        // empty destination path
        assert!(sys::copyfile(&file1, "").is_err());

        // empty source path
        assert!(sys::copyfile("", &file3).is_err());

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_copyfile_p() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("copyfile_p");
        let file1 = tmpdir.mash("file1");
        let file2 = tmpdir.mash("file2");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // copy to same dir
        assert!(sys::touch_p(&file1, 0o644).is_ok());
        assert!(sys::copyfile_p(&file1, &file2).unwrap().mode(0o555).e().is_ok());
        assert_eq!(file2.mode().unwrap(), 0o100555);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_mkdir_p() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("mkdir_p");
        let dir1 = setup.temp.mash("dir1");
        let dir2 = setup.temp.mash("dir2");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());

        // test
        assert!(sys::mkdir(&dir1).is_ok());
        assert_eq!(dir1.mode().unwrap(), 0o40755);
        assert!(sys::mkdir_p(&dir2, 0o555).is_ok());
        assert_eq!(dir2.mode().unwrap(), 0o40555);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_remove() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("remove_dir");
        let tmpfile = setup.temp.mash("remove_file");

        // Remove empty directory
        assert!(sys::mkdir(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), true);
        assert!(sys::remove(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);

        // Remove file
        assert!(sys::touch(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), true);
        assert!(sys::remove(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), false);
    }

    #[test]
    fn test_remove_all() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("remove_all");

        assert!(sys::mkdir(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), true);
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_revoking() {
        // test other octet
        assert_eq!(sys::revoking_mode(0o0777, 0o0777), false);
        assert_eq!(sys::revoking_mode(0o0776, 0o0775), false);
        assert_eq!(sys::revoking_mode(0o0770, 0o0771), false);
        assert_eq!(sys::revoking_mode(0o0776, 0o0772), true);
        assert_eq!(sys::revoking_mode(0o0775, 0o0776), true);
        assert_eq!(sys::revoking_mode(0o0775, 0o0774), true);

        // Test group octet
        assert_eq!(sys::revoking_mode(0o0777, 0o0777), false);
        assert_eq!(sys::revoking_mode(0o0767, 0o0757), false);
        assert_eq!(sys::revoking_mode(0o0707, 0o0717), false);
        assert_eq!(sys::revoking_mode(0o0767, 0o0727), true);
        assert_eq!(sys::revoking_mode(0o0757, 0o0767), true);
        assert_eq!(sys::revoking_mode(0o0757, 0o0747), true);

        // Test owner octet
        assert_eq!(sys::revoking_mode(0o0777, 0o0777), false);
        assert_eq!(sys::revoking_mode(0o0677, 0o0577), false);
        assert_eq!(sys::revoking_mode(0o0077, 0o0177), false);
        assert_eq!(sys::revoking_mode(0o0677, 0o0277), true);
        assert_eq!(sys::revoking_mode(0o0577, 0o0677), true);
        assert_eq!(sys::revoking_mode(0o0577, 0o0477), true);
        assert_eq!(sys::revoking_mode(0o0577, 0o0177), true);
    }

    #[test]
    fn test_symlink() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("symlink");
        let file1 = tmpdir.mash("file1");
        let link1 = tmpdir.mash("link1");
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert!(sys::mkdir(&tmpdir).is_ok());
        assert!(sys::touch(&file1).is_ok());
        assert!(sys::symlink(&link1, &file1).is_ok());
        assert_eq!(link1.exists(), true);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_touch() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("touch");
        let tmpfile = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        assert!(sys::touch(&tmpfile).is_ok());

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_touch_p() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("touch_p");
        let tmpfile = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        assert!(sys::touch_p(&tmpfile, 0o555).is_ok());
        assert_eq!(tmpfile.mode().unwrap(), 0o100555);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
