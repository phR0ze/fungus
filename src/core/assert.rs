#![allow(unused_imports)]
use crate::function;
use gory::*;

pub const TEST_TEMP_DIR: &str = "tests/temp";

/// Create the test `setup` function to be called in tests to disable RUST_BACKTRACE
/// and create a directory to work in for testing that depends on modifying files on
/// disk. The intent is to provide a thread safe space from which to manipulate
/// files during a test.
///
/// `setup` accepts two arguments `root` and `func_name`. `root` and `func_name` are
/// joined as a path and treated as the directory path that will be created for
/// tests.
///
/// ### Examples
/// ```
/// use fungus::assert::*;
///
/// create_test_setup_func!();
/// setup("tests/temp", "create_test_setup_func");
/// assert_dir!("tests/temp/create_test_setup_func");
/// assert_remove_all!("tests/temp/create_test_setup_func");
/// ```
#[macro_export]
macro_rules! create_test_setup_func {
    () => {
        use std::sync::Once;
        static TEST_INIT: Once = Once::new();
        fn setup<T: AsRef<Path>, U: AsRef<Path>>(root: T, func_name: U) -> PathBuf {
            TEST_INIT.call_once(|| {
                std::env::set_var("RUST_BACKTRACE", "0");
            });
            let target = root.as_ref().to_owned().mash(func_name.as_ref());
            assert!(sys::remove_all(&target).is_ok());
            assert!(sys::mkdir(&target).is_ok());
            target
        }
    };
}

/// Call the `setup` function created by `create_test_setup_func!` with default
/// `root` and `func_name` or optionally override those values. `root` will
/// default to `TEST_TEMP_DIR` and `func_name` defaults to the function name
/// using `function!`. However since doc tests always have a default function
/// name of `rust_out::main` its useful to override the function name in those
/// cases.
///
/// ### Examples
/// ```
/// use fungus::assert::*;
/// create_test_setup_func!();
///
/// // Defaults
/// fn assert_setup_default() {
///     let tmpdir = assert_setup!();
///     assert_dir!(&tmpdir);
///     assert_eq!(&tmpdir, &PathBuf::from(TEST_TEMP_DIR).mash("assert_setup_default"));
///     assert_remove_all!(&tmpdir);
/// }
/// assert_setup_default();
///
/// // Alternate function name
/// let func_name = "assert_setup_func";
/// let tmpdir = assert_setup!(&func_name);
/// assert_dir!(&tmpdir);
/// assert_eq!(&tmpdir, &PathBuf::from(TEST_TEMP_DIR).mash(&func_name));
/// assert_remove_all!(&tmpdir);
///
/// // Alternate root and function name
/// let root = "tests/temp/assert_setup_root";
/// let func_name = "assert_setup_func";
/// let tmpdir = assert_setup!(&root, &func_name);
/// assert_dir!(&tmpdir);
/// assert_eq!(&tmpdir, &PathBuf::from(&root).mash(&func_name));
/// assert_remove_all!(&root);
/// ```
#[macro_export]
macro_rules! assert_setup {
    () => {
        setup(TEST_TEMP_DIR, function!())
    };
    ($func:expr) => {
        setup(TEST_TEMP_DIR, $func)
    };
    ($root:expr, $func:expr) => {
        setup($root, $func)
    };
}

/// Assert that a directory exists
///
/// ### Examples
/// ```
/// use fungus::assert::*;
/// create_test_setup_func!();
///
/// let tmpdir = assert_setup!("assert_dir");
/// assert_dir!(&tmpdir);
/// assert_remove_all!(&tmpdir);
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
/// ```
/// use fungus::assert::*;
///
/// let tmpdir = PathBuf::from(TEST_TEMP_DIR).mash("assert_no_dir");
/// assert_no_dir!(&tmpdir);
/// ```
#[macro_export]
macro_rules! assert_no_dir {
    ($path:expr) => {
        if sys::is_dir($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_no_dir!".cyan(), format!("{:?}", $path).red(), "exists".yellow());
        }
    };
}

/// Assert that a file or directory exists
///
/// ### Examples
/// ```
/// use fungus::assert::*;
/// create_test_setup_func!();
///
/// let tmpdir = assert_setup!("assert_exists");
/// assert_exists!(&tmpdir);
/// let file = tmpdir.mash("file");
/// sys::touch(&file).unwrap();
/// assert_exists!(&file);
/// assert_remove_all!(&tmpdir);
/// ```
#[macro_export]
macro_rules! assert_exists {
    ($path:expr) => {
        if !sys::exists($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_exists!".cyan(), format!("{:?}", $path).red(), "doesn't exist".yellow());
        }
    };
}

/// Assert that a file or directory doesn't exists
///
/// ### Examples
/// ```
/// use fungus::assert::*;
///
/// assert_no_exists!("tests/temp/assert_no_exists");
/// assert_no_exists!("tests/temp/assert_no_exists/file");
/// ```
#[macro_export]
macro_rules! assert_no_exists {
    ($path:expr) => {
        if sys::exists($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_no_exists!".cyan(), format!("{:?}", $path).red(), "exists".yellow());
        }
    };
}

/// Assert that a file exists
///
/// ### Examples
/// ```
/// use fungus::assert::*;
/// create_test_setup_func!();
///
/// let tmpdir = assert_setup!("assert_file");
/// let file = tmpdir.mash("file");
/// sys::touch(&file).unwrap();
/// assert_file!(&file);
/// assert_remove_all!(&tmpdir);
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
/// ```
/// use fungus::assert::*;
///
/// assert_no_file!("tests/temp/assert_no_file/file");
/// ```
#[macro_export]
macro_rules! assert_no_file {
    ($path:expr) => {
        if sys::is_file($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_no_file!".cyan(), format!("{:?}", $path).red(), "exists".yellow());
        }
    };
}

/// Wraps `sys::remove` in an assertion that the file no longer exists and
/// provides some nice output if something fails.
///
/// ### Examples
/// ```
/// use fungus::assert::*;
/// create_test_setup_func!();
///
/// let tmpdir = assert_setup!();
/// let file = tmpdir.mash("file");
/// sys::touch(&file).unwrap();
/// assert_file!(&file);
/// assert_remove!(&file);
/// assert_no_file!(&file);
/// assert_remove_all!(&tmpdir);
/// ```
#[macro_export]
macro_rules! assert_remove {
    ($path:expr) => {
        if let Err(err) = sys::remove($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_remove!".cyan(), format!("{:?}", $path).red(), format!("{}", err).yellow());
        }
        if sys::exists($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_remove!".cyan(), format!("{:?}", $path).red(), "still exists".yellow());
        }
    };
}

/// Wraps `sys::remove_all` in an assertion that the directory no longer exists
/// and provides some nice output if something fails.
///
/// ### Examples
/// ```
/// use fungus::assert::*;
/// create_test_setup_func!();
///
/// let tmpdir = assert_setup!();
/// assert_exists!(&tmpdir);
/// assert_remove_all!(&tmpdir);
/// assert_no_exists!(&tmpdir);
/// ```
#[macro_export]
macro_rules! assert_remove_all {
    ($path:expr) => {
        if let Err(err) = sys::remove_all($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_remove_all!".cyan(), format!("{:?}", $path).red(), format!("{}", err).yellow());
        }
        if sys::exists($path) {
            panic!("\n{}:\n  {} => {}\n", "assert_remove_all!".cyan(), format!("{:?}", $path).red(), "still exists".yellow());
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
        // Defaults
        {
            let tmpdir = assert_setup!();
            assert_dir!(&tmpdir);
            assert_eq!(&tmpdir, &PathBuf::from(TEST_TEMP_DIR).mash("test_assert_setup"));
            assert_remove_all!(&tmpdir);
        }

        // Alternate func name
        {
            let func_name = "test_assert_setup_alt_func";
            let tmpdir = assert_setup!(&func_name);
            assert_dir!(&tmpdir);
            assert_eq!(&tmpdir, &PathBuf::from(TEST_TEMP_DIR).mash(&func_name));
            assert_remove_all!(&tmpdir);
        }

        // Alternate temp dir name and func name
        {
            let root = "tests/temp/test_assert_setup_dir";
            let func_name = "test_assert_setup_alt_func";
            let tmpdir = assert_setup!(&root, &func_name);
            assert_dir!(&tmpdir);
            assert_eq!(&tmpdir, &PathBuf::from(&root).mash(&func_name));
            assert_remove_all!(&root);
        }
    }

    #[test]
    fn test_assert_dir() {
        let tmpdir = assert_setup!();

        let temp_dir = tmpdir.mash("foo");
        assert_no_dir!(&temp_dir);
        assert!(!sys::is_dir(&temp_dir));
        sys::mkdir(&temp_dir).unwrap();
        assert_dir!(&temp_dir);
        assert!(sys::is_dir(&temp_dir));

        assert_remove_all!(&tmpdir);
    }

    #[test]
    fn test_assert_file() {
        let tmpdir = assert_setup!();

        let temp_file = tmpdir.mash("foo");
        assert_no_file!(&temp_file);
        assert!(!sys::is_file(&temp_file));
        sys::touch(&temp_file).unwrap();
        assert_file!(&temp_file);
        assert!(sys::is_file(&temp_file));

        assert_remove_all!(&tmpdir);
    }

    #[test]
    fn test_assert_exists() {
        let tmpdir = assert_setup!();

        // Test file exists
        {
            let file = tmpdir.mash("file");
            assert_no_exists!(&file);
            assert!(!sys::exists(&file));
            sys::touch(&file).unwrap();
            assert_exists!(&file);
            assert!(sys::exists(&file));

            sys::remove(&file).unwrap();
            assert_no_exists!(&file);
            assert!(!sys::exists(&file));
        }

        // Test dir exists
        {
            let dir = tmpdir.mash("dir");
            assert_no_exists!(&dir);
            assert!(!sys::exists(&dir));
            sys::mkdir(&dir).unwrap();
            assert_exists!(&dir);
            assert!(sys::exists(&dir));

            sys::remove_all(&dir).unwrap();
            assert_no_exists!(&dir);
            assert!(!sys::exists(&dir));
        }

        assert_remove_all!(&tmpdir);
    }

    #[test]
    fn test_assert_remove() {
        let tmpdir = assert_setup!();

        let file = tmpdir.mash("foo");
        sys::touch(&file).unwrap();
        assert_file!(&file);
        assert_remove!(&file);
        assert_no_file!(&file);

        assert_remove_all!(&tmpdir);
    }

    #[test]
    fn test_assert_remove_all() {
        let tmpdir = assert_setup!();

        let file = tmpdir.mash("foo");
        sys::touch(&file).unwrap();
        assert_file!(&file);
        assert_remove!(&file);
        assert_no_file!(&file);
        assert_dir!(&tmpdir);
        assert_remove_all!(&tmpdir);
        assert_no_dir!(&tmpdir);
    }
}
