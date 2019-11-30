use std::fmt;

use crate::error::*;

/// An error indicating something went wrong with an iterator operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IterError {
    /// An error indicating that the iterator item was not found
    ItemNotFound,

    /// An error indicating that multiple items were found for the iterator
    MultipleItemsFound,
}
impl IterError {
    /// An error indicating that the iterator item was not found
    pub fn item_not_found() -> Error {
        Error::from(IterError::ItemNotFound)
    }

    /// An error indicating that multiple items were found for the iterator
    pub fn multiple_items_found() -> Error {
        Error::from(IterError::MultipleItemsFound)
    }
}

impl fmt::Display for IterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IterError::ItemNotFound => write!(f, "iterator item not found"),
            IterError::MultipleItemsFound => write!(f, "multiple iterator items found"),
        }
    }
}

impl From<IterError> for Error {
    fn from(err: IterError) -> Error {
        Error::from(ErrorKind::Iter(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use failure::Fail;

    // when the iter error gets converted to a Result<i32, Error> it will get the failure goodness
    fn iter_error_result() -> Result<i32> {
        Err(IterError::item_not_found())
    }

    // #[test]
    // fn test_backtrace() {
    //     let err = iter_error_result().unwrap_err();
    //     println!("{:?}", err)
    // }

    #[test]
    fn test_iter_error_display() {
        assert_eq!("iterator item not found", format!("{}", IterError::item_not_found().kind()));
    }

    #[test]
    fn test_matching_error() {
        if let ErrorKind::Iter(err) = iter_error_result().unwrap_err().kind() {
            assert_eq!(&IterError::ItemNotFound, err);
        } else {
            panic!();
        }
    }
}
