use std::iter::Iterator;

use crate::*;

// Iterator extensions and utilities
//--------------------------------------------------------------------------------------------------

pub trait IteratorExt: Iterator {
    fn first(self) -> Option<Self::Item>
    where
        Self: Sized;
    fn first_result(self) -> Result<Self::Item>
    where
        Self: Sized;
    fn last_result(self) -> Result<Self::Item>
    where
        Self: Sized;
    fn single(self) -> Result<Self::Item>
    where
        Self: Sized;
}
impl<T: ?Sized> IteratorExt for T
where
    T: Iterator,
{
    /// Returns the first element of the iterator. Alias to nth(0).
    ///
    /// `first()` will return [`None`] if `n` is greater than or equal to the length of the
    /// iterator.
    ///
    /// # Examples
    /// ```
    /// use rs::core::*;
    ///
    /// assert_eq!((0..10).filter(|&x| x == 2).first().unwrap(), 2);
    /// ```
    fn first(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.nth(0)
    }

    /// If the iterator yields at least one element, the first element will be returned,
    /// otherwise an error will be returned.
    ///
    /// # Examples
    /// ```
    /// use rs::core::*;
    ///
    /// assert_eq!((0..10).filter(|&x| x == 2).first().unwrap(), 2);
    /// ```
    fn first_result(mut self) -> Result<Self::Item>
    where
        Self: Sized,
    {
        match self.next() {
            Some(first) => Ok(first),
            None => Err(IterError::item_not_found()),
        }
    }

    /// If the iterator yields at least one element, the last element will be returned,
    /// otherwise an error will be returned.
    ///
    /// # Examples
    /// ```
    /// use rs::core::*;
    ///
    /// assert_eq!((0..10).filter(|&x| x == 2).last().unwrap(), 2);
    /// ```
    fn last_result(self) -> Result<Self::Item>
    where
        Self: Sized,
    {
        match self.last() {
            Some(item) => Ok(item),
            None => Err(IterError::item_not_found()),
        }
    }

    /// If the iterator yields as single element, that element will be returned, otherwise
    /// an error will be returned.
    ///
    /// # Examples
    /// ```
    /// use rs::core::*;
    ///
    /// assert_eq!((0..10).filter(|&x| x == 2).single().unwrap(), 2);
    /// ```
    fn single(mut self) -> Result<Self::Item>
    where
        Self: Sized,
    {
        match self.next() {
            Some(item) => match self.next() {
                Some(_) => Err(IterError::multiple_items_found()),
                None => Ok(item),
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
        assert_eq!(Component::Normal(OsStr::new("foo")), PathBuf::from("foo/bar").components().first().unwrap());
        assert_ne!(Component::Normal(OsStr::new("bar")), PathBuf::from("foo/bar").components().first().unwrap());
    }

    #[test]
    fn test_first_result() {
        assert_eq!(Component::Normal(OsStr::new("foo")), PathBuf::from("foo/bar").components().first_result().unwrap());
        assert_ne!(Component::Normal(OsStr::new("bar")), PathBuf::from("foo/bar").components().first_result().unwrap());
    }

    #[test]
    fn test_last_result() {
        assert_eq!(Component::Normal(OsStr::new("bar")), PathBuf::from("foo/bar").components().last_result().unwrap());
        assert_ne!(Component::Normal(OsStr::new("foo")), PathBuf::from("foo/bar").components().last_result().unwrap());
    }

    #[test]
    fn test_single() {
        assert_eq!((0..10).filter(|&x| x == 2).single().unwrap(), 2);
        assert!((0..10).filter(|&x| x > 2).single().unwrap_err().eq(&IterError::multiple_items_found()));

        if let ErrorKind::Iter(err) = (0..10).filter(|&x| x > 2 && x < 5).single().unwrap_err().kind() {
            assert_eq!(err, &IterError::MultipleItemsFound)
        } else {
            assert!(false)
        }
    }
}
