use std::io;
use std::iter::Iterator;

use crate::*;

// Extensions for the Iterator trait and other iterator utilites
//--------------------------------------------------------------------------------------------------

pub trait IteratorExt: Iterator {
    fn single(mut self) -> Result<Self::Item>
    where
        Self: Sized;
}
impl<T: ?Sized> IteratorExt for T
where
    T: Iterator,
{
    fn single(mut self) -> Result<Self::Item>
    where
        Self: Sized,
    {
        match self.next() {
            Some(first) => match self.next() {
                Some(_) => Err(Error::new()),
                None => Ok(first),
            },
            None => Err(Error::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::path::{Component, PathBuf};

    #[test]
    fn test_first() {
        let path = PathBuf::from("/foo/bar");
        //assert!(path.components().first());
    }

    #[test]
    fn test_single() {
        assert!(PathBuf::new().components().single().is_err());
        //assert!(PathBuf::new().components().single().contains_err(Error::new()));
        assert_eq!(Component::Normal(OsStr::new("foo")), PathBuf::from("foo").components().single().unwrap());
    }
}
