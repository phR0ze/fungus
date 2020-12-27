use crate::enc::gzip;
use crate::errors::*;
use crate::sys::{self, PathExt};
use std::fs::File;
use std::path::Path;

cfgblock! {
    #[cfg(feature = "_enc_")]
    use flate2::{self, Compression};
    use flate2::read::GzDecoder;
    use flate2::write::GzEncoder;
}

/// Create a tarball `tarfile` uing gzip compression from the files implicated by the `glob`.
/// Handles file globbing and recursively adds source files based on glob.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("tar_create_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let tarball = tmpdir.mash("tarball.tgz");
/// let dst = tmpdir.mash("dst");
/// let dstfile = dst.mash("file1");
/// assert!(sys::write(&file1, "single file\n").is_ok());
/// assert!(tar::create(&tarball, &file1).is_ok());
/// assert!(tar::extract_all(&tarball, &dst).is_ok());
/// assert_eq!(sys::readstring(&dstfile).unwrap(), "single file\n".to_string());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(feature = "_enc_")]
pub fn create<T: AsRef<Path>, U: AsRef<Path>>(tarfile: T, glob: U) -> FuResult<()> {
    let tarfile = tarfile.as_ref().abs()?;

    // Handle globbing
    let sources = sys::glob(glob.as_ref())?;
    if sources.is_empty() {
        return Err(PathError::does_not_exist(glob.as_ref()).into());
    }

    // Create the tarfile
    let tar_gz = File::create(&tarfile)?;
    let encoder = GzEncoder::new(tar_gz, Compression::default());
    let mut tarball = tar::Builder::new(encoder);

    // Include all source files in the tarball
    for source in sources {
        if source.is_file() {
            let mut f = File::open(&source)?;
            tarball.append_file(source.base()?, &mut f)?;
        } else {
            tarball.append_dir_all(source.base()?, &source)?;
        }
    }

    Ok(())
}

/// Extract all tarball files into the given `dst` directory.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("tar_extract_all_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let file1 = tmpdir.mash("file1");
/// let tarball = tmpdir.mash("tarball.tgz");
/// let dst = tmpdir.mash("dst");
/// let dstfile = dst.mash("file1");
/// assert!(sys::write(&file1, "single file\n").is_ok());
/// assert!(tar::create(&tarball, &file1).is_ok());
/// assert!(tar::extract_all(&tarball, &dst).is_ok());
/// assert_eq!(sys::readstring(&dstfile).unwrap(), "single file\n".to_string());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(feature = "_enc_")]
pub fn extract_all<T: AsRef<Path>, U: AsRef<Path>>(tarfile: T, dst: U) -> FuResult<()> {
    let dst = dst.as_ref().abs()?;
    let tarfile = tarfile.as_ref().abs()?;

    if gzip::is_gzipped(&tarfile)? {
        let f = File::open(&tarfile)?;
        let tar = GzDecoder::new(f);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(&dst)?;
    } else {
        let f = File::open(&tarfile)?;
        let mut archive = tar::Archive::new(f);
        archive.unpack(&dst)?;
    }

    Ok(())
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(feature = "_enc_")]
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // Test setup
    fn setup<T: AsRef<Path>>(path: T) -> PathBuf {
        let temp = PathBuf::from("tests/temp").abs().unwrap();
        sys::mkdir(&temp).unwrap();
        temp.mash(path.as_ref())
    }

    #[test]
    fn test_create_and_extract_multiple() {
        let tmpdir = setup("tar_create_and_extract_multile");
        let dir1 = tmpdir.mash("dir1");
        let file1 = dir1.mash("file1");
        let dir2 = tmpdir.mash("dir2");
        let file2 = dir2.mash("file2");
        let tarball = tmpdir.mash("tarball.tgz");
        let dst = tmpdir.mash("dst");
        let dstdir1 = dst.mash("dir1");
        let dstfile1 = dstdir1.mash("file1");
        let dstdir2 = dst.mash("dir2");
        let dstfile2 = dstdir2.mash("file2");
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // Create tarball
        assert!(sys::mkdir(&dir1).is_ok());
        assert!(sys::mkdir(&dir2).is_ok());
        assert!(sys::write(&file1, "single file1\n").is_ok());
        assert!(sys::write(&file2, "single file2\n").is_ok());
        assert!(tar::create(&tarball, tmpdir.mash("dir*")).is_ok());

        // Extract tarball
        assert!(tar::extract_all(&tarball, &dst).is_ok());
        assert_eq!(sys::readstring(&dstfile1).unwrap(), "single file1\n".to_string());
        assert_eq!(sys::readstring(&dstfile2).unwrap(), "single file2\n".to_string());

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_create_and_extract_multiple_files() {
        let tmpdir = setup("tar_create_and_extract_multile_files");
        let file1 = tmpdir.mash("file1");
        let file2 = tmpdir.mash("file2");
        let tarball = tmpdir.mash("tarball.tgz");
        let dst = tmpdir.mash("dst");
        let dstfile1 = dst.mash("file1");
        let dstfile2 = dst.mash("file2");
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // Create tarball
        assert!(sys::write(&file1, "single file1\n").is_ok());
        assert!(sys::write(&file2, "single file2\n").is_ok());
        assert!(tar::create(&tarball, tmpdir.mash("file*")).is_ok());

        // Extract tarball
        assert!(tar::extract_all(&tarball, &dst).is_ok());
        assert_eq!(sys::readstring(&dstfile1).unwrap(), "single file1\n".to_string());
        assert_eq!(sys::readstring(&dstfile2).unwrap(), "single file2\n".to_string());

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_create_and_extract_single_file() {
        let tmpdir = setup("tar_create_and_extract_single_file");
        let file1 = tmpdir.mash("file1");
        let tarball = tmpdir.mash("tarball.tgz");
        let dst = tmpdir.mash("dst");
        let dstfile = dst.mash("file1");
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(sys::mkdir(&tmpdir).is_ok());

        // Create tarball
        assert!(sys::write(&file1, "single file\n").is_ok());
        assert!(tar::create(&tarball, &file1).is_ok());

        // Extract tarball
        assert!(tar::extract_all(&tarball, &dst).is_ok());
        assert_eq!(sys::readstring(&dstfile).unwrap(), "single file\n".to_string());

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_extract_sys_tgz() {
        let tmpdir = setup("tar_extract_sys_tgz");
        let tarball = tmpdir.mash("../../alpine-base.tgz");
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(tar::extract_all(&tarball, &tmpdir).is_ok());
        assert_eq!(sys::readlines(tmpdir.mash("README.md")).unwrap()[0], "alpine-base".to_string());
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_extract_sys_tar() {
        let tmpdir = setup("tar_extract_sys_tar");
        let tarball = tmpdir.mash("../../alpine-base.tar");
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(tar::extract_all(&tarball, &tmpdir).is_ok());
        assert_eq!(sys::readlines(tmpdir.mash("README.md")).unwrap()[0], "alpine-base".to_string());
        assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
