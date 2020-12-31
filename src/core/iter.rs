use crate::errors::*;
use std::{fmt, iter::Iterator};

// Iterator extensions and utilities
//--------------------------------------------------------------------------------------------------

/// Assert that the elements of the given iterables are equal and `Panics` when when not.
///
/// # Examples
/// ```
/// use fungus::core::*;
///
/// assert_iter_eq(vec![1, 2, 3].into_iter(), vec![1, 2, 3].into_iter());
/// ```
pub fn assert_iter_eq<T, U>(x: T, y: U)
where
    T: IntoIterator,
    U: IntoIterator,
    T::Item: fmt::Debug+PartialEq<U::Item>,
    U::Item: fmt::Debug,
{
    let mut x = x.into_iter();
    let mut y = y.into_iter();
    loop {
        match (x.next(), y.next()) {
            // Match done
            (None, None) => return,

            // Match items
            (a, b) => {
                let equal = match (&a, &b) {
                    // Compare the two items
                    (&Some(ref a), &Some(ref b)) => a == b,

                    // Different lengths
                    _ => false,
                };
                assert!(equal, "Iterators not equal {:?} != {:?}", a, b);
            },
        }
    }
}

/// Iterator adaptors to simplify some operations
pub trait IteratorExt: Iterator {
    /// Consume the entire iterator eagerly up until but not including the last call to
    /// get None. Allows caller to then call next and get None.
    ///
    /// # Examples
    /// ```
    /// use fungus::core::*;
    ///
    /// assert_eq!(vec![0, 1, 2].into_iter().consume().next(), None);
    /// ```
    fn consume(self) -> Self
    where
        Self: Sized;

    /// Drop the first `n` items if positive from the iterator eagerly and then return the
    /// iterator. Drop the last `|n|` items if negative from the iterator eagerly and then
    /// return the iterator.
    ///
    /// # Examples
    /// ```
    /// use fungus::core::*;
    ///
    /// assert_iter_eq(vec![1, 2, 3].into_iter().drop(1), vec![2, 3]);
    /// assert_iter_eq(vec![1, 2, 3].into_iter().drop(-1), vec![1, 2]);
    /// ```
    fn drop(self, n: isize) -> Self
    where
        Self: Sized,
        Self: DoubleEndedIterator;

    /// Returns the first element of the iterator. Alias to nth(0).
    ///
    /// `first()` will return [`None`] if `n` is greater than or equal to the length of the
    /// iterator.
    ///
    /// # Examples
    /// ```
    /// use fungus::core::*;
    ///
    /// assert_eq!((0..10).filter(|&x| x == 2).first().unwrap(), 2);
    /// ```
    fn first(self) -> Option<Self::Item>
    where
        Self: Sized;

    /// If the iterator yields at least one element, the first element will be returned,
    /// otherwise an error will be returned.
    ///
    /// # Examples
    /// ```
    /// use fungus::core::*;
    ///
    /// assert_eq!((0..10).filter(|&x| x == 2).first().unwrap(), 2);
    /// ```
    fn first_result(self) -> FuResult<Self::Item>
    where
        Self: Sized;

    /// If the iterator yields at least one element, the last element will be returned,
    /// otherwise an error will be returned.
    ///
    /// # Examples
    /// ```
    /// use fungus::core::*;
    ///
    /// assert_eq!((0..10).filter(|&x| x == 2).last().unwrap(), 2);
    /// ```
    fn last_result(self) -> FuResult<Self::Item>
    where
        Self: Sized;

    /// If the iterator yields a single element, that element will be returned, otherwise an
    /// error will be returned.
    ///
    /// # Examples
    /// ```
    /// use fungus::core::*;
    ///
    /// assert_eq!((0..10).filter(|&x| x == 2).single().unwrap(), 2);
    /// ```
    fn single(self) -> FuResult<Self::Item>
    where
        Self: Sized;

    /// Slice returns this iterator eagerly to only iterate over the range of elements called out
    /// by the given indices. Allows for negative notation.
    ///
    /// Note this operation uses count() to determine length which means cost O(n) out of the gate.
    ///
    /// # Examples
    /// ```
    /// use fungus::core::*;
    ///
    /// let mut iter = vec![0, 1, 2].into_iter().slice(0, 0);
    /// assert_eq!(iter.next(), Some(0));
    /// assert_eq!(iter.next(), None);
    ///
    /// let mut iter = vec![0, 1, 2].into_iter().slice(-1, -1);
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    ///
    /// let mut iter = vec![0, 1, 2].into_iter().slice(-2, -1);
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn slice(self, left: isize, right: isize) -> Self
    where
        Self: Sized,
        Self: Clone,
        Self: DoubleEndedIterator;

    /// If the iterator yields at least one element, true will be returned else false
    ///
    /// # Examples
    /// ```
    /// use fungus::core::*;
    ///
    /// assert_eq!((0..10).filter(|&x| x == 2).some(), true);
    /// ```
    fn some(self) -> bool
    where
        Self: Sized;
}

impl<T: ?Sized> IteratorExt for T
where
    T: Iterator,
{
    #[allow(clippy::all)]
    fn consume(mut self) -> Self
    where
        Self: Sized,
    {
        let mut iter = (&mut self).peekable();
        while let Some(_) = iter.next() {}
        self
    }

    fn drop(mut self, n: isize) -> Self
    where
        Self: Sized,
        Self: DoubleEndedIterator,
    {
        // Drop left
        if n > 0 {
            self.nth(n as usize - 1);
        }

        // Drop right
        if n < 0 {
            (&mut self).rev().nth(n.abs() as usize - 1);
        }
        self
    }

    fn first(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next()
    }

    fn first_result(mut self) -> FuResult<Self::Item>
    where
        Self: Sized,
    {
        match self.next() {
            Some(first) => Ok(first),
            None => Err(IterError::item_not_found().into()),
        }
    }

    fn last_result(self) -> FuResult<Self::Item>
    where
        Self: Sized,
    {
        match self.last() {
            Some(item) => Ok(item),
            None => Err(IterError::item_not_found().into()),
        }
    }

    fn single(mut self) -> FuResult<Self::Item>
    where
        Self: Sized,
    {
        match self.next() {
            Some(item) => match self.next() {
                Some(_) => Err(IterError::multiple_items_found().into()),
                None => Ok(item),
            },
            None => Err(IterError::item_not_found().into()),
        }
    }

    fn slice(mut self, left: isize, right: isize) -> Self
    where
        Self: Sized,
        Self: Clone,
        Self: DoubleEndedIterator,
    {
        // Convert left to postive notation and trim
        let (mut l, mut r): (usize, usize) = (left as usize, 0);
        let len = (self.clone()).count() as isize;
        if left < 0 {
            l = (len + left) as usize;
        }
        if l > 0 {
            self.nth(l - 1);
        }

        // Convert right to negative notation and trim.
        // Offset to have inclusive behavior.
        if right > 0 && right < len {
            r = (right - len + 1).abs() as usize;
        } else if right < 0 && right.abs() <= len {
            r = (right.abs() - 1).abs() as usize;
        } else if right < 0 {
            r = len as usize;
        }
        if r > 0 {
            (&mut self).rev().nth(r - 1);
        }

        // Get first or last
        if left == 0 && right == 0 {
            let i = len - 2;
            if i > 0 {
                (&mut self).rev().nth(i as usize);
            }
        }

        self
    }

    fn some(mut self) -> bool
    where
        Self: Sized,
    {
        self.next().is_some()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_slice() {
        // Both negative
        assert_eq!(vec![0, 1, 2, 3].into_iter().slice(-1, -5).next(), None); // right out of bounds negatively consumes all
        assert_iter_eq(vec![0, 1, 2, 3], vec![0, 1, 2, 3].into_iter().slice(-4, -1)); // get all
        assert_iter_eq(vec![0, 1, 2], vec![0, 1, 2, 3].into_iter().slice(-4, -2)); // get all but last
        assert_iter_eq(vec![0, 1], vec![0, 1, 2, 3].into_iter().slice(-4, -3)); // get all but last 2
        assert_iter_eq(vec![3], vec![0, 1, 2, 3].into_iter().slice(-1, -1)); // get last
        assert_iter_eq(vec![2], vec![0, 1, 2, 3].into_iter().slice(-2, -2)); // get index 2
        assert_iter_eq(vec![1], vec![0, 1, 2, 3].into_iter().slice(-3, -3)); // get index 1
        assert_iter_eq(vec![0], vec![0, 1, 2, 3].into_iter().slice(-4, -4)); // get first
        assert_iter_eq(vec![1, 2, 3], vec![0, 1, 2, 3].into_iter().slice(-3, -1)); // get all but first
        assert_iter_eq(vec![1, 2], vec![0, 1, 2, 3].into_iter().slice(-3, -2)); // get middle
        assert_eq!(vec![0, 1, 2, 3].into_iter().slice(-1, -2).next(), None); // mutually exclusive consumes everything

        // Both positive
        assert_iter_eq(vec![0, 1, 2, 3], vec![0, 1, 2, 3].into_iter().slice(0, 4)); // right out of bounds positively gets moved in
        assert_iter_eq(vec![0, 1, 2, 3], vec![0, 1, 2, 3].into_iter().slice(0, 3)); // get all
        assert_iter_eq(vec![0, 1, 2], vec![0, 1, 2, 3].into_iter().slice(0, 2)); // get all but last
        assert_iter_eq(vec![0, 1], vec![0, 1, 2, 3].into_iter().slice(0, 1)); // get all but last 2
        assert_iter_eq(vec![3], vec![0, 1, 2, 3].into_iter().slice(3, 3)); // get last
        assert_iter_eq(vec![2], vec![0, 1, 2, 3].into_iter().slice(2, 2)); // get index 2
        assert_iter_eq(vec![1], vec![0, 1, 2, 3].into_iter().slice(1, 1)); // get index 1
        assert_iter_eq(vec![0], vec![0, 1, 2, 3].into_iter().slice(0, 0)); // get first
        assert_iter_eq(vec![1, 2, 3], vec![0, 1, 2, 3].into_iter().slice(1, 3)); // get all but first
        assert_iter_eq(vec![1, 2], vec![0, 1, 2, 3].into_iter().slice(1, 2)); // get middle
        assert_eq!(vec![0, 1, 2, 3].into_iter().slice(3, 2).next(), None); // mutually exclusive consumes everything
        assert_eq!(vec![0, 1, 2, 3].into_iter().slice(4, 3).next(), None); // left out of bounds consumes everything

        // Left postive and right negative
        assert_eq!(vec![0, 1, 2, 3].into_iter().slice(0, -5).next(), None); // right out of bounds negatively consumes all
        assert_iter_eq(vec![0, 1, 2, 3], vec![0, 1, 2, 3].into_iter().slice(0, -1)); // get all
        assert_iter_eq(vec![0, 1, 2], vec![0, 1, 2, 3].into_iter().slice(0, -2)); // get all but last
        assert_iter_eq(vec![0, 1], vec![0, 1, 2, 3].into_iter().slice(0, -3)); // get all but last 2
        assert_iter_eq(vec![3], vec![0, 1, 2, 3].into_iter().slice(3, -1)); // get last
        assert_iter_eq(vec![2], vec![0, 1, 2, 3].into_iter().slice(2, -2)); // get index 2
        assert_iter_eq(vec![1], vec![0, 1, 2, 3].into_iter().slice(1, -3)); // get index 1
        assert_iter_eq(vec![0], vec![0, 1, 2, 3].into_iter().slice(0, -4)); // get first
        assert_iter_eq(vec![1, 2, 3], vec![0, 1, 2, 3].into_iter().slice(1, -1)); // get all but first
        assert_iter_eq(vec![1, 2], vec![0, 1, 2, 3].into_iter().slice(1, -2)); // get middle
        assert_eq!(vec![0, 1, 2, 3].into_iter().slice(3, -2).next(), None); // mutually exclusive consumes everything
        assert_eq!(vec![0, 1, 2, 3].into_iter().slice(4, -1).next(), None); // left out of bounds consumes everything
    }

    #[test]
    fn test_consume() {
        assert_eq!(vec![0].into_iter().nth(0), Some(0));
        assert_eq!(vec![0, 1, 2].into_iter().consume().nth(0), None);
    }

    #[test]
    fn test_drop() {
        // Start
        assert_iter_eq(vec![2, 3], vec![1, 2, 3].into_iter().drop(1));
        assert_iter_eq(PathBuf::from("bar").components(), PathBuf::from("foo/bar").components().drop(1));
        assert_iter_eq(PathBuf::from("bar").components(), PathBuf::from("/foo/bar").components().drop(2));

        // End
        assert_iter_eq(vec![1, 2], vec![1, 2, 3].into_iter().drop(-1));
        assert_eq!(1, vec![1, 2, 3].into_iter().drop(-1).next().unwrap());
        assert_iter_eq(PathBuf::from("foo").components(), PathBuf::from("foo/bar").components().drop(-1));
        assert_iter_eq(PathBuf::from("/").components(), PathBuf::from("/foo/bar").components().drop(-2));
    }

    #[test]
    fn test_eq() {
        assert_iter_eq(vec![1, 2], vec![1, 2]);
        assert!(std::panic::catch_unwind(|| assert_iter_eq(vec![1, 2], vec![1, 3])).is_err());
        assert_iter_eq(PathBuf::from("foo/bar").components(), PathBuf::from("foo/bar").components());
        assert!(std::panic::catch_unwind(|| assert_iter_eq(PathBuf::from("foo/bar").components(), PathBuf::from("bar").components())).is_err());
    }

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
        assert_eq!((0..10).filter(|&x| x > 2).single().unwrap_err().downcast_ref::<IterError>(), Some(&IterError::multiple_items_found()));
        assert_eq!((0..10).filter(|&x| x > 2 && x < 5).single().unwrap_err().downcast_ref::<IterError>(), Some(&IterError::multiple_items_found()));
    }

    #[test]
    fn test_some() {
        assert_eq!((0..10).filter(|&x| x == 2).some(), true);
        assert_eq!((0..10).filter(|&x| x == 11).some(), false);
    }
}
