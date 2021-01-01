pub trait OptionExt<T> {
    fn has<U>(&self, value: U) -> bool
    where
        U: PartialEq<T>;
}

impl<T> OptionExt<T> for Option<T> {
    /// Returns `true` if the option is a [`Some`] value containing the given value.
    ///
    /// # Examples
    /// ```
    /// use fungus::core::*;
    ///
    /// let x: Option<u32> = Some(2);
    /// assert!(x.has(2));
    ///
    /// let x: Option<u32> = Some(3);
    /// assert!(!x.has(2));
    ///
    /// let x: Option<u32> = None;
    /// assert!(!x.has(2));
    /// ```
    fn has<U>(&self, x: U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            Some(y) => x == *y,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Component;

    #[test]
    fn test_has() {
        assert!(Some(Component::ParentDir).has(Component::ParentDir));
        assert_eq!(Some(Component::ParentDir).has(Component::ParentDir), true);
        assert_eq!(None.has(Component::ParentDir), false);
    }
}
