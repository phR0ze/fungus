use crate::{errors::*, sys::PathExt};
use std::{fs::File, io::prelude::*, path::Path};

/// Returns true if the given `path` is a gzipped file
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("gzip_is_gzipped_doc");
/// let gzipped = tmpdir.mash("../../alpine-base.tgz");
/// assert_eq!(gzip::is_gzipped(gzipped).unwrap(), true);
/// ```
pub fn is_gzipped<T: AsRef<Path>>(path: T) -> FuResult<bool> {
    let path = path.as_ref().abs()?;

    // Read the first 2 bytes of the file
    let mut f = File::open(&path)?;
    let mut buffer = [0; 2];
    f.read_exact(&mut buffer)?;

    // Test against the gzip header signature 0x1f8b
    if buffer == [0x1f, 0x8b] || buffer == [0x8b, 0x1f] {
        return Ok(true);
    }
    Ok(false)
}

// Unit tests
// -------------------------------------------------------------------------------------------------
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
    fn test_is_gzipped() {
        let tmpdir = setup("is_gzipped");
        let tarball = tmpdir.mash("../../alpine-base.tar");
        let gzipped = tmpdir.mash("../../alpine-base.tgz");

        assert_eq!(gzip::is_gzipped(tarball).unwrap(), false);
        assert_eq!(gzip::is_gzipped(gzipped).unwrap(), true);
    }
}
