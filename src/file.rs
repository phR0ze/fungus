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
    // let mut clone = true;
    let dstabs = dst.as_ref().abs()?;

    // // Handle globbing
    // let sources = crate::path::glob(&src)?;
    // if sources.len() == 0 {
    //     return Err(PathError::does_not_exist(&src).into());
    // }

    // // Copy into destination vs clone as destination
    // if dstabs.is_dir() || sources.len() > 1 {
    //     clone = false;
    // }

    // // Recurse on sources
    // for srcroot in sources {
    //     for entry in WalkDir::new(&srcroot).follow_links(false) {
    //         let entry = entry?;
    //         let srcpath = entry.path().abs()?;

    //         // Set proper dst path
    //         let dstpath = match clone {
    //             true => dstabs.join(srcpath.trim_prefix(srcroot)?),
    //             false => dstabs.join(srcpath.trim_prefix(srcroot)?),
    //         }
    //     }
    // }

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
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().join("doc_copyfile");
/// assert!(sys::mkdir_p(&tmpdir).is_ok());
/// let file1 = tmpdir.join("file1");
/// let file2 = tmpdir.join("file2");
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
                dstpath = dstpath.join(srcpath.base()?)
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
/// use fungus::presys::*;
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
/// use fungus::presys::*;
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
/// use fungus::presys::*;
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
/// Uses default file creation permissions 0o666 - umask usually ends up being 0o644.
///
/// ### Examples
/// ```
/// use fungus::presys::*;
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

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
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
    fn test_copyfile() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("copyfile");
        let file1 = tmpdir.join("file1");
        let file2 = tmpdir.join("file2");
        let link1 = tmpdir.join("link1");
        let link2 = tmpdir.join("link2");
        let file3 = tmpdir.join("dir1/file3");

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
        let tmpdir = setup.temp.join("remove_dir");
        let tmpfile = setup.temp.join("remove_file");

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
        let tmpdir = setup.temp.join("remove_all");

        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), true);
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_symlink() {
        let setup = Setup::init();
        let tmpdir = setup.temp.join("symlink");
        let file1 = tmpdir.join("file1");
        let link1 = tmpdir.join("link1");
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
        let tmpdir = setup.temp.join("touch");
        let tmpfile = tmpdir.join("file1");
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert!(sys::mkdir_p(&tmpdir).is_ok());
        assert!(sys::touch(&tmpfile).is_ok());
        assert_eq!(tmpfile.exists(), true);

        // Clean up
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }
}
