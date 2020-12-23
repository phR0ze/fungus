use std::env;
use std::ffi::OsStr;

/// Determine if the environment has an attached tty
pub fn isatty() -> bool {
    unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }
}

/// Get the value of the given environment variable as a flag.
///
/// The flag will be considered `true` if the environment variable is set and the
/// value is any value other than `0` or a case insensitive version of `false`.
/// The flag will be considered `false` if the environment variable is unset or
/// it is set and the value is a `0` or a case insensitive version of `false`.
///
/// ### Examples
/// ```rust
/// use fungus::prelude::*;
///
/// // Unset variables will be default to the given value
/// assert!(!sys::env::flag("FOOBAR", false));
/// assert!(sys::env::flag("FOOBAR", true));
///
/// // Disabled variables will always be `false` despite default
/// std::env::set_var("FOOBAR", "0");
/// assert!(!sys::env::flag("FOOBAR", false));
/// assert!(!sys::env::flag("FOOBAR", true));
///
/// // Enabled variables will always be `true` despite default
/// std::env::set_var("FOOBAR", "1");
/// assert!(sys::env::flag("FOOBAR", false));
/// assert!(sys::env::flag("FOOBAR", true));
/// ```
pub fn flag<K: AsRef<OsStr>>(key: K, default: bool) -> bool {
    !matches!(env::var(key).unwrap_or_else(|_| default.to_string()).to_lowercase().as_str(), "false" | "0")
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tty() {
        assert!(isatty() || !isatty());
    }

    #[test]
    fn test_flag() {
        env::remove_var("FOOBAR");

        // Test unset case
        assert!(flag("FOOBAR", true));
        assert!(!flag("FOOBAR", false));

        // Test set to falsy
        env::set_var("FOOBAR", "0");
        assert!(!flag("FOOBAR", false));
        assert!(!flag("FOOBAR", true));

        env::set_var("FOOBAR", "false");
        assert!(!flag("FOOBAR", false));
        assert!(!flag("FOOBAR", true));

        env::set_var("FOOBAR", "False");
        assert!(!flag("FOOBAR", false));
        assert!(!flag("FOOBAR", true));

        // Test set to truthy
        env::set_var("FOOBAR", "true");
        assert!(flag("FOOBAR", false));
        assert!(flag("FOOBAR", true));

        env::set_var("FOOBAR", "True");
        assert!(flag("FOOBAR", false));
        assert!(flag("FOOBAR", true));

        env::set_var("FOOBAR", "blah");
        assert!(flag("FOOBAR", false));
        assert!(flag("FOOBAR", true));
    }
}
