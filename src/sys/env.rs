use crate::error::*;
use crate::sys::PathExt;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{env, io};

/// Returns the arguments that this program was started with (normally passed
/// via the command line).
/// Wraps std::env::args
///
/// The first element is traditionally the path of the executable, but it can be
/// set to arbitrary text, and may not even exist. This means this property should
/// not be relied upon for security purposes.
///
/// On Unix systems the shell usually expands unquoted arguments with glob patterns
/// (such as `*` and `?`). On Windows this is not done, and such arguments are
/// passed as-is.
///
/// On glibc Linux systems, arguments are retrieved by placing a function in `.init_array`.
/// Glibc passes `argc`, `argv`, and `envp` to functions in `.init_array`, as a non-standard
/// extension. This allows `std::env::args` to work even in a `cdylib` or `staticlib`, as it
/// does on macOS and Windows.
///
/// # Panics
///
/// The returned iterator will panic during iteration if any argument to the
/// process is not valid unicode. If this is not desired,
/// use the [`args_os`] function instead.
///
/// ### Examples
/// ```rust
/// use fungus::prelude::*;
///
/// // Prints each argument on a separate line
/// for argument in env::args() {
///     println!("{}", argument);
/// }
/// ```
pub fn args() -> env::Args {
    env::args()
}

/// Returns the current working directory as a [`PathBuf`].
/// Wraps std::env::current_dir
///
/// # Errors
///
/// Returns an [`Err`] if the current working directory value is invalid.
/// Possible cases:
///
/// * Current directory does not exist.
/// * There are insufficient permissions to access the current directory.
///
/// ### Examples
/// ```rust
/// use fungus::prelude::*;
///
/// println!("current working directory: {:?}", sys::env::cwd().unwrap());
/// ```
pub fn cwd() -> io::Result<PathBuf> {
    env::current_dir()
}

/// Returns the full filesystem path of the current running executable.
/// Wraps std::env::current_exec
///
/// ### Platform-specific behavior
///
/// If the executable was invoked through a symbolic link, some platforms will
/// return the path of the symbolic link and other platforms will return the
/// path of the symbolic link’s target.
///
/// # Errors
///
/// Acquiring the path of the current executable is a platform-specific operation
/// that can fail for a good number of reasons. Some errors can include, but not
/// be limited to, filesystem operations failing or general syscall failures.
///
/// On Linux systems, if this is compiled as `foo`:
///
/// ```bash
/// $ rustc foo.rs
/// $ ./foo
/// Ok("/home/alex/foo")
/// ```
///
/// And you make a hard link of the program:
///
/// ```bash
/// $ ln foo bar
/// ```
///
/// When you run it, you won’t get the path of the original executable, you’ll
/// get the path of the hard link:
///
/// ```bash
/// $ ./bar
/// Ok("/home/alex/bar")
/// ```
///
/// This sort of behavior has been known to [lead to privilege escalation] when
/// used incorrectly.
///
/// [lead to privilege escalation]: https://securityvulns.com/Wdocument183.html
///
/// ### Examples
/// ```rust
/// use fungus::prelude::*;
///
/// println!("current executable path: {:?}", sys::exe().unwrap());
/// ```
pub fn exe() -> io::Result<PathBuf> {
    env::current_exe()
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
/// assert!(!sys::env_flag("FOOBAR", false));
/// assert!(sys::env_flag("FOOBAR", true));
///
/// // Disabled variables will always be `false` despite default
/// std::env::set_var("FOOBAR", "0");
/// assert!(!sys::env_flag("FOOBAR", false));
/// assert!(!sys::env_flag("FOOBAR", true));
///
/// // Enabled variables will always be `true` despite default
/// sys::set_var("FOOBAR", "1");
/// assert!(sys::env_flag("FOOBAR", false));
/// assert!(sys::env_flag("FOOBAR", true));
/// ```
pub fn flag<K: AsRef<OsStr>>(key: K, default: bool) -> bool {
    !matches!(env::var(key).unwrap_or_else(|_| default.to_string()).to_lowercase().as_str(), "false" | "0")
}

/// Determine if the environment has an attached tty
///
/// ### Examples
/// ```rust
/// use fungus::prelude::*;
///
/// println!("{:?}", sys::hastty());
/// ```
pub fn hastty() -> bool {
    unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }
}

/// Changes the current working directory to the specified path.
/// Provides basic path expansion
///
/// Returns an [`Err`] if the operation fails.
///
/// ### Examples
/// ```rust,ignore
/// use fungus::prelude::*;
///
/// sys::set_cwd("~/").unwrap();
/// println!("current working directory: {:?}", sys::cwd().unwrap());
/// ```
pub fn set_cwd<P: AsRef<Path>>(path: P) -> Result<()> {
    let abs = path.as_ref().abs()?;
    Ok(env::set_current_dir(abs)?)
}

/// Sets the environment variable `k` to the value `v` for the currently running
/// process.
/// Wraps std::env::set_var
///
/// Note that while concurrent access to environment variables is safe in Rust,
/// some platforms only expose inherently unsafe non-threadsafe APIs for
/// inspecting the environment. As a result, extra care needs to be taken when
/// auditing calls to unsafe external FFI functions to ensure that any external
/// environment accesses are properly synchronized with accesses in Rust.
///
/// Discussion of this unsafety on Unix may be found in:
///
///  - [Austin Group Bugzilla](http://austingroupbugs.net/view.php?id=188)
///  - [GNU C library Bugzilla](https://sourceware.org/bugzilla/show_bug.cgi?id=15607#c2)
///
/// # Panics
///
/// This function may panic if `key` is empty, contains an ASCII equals sign
/// `'='` or the NUL character `'\0'`, or when the value contains the NUL
/// character.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// env::set_var("KEY", "VALUE");
/// assert_eq!(sys::var("KEY"), Ok("VALUE".to_string()));
/// ```
pub fn set_var<K: AsRef<OsStr>, V: AsRef<OsStr>>(k: K, v: V) {
    env::set_var(k, v)
}

/// Unset an environment variable from the environment of the currently running process.
/// Wraps std::env::remove_var
///
/// Note that while concurrent access to environment variables is safe in Rust,
/// some platforms only expose inherently unsafe non-threadsafe APIs for
/// inspecting the environment. As a result extra care needs to be taken when
/// auditing calls to unsafe external FFI functions to ensure that any external
/// environment accesses are properly synchronized with accesses in Rust.
///
/// Discussion of this unsafety on Unix may be found in:
///
///  - [Austin Group Bugzilla](http://austingroupbugs.net/view.php?id=188)
///  - [GNU C library Bugzilla](https://sourceware.org/bugzilla/show_bug.cgi?id=15607#c2)
///
/// # Panics
///
/// This function may panic if `key` is empty, contains an ASCII equals sign
/// `'='` or the NUL character `'\0'`, or when the value contains the NUL
/// character.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let key = "KEY";
/// sys::set_var(key, "VALUE");
/// assert_eq!(sys::var(key), Ok("VALUE".to_string()));
///
/// sys::unset_var(key);
/// assert!(sys::disabled(key));
/// ```
pub fn unset_var<K: AsRef<OsStr>>(k: K) {
    env::remove_var(k)
}

/// Fetches the environment variable `key` from the current process.
/// Wraps std::env::var
///
/// # Errors
/// * Environment variable is not present
/// * Environment variable is not valid unicode
///
/// # Panics
///
/// This function may panic if `key` is empty, contains an ASCII equals sign
/// `'='` or the NUL character `'\0'`, or when the value contains the NUL
/// character.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let key = "KEY";
/// sys::set_var(key, "VALUE");
/// assert_eq!(sys::var(key), Ok("VALUE".to_string()));
/// ```
pub fn var<K: AsRef<OsStr>>(key: K) -> std::result::Result<String, env::VarError> {
    env::var(key)
}

/// Returns an iterator of (variable, value) pairs of strings, for all the
/// environment variables of the current process.
/// Wraps std::env::vars
///
/// The returned iterator contains a snapshot of the process's environment
/// variables at the time of this invocation. Modifications to environment
/// variables afterwards will not be reflected in the returned iterator.
///
/// # Panics
///
/// While iterating, the returned iterator will panic if any key or value in the
/// environment is not valid unicode. If this is not desired, consider using
/// [`env::vars_os()`].
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// for (key, value) in env::vars() {
///     println!("{}: {}", key, value);
/// }
/// ```
pub fn vars() -> env::Vars {
    env::vars()
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_tty() {
        assert!(sys::hastty() || !sys::hastty());
    }

    #[test]
    fn test_flag() {
        sys::unset_var("FOOBAR");

        // Test unset case
        assert!(sys::flag("FOOBAR", true));
        assert!(!sys::flag("FOOBAR", false));

        // Test set to falsy
        sys::set_var("FOOBAR", "0");
        assert!(!sys::flag("FOOBAR", false));
        assert!(!sys::flag("FOOBAR", true));

        sys::set_var("FOOBAR", "false");
        assert!(!sys::flag("FOOBAR", false));
        assert!(!sys::flag("FOOBAR", true));

        sys::set_var("FOOBAR", "False");
        assert!(!sys::flag("FOOBAR", false));
        assert!(!sys::flag("FOOBAR", true));

        // Test set to truthy
        sys::set_var("FOOBAR", "true");
        assert!(sys::flag("FOOBAR", false));
        assert!(sys::flag("FOOBAR", true));

        sys::set_var("FOOBAR", "True");
        assert!(sys::flag("FOOBAR", false));
        assert!(sys::flag("FOOBAR", true));

        sys::set_var("FOOBAR", "blah");
        assert!(sys::flag("FOOBAR", false));
        assert!(sys::flag("FOOBAR", true));
    }
}
