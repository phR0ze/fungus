use std::fs;
use std::fs::File;
use std::os::unix;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::core::*;
use crate::path::PathExt;

/// Copies src to dst recursively, creating destination directories as needed and handling path
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
                    mkdir_p(dstpath)?;
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

/// Copies a single file from src to dst, creating destination directories as needed and handling
/// path expansion returning an absolute path of the destination.
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
/// assert!(sys::mkdir_p(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let file2 = tmpdir.mash("file2");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir_p(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::copyfile(&file1, &file2).is_ok());
/// assert_eq!(file2.exists(), true);
/// assert_eq!(file1.mode().unwrap(), file2.mode().unwrap());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn copyfile<T: AsRef<Path>, U: AsRef<Path>>(src: T, dst: U) -> Result<PathBuf> {
    // Configure and check source
    let srcpath = src.as_ref().abs()?;
    if !srcpath.exists() {
        return Err(PathError::does_not_exist(src).into());
    }
    if srcpath.is_dir() || srcpath.is_symlink_dir() {
        return Err(PathError::is_not_file_or_symlink_to_file(&src).into());
    }

    // Configure and check the destination
    let mut dstpath = dst.as_ref().abs()?;
    match dstpath.exists() {
        // Exists so dst is either a file to overwrite or a dir to copy into
        true => {
            if dstpath.is_dir() {
                dstpath = dstpath.mash(srcpath.base()?)
            }
        }

        // Doesn't exist so dst is a new destination name, ensure all paths exist
        false => {
            let srcdir = srcpath.dir()?;
            let dstdir = dstpath.dir()?;
            if srcdir != dstdir {
                mkdir_p(dstdir)?.chmod(srcdir.mode()?)?;
            }
        }
    }

    // Check for same file
    if srcpath == dstpath {
        return Ok(dstpath);
    }

    // Recreate link or copy file including permissions
    if srcpath.is_symlink() {
        symlink(&dstpath, srcpath.readlink()?)?;
    } else {
        fs::copy(&srcpath, &dstpath)?;
    }

    Ok(dstpath)
}

/// Creates the given directory and any parent directories needed, handling path expansion and
/// returning an absolute path created.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_mkdir_p");
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
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_remove");
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
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_remove_all");
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
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_symlink");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let link1 = tmpdir.mash("link1");
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
/// Uses default file creation permissions 0o666 - umask usually ends up being 0o644.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("doc_touch");
/// let tmpfile = tmpdir.mash("file1");
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
        assert!(sys::mkdir_p(&tmpdir).is_ok());

        // copy directory with files
        assert!(sys::mkdir_p(&dirlink).is_ok());
        assert!(sys::mkdir_p(&dir1).is_ok());
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
        assert!(sys::mkdir_p(&tmpdir).is_ok());

        // copy directory with files
        assert!(sys::mkdir_p(&dir1).is_ok());
        assert!(sys::touch(&dir1file).is_ok());
        assert!(sys::mkdir_p(&dir2).is_ok());
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
        assert!(sys::mkdir_p(&tmpdir).is_ok());

        // copy directory with files
        assert!(sys::mkdir_p(&dir1).is_ok());
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
        assert!(sys::mkdir_p(&tmpdir).is_ok());

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
        assert!(sys::mkdir_p(&tmpdir).is_ok());

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
    fn test_remove() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("remove_dir");
        let tmpfile = setup.temp.mash("remove_file");

        // Remove empty directory
        assert!(sys::mkdir_p(&tmpdir).is_ok());
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

        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), true);
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_symlink() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("symlink");
        let file1 = tmpdir.mash("file1");
        let link1 = tmpdir.mash("link1");
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert!(sys::mkdir_p(&tmpdir).is_ok());
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
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert!(sys::touch(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), true);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }
}
