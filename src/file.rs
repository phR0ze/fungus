#[cfg(feature = "_crypto_")]
use blake2::{Blake2b, Digest};
#[cfg(feature = "_libc_")]
use libc;
#[cfg(feature = "_libc_")]
use std::ffi::CString;
#[cfg(feature = "libc_")]
use std::io;
#[cfg(feature = "_libc_")]
use std::os::unix::ffi::OsStrExt;

use std::fs::{self, File};
use std::io::{self, prelude::*, BufRead, BufReader};
use std::os::unix::{self, fs::PermissionsExt};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::prelude::*;

/// Chmod provides flexible options for changing file permission with optional configuration.
#[derive(Debug, Clone)]
pub struct Chmod {
    mode: u32,       // mode to use
    path: PathBuf,   // path to chmod
    dirs: bool,      // chmod only dirs
    files: bool,     // chmod only files
    recursive: bool, // chmod recursively
}

impl Chmod {
    /// Target all files and directories. Default is enabled.
    pub fn all(&mut self) -> &mut Self {
        self.dirs = false;
        self.files = false;
        self
    }

    /// Target only directories. Default is disabled.
    pub fn dirs(&mut self) -> &mut Self {
        self.dirs = true;
        self.files = false;
        self
    }

    /// Target only files. Default is disabled.
    pub fn files(&mut self) -> &mut Self {
        self.dirs = false;
        self.files = true;
        self
    }

    /// Update the `mode` option. Default is the current file's mode or to 0o644 if it doesn't exist
    pub fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Update the `mode` option to add read permission to all.
    pub fn add_r(&mut self) -> &mut Self {
        self.mode |= 0o0444;
        self
    }

    /// Update the `mode` option to add write permission to all.
    pub fn add_w(&mut self) -> &mut Self {
        self.mode |= 0o0222;
        self
    }

    /// Update the `mode` option to add execute permission to all.
    pub fn add_x(&mut self) -> &mut Self {
        self.mode |= 0o0111;
        self
    }

    /// Update the `mode` option to make readonly.
    pub fn readonly(&mut self) -> &mut Self {
        self.sub_w().sub_x()
    }

    /// Update the `mode` option to drop group and other permissions.
    pub fn secure(&mut self) -> &mut Self {
        self.mode &= 0o7700;
        self
    }

    /// Update the `mode` option to subtract read permission from all.
    pub fn sub_r(&mut self) -> &mut Self {
        self.mode &= 0o7333;
        self
    }

    /// Update the `mode` option to subtract write permission from all.
    pub fn sub_w(&mut self) -> &mut Self {
        self.mode &= 0o7555;
        self
    }

    /// Update the `mode` option to subtract execute permission from all.
    pub fn sub_x(&mut self) -> &mut Self {
        self.mode &= 0o7666;
        self
    }

    /// Update the `path` option.
    pub fn path<T: AsRef<Path>>(&mut self, path: T) -> &mut Self {
        self.path = path.as_ref().to_path_buf();
        self
    }

    /// Update the `recurse` option. Default is enabled.
    /// When `yes` is `true`, source is recursively walked.
    pub fn recurse(&mut self, yes: bool) -> &mut Self {
        self.recursive = yes;
        self
    }

    /// Execute the [`Chmod`] options against the set `path` with the set `mode`.
    pub fn chmod(&self) -> Result<()> {
        // Handle globbing
        let sources = crate::path::glob(&self.path)?;
        if sources.len() == 0 {
            return Err(PathError::does_not_exist(&self.path).into());
        }

        // Execute the chmod for all sources
        for source in sources {
            let (is_dir, old_mode) = match self.dirs || self.files || self.recursive {
                true => (source.is_dir(), source.mode()?),
                false => (false, 0),
            };

            // Grant permissions on the way in
            if (!self.dirs && !self.files) || (self.dirs && is_dir) || (self.files && !is_dir) {
                if !self.recursive || !is_dir || (self.recursive && !revoking_mode(old_mode, self.mode)) {
                    source.setperms(fs::Permissions::from_mode(self.mode))?;
                }
            }

            // Handle recursion
            if self.recursive && is_dir {
                for path in crate::path::paths(&source)? {
                    self.clone().path(path).chmod()?;
                }
            }

            // Revoke permissions on the way out
            if (!self.dirs && !self.files) || (self.dirs && is_dir) || (self.files && !is_dir) {
                if self.recursive && is_dir && revoking_mode(old_mode, self.mode) {
                    source.setperms(fs::Permissions::from_mode(self.mode))?;
                }
            }
        }
        Ok(())
    }
}

/// Wraps `chmod_p` to apply the given `mode` to all files/dirs using recursion and invoking
/// the mode change on the close of this function call.
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_chmod");
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
pub fn chmod<T: AsRef<Path>>(path: T, mode: u32) -> Result<()> {
    chmod_p(path)?.mode(mode).chmod()
}

/// Create [`Chmod`] options providing path expansion, globbing, recursion and error
/// tracing while setting the `mode`. This function provides more control over options
/// than the `chmod` function. Changes are not invoked until the `chmod` method is called.
/// Symbolic links will have the target mode changed.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_chmod_p");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::chmod_p(&file1).unwrap().mode(0o644).chmod().is_ok());
/// assert_eq!(file1.mode().unwrap(), 0o100644);
/// assert!(sys::chmod_p(&file1).unwrap().mode(0o555).chmod().is_ok());
/// assert_eq!(file1.mode().unwrap(), 0o100555);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn chmod_p<T: AsRef<Path>>(path: T) -> Result<Chmod> {
    let path = path.as_ref().abs()?;
    let mode = match path.mode() {
        Ok(x) => x,
        _ => 0o644,
    };
    Ok(Chmod { path: path, mode: mode, dirs: false, files: false, recursive: true })
}

/// Change the ownership of the `path` providing path expansion, globbing, recursion and error
/// tracing. Follows links.
///
//// ### Examples
/// ```ignore
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_chown");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::chown(&file1, user::getuid(), user::getgid()).is_ok());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(feature = "_libc_")]
pub fn chown<T: AsRef<Path>>(path: T, uid: u32, gid: u32) -> Result<()> {
    chown_p(path, uid, gid, true)
}

/// Change the ownership of the `path` providing path expansion, globbing, recursion and error
/// tracing. Does not follow links.
///
//// ### Examples
/// ```ignore
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_chown");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::chown(&file1, user::getuid(), user::getgid()).is_ok());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(feature = "_libc_")]
pub fn lchown<T: AsRef<Path>>(path: T, uid: u32, gid: u32) -> Result<()> {
    chown_p(path, uid, gid, false)
}

/// Private implementation of chown
#[cfg(feature = "_libc_")]
fn chown_p<T: AsRef<Path>>(path: T, uid: u32, gid: u32, follow: bool) -> Result<()> {
    let path = path.as_ref().abs()?;

    // Handle globbing
    let sources = crate::path::glob(&path)?;
    if sources.len() == 0 {
        return Err(PathError::does_not_exist(&path).into());
    }

    // Execute the chmod for all sources
    for source in sources {
        for entry in WalkDir::new(&source).follow_links(false).sort_by(|x, y| x.file_name().cmp(y.file_name())) {
            let srcpath = entry?.into_path();
            let osstr = CString::new(srcpath.as_os_str().as_bytes())?;
            let ret = unsafe {
                if follow {
                    libc::chown(osstr.as_ptr(), uid, gid)
                } else {
                    libc::lchown(osstr.as_ptr(), uid, gid)
                }
            };
            if ret != 0 {
                return Err(io::Error::last_os_error().into());
            }
        }
    }
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
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_copy");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let file2 = tmpdir.mash("file2");
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
            let srcpath = entry?.into_path();

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
    mode: Option<u32>,  // mode to chmod the file to if set
    follow_links: bool, // follow links when copying files
}

impl Copyfile {
    /// Update the `follow` option. Default is disabled.
    /// When `yes` is `true`, links are followed.
    pub fn follow(&mut self, yes: bool) -> &mut Self {
        self.follow_links = yes;
        self
    }

    /// Update the `mode` option. Default is disabled.
    pub fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = Some(mode);
        self
    }

    /// Execute the copyfile operation with the current options.
    pub fn copy(&mut self) -> Result<PathBuf> {
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
            return Ok(self.dst.clone());
        }

        // Recreate link or copy file including permissions
        if self.src.is_symlink() {
            symlink(&self.dst, self.src.readlink()?)?;
        } else {
            fs::copy(&self.src, &self.dst)?;
            if let Some(mode) = self.mode {
                chmod_p(&self.dst)?.mode(mode).recurse(false).chmod()?;
            }
        }

        Ok(self.dst.clone())
    }
}

/// Wraps `copyfile_p` to copy the given `src` to the given `dst`. Disables follow_links
/// and uses the source mode rather than an optionally set one.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_copyfile");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let file2 = tmpdir.mash("file2");
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::copyfile(&file1, &file2).is_ok());
/// assert_eq!(file2.exists(), true);
/// assert_eq!(file1.mode().unwrap(), file2.mode().unwrap());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn copyfile<T: AsRef<Path>, U: AsRef<Path>>(src: T, dst: U) -> Result<()> {
    copyfile_p(src, dst)?.copy()?;
    Ok(())
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
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_copyfile_p");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let file2 = tmpdir.mash("file2");
/// assert!(sys::touch_p(&file1, 0o644).is_ok());
/// assert!(sys::copyfile_p(&file1, &file2).unwrap().mode(0o555).copy().is_ok());
/// assert_eq!(file2.mode().unwrap(), 0o100555);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn copyfile_p<T: AsRef<Path>, U: AsRef<Path>>(src: T, dst: U) -> Result<Copyfile> {
    Ok(Copyfile { src: src.as_ref().abs()?, dst: dst.as_ref().abs()?, mode: None, follow_links: false })
}

/// Computes and returns the digest of the given `path`.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_digest");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let file2 = tmpdir.mash("file2");
/// assert!(sys::write(&file1, "this is a test").is_ok());
/// assert!(sys::copyfile(&file1, &file2).is_ok());
/// assert_iter_eq(sys::digest(&file1).unwrap(), sys::digest(&file2).unwrap());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(feature = "_crypto_")]
pub fn digest<T: AsRef<Path>>(path: T) -> Result<Vec<u8>> {
    Ok(Blake2b::digest(&readbytes(path)?).into_iter().collect())
}

/// Returns the first captured string from the given regular expression `rx`.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_extract_string");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let rx = Regex::new(r"'([^']+)'\s+\((\d{4})\)").unwrap();
/// assert!(sys::write(&tmpfile, "Not my favorite movie: 'Citizen Kane' (1941).").is_ok());
/// assert_eq!(sys::extract_string(&tmpfile, &rx).unwrap(), "Citizen Kane");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn extract_string<T: AsRef<Path>>(path: T, rx: &Regex) -> Result<String> {
    let data = readstring(path)?;
    let caps = rx.captures(&data).ok_or_else(|| FileError::FailedToExtractString)?;
    let value = caps.get(1).ok_or_else(|| FileError::FailedToExtractString)?;
    Ok(value.as_str().to_string())
}

/// Returns the first captured string from the given regular expression `rx`.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_extract_string_p");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let rx = Regex::new(r"'([^']+)'\s+\((\d{4})\)").unwrap();
/// assert!(sys::write(&tmpfile, "Not my favorite movie: 'Citizen Kane' (1941).").is_ok());
/// assert_eq!(sys::extract_string(&tmpfile, &rx).unwrap(), "Citizen Kane");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn extract_string_p<T: AsRef<Path>, U: AsRef<str>>(path: T, rx: U) -> Result<String> {
    extract_string(path, &Regex::new(rx.as_ref())?)
}

/// Returns the captured strings from the given regular expression `rx`.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_extract_strings");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let rx = Regex::new(r"'([^']+)'\s+\((\d{4})\)").unwrap();
/// assert!(sys::write(&tmpfile, "Not my favorite movie: 'Citizen Kane' (1941).").is_ok());
/// assert_eq!(sys::extract_strings(&tmpfile, &rx).unwrap(), vec!["Citizen Kane", "1941"]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn extract_strings<T: AsRef<Path>>(path: T, rx: &Regex) -> Result<Vec<String>> {
    let data = readstring(path)?;
    let caps = rx.captures(&data).ok_or_else(|| FileError::FailedToExtractString)?;
    let values = caps.iter().skip(1).filter_map(|x| x).filter_map(|x| Some(x.as_str().to_string())).collect::<Vec<String>>();
    if values.len() == 0 {
        return Err(FileError::FailedToExtractString.into());
    }
    Ok(values)
}

/// Returns the captured strings from the given regular expression `rx`.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_extract_strings_p");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::write(&tmpfile, "Not my favorite movie: 'Citizen Kane' (1941).").is_ok());
/// assert_eq!(sys::extract_strings_p(&tmpfile, r"'([^']+)'\s+\((\d{4})\)").unwrap(), vec!["Citizen Kane", "1941"]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn extract_strings_p<T: AsRef<Path>, U: AsRef<str>>(path: T, rx: U) -> Result<Vec<String>> {
    extract_strings(path, &Regex::new(rx.as_ref())?)
}

/// Creates the given directory and any parent directories needed, handling path expansion and
/// returning an absolute path created.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_mkdir");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert_eq!(tmpdir.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn mkdir<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    let path = path.as_ref().abs()?;
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

/// Wraps `mkdir` allowing for setting the directory's mode.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_mkdir");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir_p(&tmpdir, 0o555).is_ok());
/// assert_eq!(tmpdir.mode().unwrap(), 0o40555);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn mkdir_p<T: AsRef<Path>>(path: T, mode: u32) -> Result<PathBuf> {
    let path = mkdir(path)?;
    chmod_p(&path)?.recurse(false).mode(mode).chmod()?;
    Ok(path)
}

/// Move a file or directory handling path expansion and globbing. Replaces destination files if
/// exist but always moves `src` into `dst` if `dst` is an existing directory.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_copy");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let file2 = tmpdir.mash("file2");
/// assert!(sys::touch(&file1).is_ok());
/// assert!(sys::move_p(&file1, &file2).is_ok());
/// assert_eq!(file1.exists(), false);
/// assert_eq!(file2.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn move_p<T: AsRef<Path>, U: AsRef<Path>>(src: T, dst: U) -> Result<()> {
    let src = src.as_ref().abs()?;

    // Handle globbing
    let sources = crate::path::glob(&src)?;
    if sources.len() == 0 {
        return Err(PathError::does_not_exist(&src).into());
    }

    // Test if dst exists and is a directory
    let dst = dst.as_ref().abs()?;
    let dst_is_dir = dst.is_dir();

    // Execute the move for all sources
    for source in sources {
        let dstpath = match dst_is_dir {
            true => dst.mash(src.base()?),
            false => dst.to_path_buf(),
        };
        fs::rename(source, dstpath)?;
    }
    Ok(())
}

/// Removes the given empty directory or file. Handles path expansion. Does
/// not follow symbolic links but rather removes the links themselves.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_remove");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::remove(&tmpdir).is_ok());
/// assert_eq!(tmpdir.exists(), false);
/// ```
pub fn remove<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref().abs()?;
    let wrapped_meta = fs::metadata(&path);
    if wrapped_meta.is_ok() {
        let meta = wrapped_meta.unwrap();
        if meta.is_file() {
            fs::remove_file(path)?;
        } else if meta.is_dir() {
            fs::remove_dir(path)?;
        }
    }
    Ok(())
}

/// Removes the given directory after removing all of its contents. Handles path expansion. Does
/// not follow symbolic links but rather removes the links themselves.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_remove_all");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert_eq!(tmpdir.exists(), false);
/// ```
pub fn remove_all<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref().abs()?;
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// Returns the contents of the `path` as a `Vec<u8>`.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_readbytes");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::write(&tmpfile, "this is a test").is_ok());
/// assert_eq!(str::from_utf8(&sys::readbytes(&tmpfile).unwrap()).unwrap(), "this is a test");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn readbytes<T: AsRef<Path>>(path: T) -> Result<Vec<u8>> {
    let path = path.as_ref().abs()?;
    match std::fs::read(path) {
        Ok(data) => Ok(data),
        Err(err) => Err(err.into()),
    }
}

/// Returns all lines from teh file as a `Vec<String>`.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_readlines");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::write(&tmpfile, "this is a test").is_ok());
/// assert_iter_eq(sys::readlines(&tmpfile).unwrap(), vec![String::from("this is a test")]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn readlines<T: AsRef<Path>>(path: T) -> Result<Vec<String>> {
    match readlines_p(path)?.collect::<io::Result<Vec<String>>>() {
        Ok(data) => Ok(data),
        Err(err) => Err(err.into()),
    }
}

/// Returns an Iterator to the Reader of the lines of the file.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_readlines_p");
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::write(&tmpfile, "this is a test").is_ok());
/// assert_iter_eq(sys::readlines_p(&tmpfile).unwrap().collect::<io::Result<Vec<String>>>().unwrap(), vec![String::from("this is a test")]);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn readlines_p<T: AsRef<Path>>(path: T) -> Result<io::Lines<BufReader<File>>> {
    let path = path.as_ref().abs()?;
    let file = File::open(path)?;
    Ok(BufReader::new(file).lines())
}

/// Returns the contents of the `path` as a `String`.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_readstring");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::write(&tmpfile, "this is a test").is_ok());
/// assert_eq!(sys::readstring(&tmpfile).unwrap(), "this is a test");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn readstring<T: AsRef<Path>>(path: T) -> Result<String> {
    let path = path.as_ref().abs()?;
    match std::fs::read_to_string(path) {
        Ok(data) => Ok(data),
        Err(err) => Err(err.into()),
    }
}

/// Returns true if the new mode is revoking permissions as compared to the old mode as pertains
/// directory read/execute permissions. This is useful when recursively modifying file permissions.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
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
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_symlink");
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
    let path = link.as_ref().abs()?;
    if path.exists() {
        return Err(PathError::exists_already(path).into());
    }
    unix::fs::symlink(target, &path)?;
    Ok(path)
}

/// Create an empty file similar to the linux touch command. Handles path expansion.
/// Uses default file creation permissions 0o666 - umask usually ends up being 0o644.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_touch");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch(&tmpfile).is_ok());
/// assert_eq!(tmpfile.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn touch<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    let path = path.as_ref().abs()?;
    if !path.exists() {
        File::create(&path)?;
    }
    Ok(path)
}

/// Wraps `touch` allowing for setting the file's mode.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_touch");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::touch_p(&tmpfile, 0o555).is_ok());
/// assert_eq!(tmpfile.mode().unwrap(), 0o100555);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn touch_p<T: AsRef<Path>>(path: T, mode: u32) -> Result<PathBuf> {
    let path = touch(path)?;
    chmod_p(&path)?.recurse(false).mode(mode).chmod()?;
    Ok(path)
}

/// Write `[u8]` data to a file which means `str` or `String`. Handles path expansion.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_write");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::write(&tmpfile, "this is a test").is_ok());
/// assert_eq!(sys::readstring(&tmpfile).unwrap(), "this is a test");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn write<T: AsRef<Path>, U: AsRef<[u8]>>(path: T, data: U) -> Result<()> {
    let path = path.as_ref().abs()?;
    let mut f = File::create(path)?;
    f.write(data.as_ref())?;

    // f.sync_all() works better than f.flush()?
    f.sync_all()?;
    Ok(())
}

/// Wraps `write` allowing for setting the file's mode.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_write_p");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// assert!(sys::write_p(&tmpfile, "this is a test", 0o666).is_ok());
/// assert_eq!(sys::readstring(&tmpfile).unwrap(), "this is a test");
/// assert_eq!(tmpfile.mode().unwrap(), 0o100666);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn write_p<T: AsRef<Path>, U: AsRef<[u8]>>(path: T, data: U, mode: u32) -> Result<()> {
    write(&path, data)?;
    chmod_p(&path)?.recurse(false).mode(mode).chmod()?;
    Ok(())
}

/// Write `&Vec<String>` data to a file as lines. Handles path expansion.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_writelines");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let lines = vec![String::from("one"), String::from("two")];
/// assert!(sys::writelines(&tmpfile, &lines).is_ok());
/// assert_iter_eq(sys::readlines(&tmpfile).unwrap(), lines);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn writelines<T: AsRef<Path>>(path: T, data: &Vec<String>) -> Result<()> {
    write(path, data.join("\n"))?;
    Ok(())
}

/// Wraps `writelines` allowing for setting the file's mode.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("file_doc_writelines_p");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("file1");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let lines = vec![String::from("one"), String::from("two")];
/// assert!(sys::writelines_p(&tmpfile, &lines, 0o666).is_ok());
/// assert_iter_eq(sys::readlines(&tmpfile).unwrap(), lines);
/// assert_eq!(tmpfile.mode().unwrap(), 0o100666);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
pub fn writelines_p<T: AsRef<Path>>(path: T, data: &Vec<String>, mode: u32) -> Result<()> {
    write(&path, data.join("\n"))?;
    chmod_p(&path)?.recurse(false).mode(mode).chmod()?;
    Ok(())
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
    fn test_chmod() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_chmod");
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
        let tmpdir = setup.temp.mash("file_chmod_p");
        let dir1 = tmpdir.mash("dir1");
        let file1 = dir1.mash("file1");
        let dir2 = dir1.mash("dir2");
        let file2 = dir2.mash("file2");
        let file3 = tmpdir.mash("file3");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&dir1).is_ok());
        assert!(sys::mkdir(&dir2).is_ok());
        assert!(sys::touch_p(&file1, 0o644).is_ok());
        assert!(sys::touch_p(&file2, 0o644).is_ok());

        // all files
        assert!(sys::chmod_p(&dir1).unwrap().mode(0o600).chmod().is_ok());
        assert_eq!(dir1.mode().unwrap(), 0o40600);

        // now fix dirs only to allow for listing directries
        assert!(sys::chmod_p(&dir1).unwrap().mode(0o755).dirs().chmod().is_ok());
        assert_eq!(dir1.mode().unwrap(), 0o40755);
        assert_eq!(file1.mode().unwrap(), 0o100600);
        assert_eq!(dir2.mode().unwrap(), 0o40755);
        assert_eq!(file2.mode().unwrap(), 0o100600);

        // now change just the files back to 644
        assert!(sys::chmod_p(&dir1).unwrap().mode(0o644).files().chmod().is_ok());
        assert_eq!(dir1.mode().unwrap(), 0o40755);
        assert_eq!(file1.mode().unwrap(), 0o100644);
        assert_eq!(dir2.mode().unwrap(), 0o40755);
        assert_eq!(file2.mode().unwrap(), 0o100644);

        // try globbing
        assert!(sys::touch_p(&file3, 0o644).is_ok());
        assert!(sys::chmod_p(tmpdir.mash("*3")).unwrap().mode(0o555).files().chmod().is_ok());
        assert_eq!(dir1.mode().unwrap(), 0o40755);
        assert_eq!(file1.mode().unwrap(), 0o100644);
        assert_eq!(dir2.mode().unwrap(), 0o40755);
        assert_eq!(file2.mode().unwrap(), 0o100644);
        assert_eq!(file2.mode().unwrap(), 0o100644);
        assert_eq!(file3.mode().unwrap(), 0o100555);

        // doesn't exist
        assert!(sys::chmod_p("bogus").unwrap().mode(0o644).chmod().is_err());

        // no path given
        assert!(sys::chmod_p("").is_err());

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_chmod_p_symbolic() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_chmod_p_symbolic");
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

        // sub_x
        assert!(sys::chmod_p(&file1).unwrap().sub_x().chmod().is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100644);
        assert_eq!(file1.is_exec(), false);

        // sub_w
        assert!(sys::chmod_p(&file1).unwrap().sub_w().chmod().is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100444);
        assert_eq!(file1.is_readonly(), true);

        // add_w
        assert!(sys::chmod_p(&file1).unwrap().add_w().chmod().is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100666);
        assert_eq!(file1.is_readonly(), false);

        // sub_r
        assert!(sys::chmod_p(&file1).unwrap().sub_r().chmod().is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100222);

        // add_r
        assert!(sys::chmod_p(&file1).unwrap().add_r().chmod().is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100666);

        // readonly
        assert!(sys::chmod_p(&file1).unwrap().readonly().chmod().is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100444);
        assert_eq!(file1.is_readonly(), true);

        // secure
        assert!(sys::chmod_p(&file1).unwrap().secure().chmod().is_ok());
        assert_eq!(file1.mode().unwrap(), 0o100400);
        assert_eq!(file1.is_readonly(), true);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_copy_empty() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_copy_empty");
        let file1 = tmpdir.mash("file1");

        // source doesn't exist
        assert!(sys::copy("", &file1).is_err());
        assert_eq!(file1.exists(), false);
    }

    #[test]
    fn test_copy_link_dir() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_copy_link_dir");
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
        let tmpdir = setup.temp.mash("file_copy_dir_copy");
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
        let tmpdir = setup.temp.mash("file_copy_dir_clone");
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
        let tmpdir = setup.temp.mash("file_copy_single_file");
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
        let tmpdir = setup.temp.mash("file_copyfile");
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
        let tmpdir = setup.temp.mash("file_copyfile_p");
        let file1 = tmpdir.mash("file1");
        let file2 = tmpdir.mash("file2");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // copy to same dir
        assert!(sys::touch_p(&file1, 0o644).is_ok());
        assert!(sys::copyfile_p(&file1, &file2).unwrap().mode(0o555).copy().is_ok());
        assert_eq!(file2.mode().unwrap(), 0o100555);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    #[cfg(feature = "_crypto_")]
    fn test_digest() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_digest");
        let file1 = tmpdir.mash("file1");
        let file2 = tmpdir.mash("file2");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        assert!(sys::write(&file1, "this is a test").is_ok());
        assert!(sys::copyfile(&file1, &file2).is_ok());
        assert_iter_eq(sys::digest(&file1).unwrap(), sys::digest(&file2).unwrap());

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_extract_string() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_extract_string");
        let file1 = tmpdir.mash("file1");
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // extract_string
        let rx = Regex::new(r"'([^']+)'\s+\((\d{4})\)").unwrap();
        assert!(sys::write(&file1, "Not my favorite movie: 'Citizen Kane' (1941).").is_ok());
        assert_eq!(sys::extract_string(&file1, &rx).unwrap(), "Citizen Kane");

        // extract_string_p
        assert_eq!(sys::extract_string_p(&file1, r"'([^']+)'\s+\((\d{4})\)").unwrap(), "Citizen Kane");

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_extract_strings() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_extract_strings");
        let file1 = tmpdir.mash("file1");
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // extract_string
        let rx = Regex::new(r"'([^']+)'\s+\((\d{4})\)").unwrap();
        assert!(sys::write(&file1, "Not my favorite movie: 'Citizen Kane' (1941).").is_ok());
        assert_eq!(sys::extract_strings(&file1, &rx).unwrap(), vec!["Citizen Kane", "1941"]);

        // extract_string_p
        assert_eq!(sys::extract_strings_p(&file1, r"'([^']+)'\s+\((\d{4})\)").unwrap(), vec!["Citizen Kane", "1941"]);

        // none
        let rx = Regex::new(r"(foo)").unwrap();
        assert!(sys::extract_strings(&file1, &rx).is_err());

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_mkdir_p() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_mkdir_p");
        let dir1 = tmpdir.mash("dir1");
        let dir2 = tmpdir.mash("dir2");

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
    fn test_move_p() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_move_p");
        let file1 = tmpdir.mash("file1");
        let file2 = tmpdir.mash("file2");
        let dir1 = tmpdir.mash("dir1");
        let dir2 = tmpdir.mash("dir2");
        let dir3 = tmpdir.mash("dir3");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&dir1).is_ok());

        // move file1 to file2 in the same dir
        assert!(sys::touch(&file1).is_ok());
        assert_eq!(file1.exists(), true);
        assert_eq!(file2.exists(), false);
        assert!(sys::move_p(&file1, &file2).is_ok());
        assert_eq!(file1.exists(), false);
        assert_eq!(file2.exists(), true);

        // move file2 into dir1
        assert!(sys::move_p(&file2, &dir1).is_ok());
        assert_eq!(file2.exists(), false);
        assert_eq!(dir1.mash("file2").exists(), true);

        // move dir1 to dir2
        assert!(sys::move_p(&dir1, &dir2).is_ok());
        assert_eq!(dir1.exists(), false);
        assert_eq!(dir2.exists(), true);
        assert_eq!(dir2.mash("file2").exists(), true);

        // move dir2 into dir3
        assert!(sys::mkdir(&dir3).is_ok());
        assert!(sys::move_p(&dir2, &dir3).is_ok());
        assert_eq!(dir1.exists(), false);
        assert_eq!(dir2.exists(), false);
        assert_eq!(dir3.exists(), true);
        assert_eq!(dir3.mash("dir2").exists(), true);
        assert_eq!(dir3.mash("dir2/file2").exists(), true);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_readbytes() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_readbytes");
        let tmpfile = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        assert!(sys::write(&tmpfile, "this is a test").is_ok());
        assert_eq!(str::from_utf8(&sys::readbytes(&tmpfile).unwrap()).unwrap(), "this is a test");

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_readlines() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_readlines");
        let tmpfile = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        assert!(sys::write(&tmpfile, "this is a test").is_ok());
        assert_iter_eq(sys::readlines(&tmpfile).unwrap(), vec![String::from("this is a test")]);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_readlines_p() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_readlines_p");
        let tmpfile = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        assert!(sys::write(&tmpfile, "this is a test").is_ok());
        assert_iter_eq(sys::readlines_p(&tmpfile).unwrap().collect::<io::Result<Vec<String>>>().unwrap(), vec![String::from("this is a test")]);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_readstring() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_readstring");
        let tmpfile = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        assert!(sys::write(&tmpfile, "this is a test").is_ok());
        assert_eq!(sys::readstring(&tmpfile).unwrap(), "this is a test");

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_remove() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_remove_dir");
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
        let tmpdir = setup.temp.mash("file_remove_all");

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
        let tmpdir = setup.temp.mash("file_symlink");
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
        let tmpdir = setup.temp.mash("file_touch");
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
        let tmpdir = setup.temp.mash("file_touch_p");
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

    #[test]
    fn test_write() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_write");
        let tmpfile = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        assert!(sys::write(&tmpfile, "this is a test").is_ok());
        assert_eq!(sys::readstring(&tmpfile).unwrap(), "this is a test");

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_write_p() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_write_p");
        let tmpfile = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        assert!(sys::write_p(&tmpfile, "this is a test", 0o666).is_ok());
        assert_eq!(sys::readstring(&tmpfile).unwrap(), "this is a test");
        assert_eq!(tmpfile.mode().unwrap(), 0o100666);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_writelines() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_writelines");
        let tmpfile = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        let lines = vec![String::from("one"), String::from("two")];
        assert!(sys::writelines(&tmpfile, &lines).is_ok());
        assert_iter_eq(sys::readlines(&tmpfile).unwrap(), lines);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_writelines_p() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("file_writelines_p");
        let tmpfile = tmpdir.mash("file1");

        // setup
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // test
        let lines = vec![String::from("one"), String::from("two")];
        assert!(sys::writelines_p(&tmpfile, &lines, 0o666).is_ok());
        assert_iter_eq(sys::readlines(&tmpfile).unwrap(), lines);
        assert_eq!(tmpfile.mode().unwrap(), 0o100666);

        // cleanup
        assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
