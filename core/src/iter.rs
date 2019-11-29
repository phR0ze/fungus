use std::iter::Iterator;

use crate::*;

// Iterator extensions and utilities
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
    /// If the iterator yields as single element, that element will be returned, otherwise
    /// an error will be returned.
    ///
    /// # Examples
    /// ```
    /// use rs::core::*;
    ///
    /// assert_eq!((0..10).filter(|&x| x == 2).single().unwrap(), 2);
    /// assert!((0..10).filter(|&x| x > 1 && x < 4).single().unwrap_err().eq(2..4));
    /// assert!((0..10).filter(|&x| x > 1 && x < 5).single().unwrap_err().eq(2..5));
    /// assert!((0..10).filter(|&_| false).single().unwrap_err().eq(0..0));
    /// ```
    fn single(mut self) -> Result<Self::Item>
    where
        Self: Sized,
    {
        match self.next() {
            Some(first) => match self.next() {
                Some(_) => Err(IterError::multiple_items_found()),
                None => Ok(first),
            },
            None => Err(IterError::item_not_found()),
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
        assert_eq!((0..10).filter(|&x| x == 2).single().unwrap(), 2);
        // assert!((0..10).filter(|&x| x > 2 && x < 4).single().unwrap_err().eq(2..4));
    }
}
