use crate::errors::*;
use std::{ffi::OsStr, path::Path, str};

pub trait StringExt {
    /// Returns the length in characters rather than bytes i.e. this is a human understandable
    /// value. However it is more costly to perform.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!("foo".size(), 3);
    /// assert_eq!("ƒoo".len(), 4); // fancy f!
    /// assert_eq!("ƒoo".size(), 3); // fancy f!
    /// ```
    fn size(&self) -> usize;
}
impl StringExt for str {
    fn size(&self) -> usize {
        self.chars().count()
    }
}

impl StringExt for String {
    fn size(&self) -> usize {
        self.chars().count()
    }
}

pub trait ToStringExt {
    /// Returns a new [`String`] from the given type.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(OsStr::new("foo").to_string().unwrap(), "foo".to_string());
    /// assert_eq!(Path::new("/foo").to_string().unwrap(), "/foo".to_string());
    /// ```
    fn to_string(&self) -> FuResult<String>;
}

impl ToStringExt for Path {
    fn to_string(&self) -> FuResult<String> {
        let _str = self.to_str().ok_or_else(|| PathError::failed_to_string(self))?;
        Ok(String::from(_str))
    }
}

impl ToStringExt for OsStr {
    fn to_string(&self) -> FuResult<String> {
        Ok(String::from(self.to_str().ok_or(StringError::FailedToString)?))
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::{
        ffi::OsStr,
        path::{Path, PathBuf},
    };

    #[test]
    fn test_str_size() {
        assert_eq!("foo".size(), 3);
        assert_eq!("ƒoo".len(), 4); // fancy f!
        assert_eq!("ƒoo".size(), 3); // fancy f!
    }

    #[test]
    fn test_string_size() {
        assert_eq!("foo".to_string().size(), 3);
        assert_eq!("ƒoo".to_string().len(), 4); // fancy f!
        assert_eq!("ƒoo".to_string().size(), 3); // fancy f!
    }

    #[test]
    fn test_osstr_to_string() {
        assert_eq!(OsStr::new("foo").to_string().unwrap(), "foo".to_string());
    }

    #[test]
    fn test_path_to_string() {
        assert_eq!(Path::new("/foo").to_string().unwrap(), "/foo".to_string());
        assert_eq!(PathBuf::from("/foo").to_string().unwrap(), "/foo".to_string());
    }
}
