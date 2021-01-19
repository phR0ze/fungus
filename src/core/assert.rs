#![allow(unused_imports)]
use crate::function;
use gory::*;

pub const TEST_TEMP_DIR: &'static str = "tests/temp";

/// Create the test `setup` function to be called in tests to disable RUST_BACKTRACE
/// and create a directory to work in for a given test. This is very useful for
/// manipulating files in a thread safe space.
///
/// ### Examples
/// ```
/// use fungus::assert::*;
///
/// create_test_setup_func!();
/// ```
#[macro_export]
macro_rules! create_test_setup_func {
    () => {
        use std::sync::Once;
        static TEST_INIT: Once = Once::new();
        fn setup<T: AsRef<Path>, U: AsRef<Path>>(root: T, path: U) -> PathBuf {
            TEST_INIT.call_once(|| {
                std::env::set_var("RUST_BACKTRACE", "0");
            });
            let target = root.as_ref().to_owned().mash(path.as_ref());
            assert!(sys::remove_all(&target).is_ok());
            assert!(sys::mkdir(&target).is_ok());
            target
        }
    };
}

/// Call the `setup` function created by `create_test_setup_func!` using a
/// the test function name and a default `root` value `tests/temp` that can be
/// optionally overrode.
///
/// ### Examples
/// ```ignore
/// use fungus::assert::*;
///
/// create_test_setup_func!();
/// assert_setup!();
/// ```
#[macro_export]
macro_rules! assert_setup {
    () => {
        setup("tests/temp", function!())
    };
    ($root:expr) => {
        setup($root, function!())
    };
}

/// Assert that a directdory exists
///
/// ### Examples
/// ```ignore
/// use fungus::assert::*;
///
/// create_test_setup_func!();
/// let target = assert_setup!().mash("foo");
/// sys::mkdir(&target).unwrap();
/// assert_dir!(target);
/// ```
#[macro_export]
macro_rules! assert_dir {
    ($path:expr) => {
        if !sys::is_dir($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_dir!".cyan(), format!("{:?}", $path).red(), "doesn't exist or not a directory".yellow());
        }
    };
}

/// Assert that a directory doesn't exists
///
/// ### Examples
/// ```ignore
/// use fungus::assert::*;
///
/// create_test_setup_func!();
/// let target = assert_setup!().mash("foo");
/// assert_no_dir!(target);
/// ```
#[macro_export]
macro_rules! assert_no_dir {
    ($path:expr) => {
        if sys::is_dir($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_no_dir!".cyan(), format!("{:?}", $path).red(), "exists".yellow());
        }
    };
}

/// Assert that a file exists
///
/// ### Examples
/// ```ignore
/// use fungus::assert::*;
///
/// create_test_setup_func!();
/// let target = assert_setup!().mash("foo");
/// sys::touch(&target).unwrap();
/// assert_file!(target);
/// ```
#[macro_export]
macro_rules! assert_file {
    ($path:expr) => {
        if !sys::is_file($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_file!".cyan(), format!("{:?}", $path).red(), "doesn't exist or not a file".yellow());
        }
    };
}

/// Assert that a file doesn't exists
///
/// ### Examples
/// ```ignore
/// use fungus::assert::*;
///
/// create_test_setup_func!();
/// let target = assert_setup!().mash("foo");
/// assert_no_file!(&target);
/// ```
#[macro_export]
macro_rules! assert_no_file {
    ($path:expr) => {
        if sys::is_file($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_no_file!".cyan(), format!("{:?}", $path).red(), "exists".yellow());
        }
    };
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::assert::*;
    create_test_setup_func!();

    #[test]
    fn test_assert_setup() {
        // Default temp dir
        {
            let temp_dir = PathBuf::from(TEST_TEMP_DIR);
            let target = assert_setup!(&temp_dir);
            assert!(sys::is_dir(&target));
            assert_eq!(&target, &PathBuf::from("tests/temp/test_assert_setup"));
            assert!(sys::remove_all(&target).is_ok());
        }

        // Alternate temp dir
        {
            let temp_dir = PathBuf::from("tests/foo");
            let target = assert_setup!(&temp_dir);
            assert!(sys::is_dir(&target));
            assert_eq!(&target, &PathBuf::from("tests/foo/test_assert_setup"));
            assert!(sys::remove_all(&temp_dir).is_ok());
        }
    }

    #[test]
    fn test_assert_dir() {
        let target = assert_setup!();
        let temp_dir = target.mash("foo");
        assert_no_dir!(&temp_dir);
        assert!(!sys::is_dir(&temp_dir));
        sys::mkdir(&temp_dir).unwrap();
        assert_dir!(&temp_dir);
        assert!(sys::is_dir(&temp_dir));
    }

    #[test]
    fn test_assert_file() {
        let target = assert_setup!();
        let temp_file = target.mash("foo");
        assert_no_file!(&temp_file);
        assert!(!sys::is_file(&temp_file));
        sys::touch(&temp_file).unwrap();
        assert_file!(&temp_file);
        assert!(sys::is_file(&temp_file));
    }
}
