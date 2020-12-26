/// Ensure the given closure is executed once the surrounding scope closes despite panics.
/// Inspired by Golang's `defer`, Java's finally and Ruby's `ensure`.
///
/// This provides a mechanism similar to Golang's `defer` that will trigger when the
/// surrounding function goes out of scope.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("core_defer_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
///
/// // Create a scope that will trigger defer's destructor
/// {
///     let _defer = defer(|| sys::remove_all(&tmpdir).unwrap());
/// }
/// assert_eq!(tmpdir.exists(), false);
/// ```
pub fn defer<T: FnMut()>(f: T) -> impl Drop {
    Defer(f)
}

pub struct Defer<T: FnMut()>(T);

impl<T: FnMut()> Drop for Defer<T> {
    fn drop(&mut self) {
        (self.0)();
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::cell::Cell;
    use std::panic::{self, catch_unwind, AssertUnwindSafe};

    // Registers a panic hook that does nothing to supress the panic output
    // that get dumped to the screen regardless of panic handling with catch_unwind
    fn supress_panic_err() {
        panic::set_hook(Box::new(|_| {}));
    }

    #[test]
    fn test_defer_fires_even_with_panic() {
        supress_panic_err();

        let obj = Cell::new(1);
        let _ = catch_unwind(AssertUnwindSafe(|| {
            defer!(obj.set(2));
            panic!();
        }));
        assert_eq!(obj.get(), 2);
    }

    #[test]
    fn test_defer_actually_waits_until_end() {
        let obj = Cell::new(1);
        let _defer = defer(|| obj.set(2));
        assert_eq!(1, obj.get());
    }
}
