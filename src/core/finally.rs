/// Finally uses Rust's object destructor to defer the execution of a function until
/// the surrounding function returns.
pub struct Finally<T: FnOnce()> {
    func: Option<T>,
}

/// Execute the wrapped function when the finally object goes out of scope
impl<T: FnOnce()> Drop for Finally<T> {
    fn drop(&mut self) {
        if let Some(func) = self.func.take() {
            func()
        }
    }
}

/// Create a Finally object that will execute the given `func` when its destructor is called.
///
/// This provides a mechanism similar to Golang's `defer` that will trigger when the
/// surrounding function goes out of scope.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("core_finally_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
///
/// // Create a scope that will trigger finally's destructor
/// {
///     let _f = finally(|| sys::remove_all(&tmpdir).unwrap());
/// }
/// assert_eq!(tmpdir.exists(), false);
/// ```
pub fn finally<T: FnOnce()>(func: T) -> Finally<T> {
    Finally { func: Some(func) }
}

// #[cfg(test)]
// mod tests {
//     use crate::prelude::*;

//     // Test setup
//     fn setup<T: AsRef<Path>>(path: T) -> PathBuf {
//         let temp = PathBuf::from("tests/temp").abs().unwrap();
//         sys::mkdir(&temp).unwrap();
//         temp.mash(path.as_ref())
//     }

//     #[test]
//     fn test_finally() {
//         let tmpdir = setup("core_finally");
//         assert!(sys::mkdir(&tmpdir).is_ok());

//         // Create scope that will trigger finally destructor
//         {
//             let _f = finally(|| sys::remove_all(&tmpdir).unwrap());
//         }

//         assert_eq!(tmpdir.exists(), false);
//     }
// }
