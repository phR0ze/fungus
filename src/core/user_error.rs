use std::fmt;
use std::error::Error as StdError;

// An error indicating that something went wrong with a user operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UserError {
    /// An error indicating that the use does not exist.
    DoesNotExistById(u32),
}
impl UserError {
    /// Return an error indicating that the user does not exist
    pub fn does_not_exist_by_id(uid: u32) -> UserError {
        UserError::DoesNotExistById(uid)
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn is<T: StdError + 'static>(&self) -> bool {
        <dyn StdError + 'static>::is::<T>(self)
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn downcast_ref<T: StdError + 'static>(&self) -> Option<&T> {
        <dyn StdError + 'static>::downcast_ref::<T>(self)
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn downcast_mut<T: StdError + 'static>(&mut self) -> Option<&mut T> {
        <dyn StdError + 'static>::downcast_mut::<T>(self)
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.as_ref().source()
    }
}

impl StdError for UserError{}

impl AsRef<dyn StdError> for UserError {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        self
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserError::DoesNotExistById(ref uid) => write!(f, "user does not exist: {}", uid),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::*;

    #[test]
    fn test_errors() {
        assert_eq!(UserError::does_not_exist_by_id(1000), UserError::DoesNotExistById(1000));
        assert_eq!(format!("{}", UserError::DoesNotExistById(1000)), "user does not exist: 1000");
    }
}
