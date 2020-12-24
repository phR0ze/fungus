use std::error::Error as StdError;
use std::fmt;

/// An error indicating something went wrong with an iterator operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IterError {
    /// An error indicating that the iterator item was not found
    ItemNotFound,

    /// An error indicating that multiple items were found for the iterator
    MultipleItemsFound,

    /// An error indicating that the indicies are mutually exclusive
    MutuallyExclusiveIndicies,
}
impl IterError {
    /// An error indicating that the iterator item was not found
    pub fn item_not_found() -> IterError {
        IterError::ItemNotFound
    }

    /// An error indicating that multiple items were found for the iterator
    pub fn multiple_items_found() -> IterError {
        IterError::MultipleItemsFound
    }

    /// An error indicating that the indicies are mutually exclusive
    pub fn mutually_exclusive_indices() -> IterError {
        IterError::MutuallyExclusiveIndicies
    }
}

impl StdError for IterError {}

impl fmt::Display for IterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IterError::ItemNotFound => write!(f, "iterator item not found"),
            IterError::MultipleItemsFound => write!(f, "multiple iterator items found"),
            IterError::MutuallyExclusiveIndicies => write!(f, "mutually exclusive indices"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    fn item_not_found() -> Result<i32> {
        Err(IterError::item_not_found())?
    }

    #[test]
    fn test_backtrace() {
        let err = item_not_found().unwrap_err();
        println!("{:?}", err)
    }

    #[test]
    fn test_item_not_found() {
        assert_eq!(item_not_found().unwrap_err().downcast_ref::<IterError>(), Some(&IterError::ItemNotFound));
        assert_eq!(format!("{}", IterError::item_not_found()), "iterator item not found");
    }

    #[test]
    fn test_multiple_items_found() {
        assert_eq!(format!("{}", IterError::multiple_items_found()), "multiple iterator items found");
    }

    #[test]
    fn test_mutually_exclusive_indices() {
        assert_eq!(format!("{}", IterError::mutually_exclusive_indices()), "mutually exclusive indices");
    }
}
