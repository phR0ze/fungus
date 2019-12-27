#[cfg(feature = "user")]
use libc;
use std::io;

use std::env;
use std::path::PathBuf;

use crate::core::*;

/// User provides options for a specific user.
#[cfg(feature = "user")]
#[derive(Debug, Clone)]
pub struct User {
    uid: u32, // user id
    gid: u32, // user group id
}

/// Get the current user
/// TODO
#[cfg(feature = "user")]
pub fn current() -> User {
    panic!("Not implemented");
}

/// Lookup a user by user id
/// TODO
#[cfg(feature = "user")]
pub fn lookup(uid: u32) -> User {
    panic!("Not implemented");
}

/// Lookup a user by user name
/// TODO
#[cfg(feature = "user")]
pub fn lookup_by_name<T: AsRef<str>>(name: T) -> User {
    panic!("Not implemented");
}

/// Returns the full path to the current user's home directory.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("home directory of the current user: {:?}", user::home().unwrap());
/// ```
pub fn home() -> Result<PathBuf> {
    let os_str = env::var("HOME")?;
    let dir = PathBuf::from(os_str);
    Ok(dir)
}

/// Returns the user ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("user id of the current user: {:?}", user::getuid());
/// ```
#[cfg(feature = "user")]
pub fn getuid() -> u32 {
    unsafe { libc::getuid() }
}

/// Returns the user effective ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("user effective id of the current user: {:?}", user::geteuid());
/// ```
#[cfg(feature = "user")]
pub fn geteuid() -> u32 {
    unsafe { libc::geteuid() }
}

/// Returns the group ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("group id of the current user: {:?}", user::getgid());
/// ```
#[cfg(feature = "user")]
pub fn getgid() -> u32 {
    unsafe { libc::getgid() }
}

/// Returns the group effective ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("group effective id of the current user: {:?}", user::getegid());
/// ```
#[cfg(feature = "user")]
pub fn getegid() -> u32 {
    unsafe { libc::getegid() }
}

/// Return true if the current user is the root user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// user::is_root();
/// ```
#[cfg(feature = "user")]
pub fn is_root() -> bool {
    getuid() == 0
}

/// Set the user ID for the current user.
///
/// ### Examples
/// ```ignore
/// use fungus::user;
///
/// user::setuid(1000);
/// ```
#[cfg(feature = "user")]
pub fn setuid(uid: u32) -> Result<()> {
    match unsafe { libc::setuid(uid) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error().into()),
    }
}

/// Set the user effective ID for the current user.
///
/// ### Examples
/// ```ignore
/// use fungus::user;
///
/// user::seteuid(1000);
/// ```
#[cfg(feature = "user")]
pub fn seteuid(euid: u32) -> Result<()> {
    match unsafe { libc::seteuid(euid) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error().into()),
    }
}

/// Set the group ID for the current user.
///
/// ### Examples
/// ```ignore
/// use fungus::user;
///
/// user::setgid(1000);
/// ```
#[cfg(feature = "user")]
pub fn setgid(gid: u32) -> Result<()> {
    match unsafe { libc::setgid(gid) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error().into()),
    }
}

/// Set the group effective ID for the current user.
///
/// ### Examples
/// ```ignore
/// use fungus::user;
///
/// user::setegid(1000);
/// ```
#[cfg(feature = "user")]
pub fn setegid(egid: u32) -> Result<()> {
    match unsafe { libc::setegid(egid) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error().into()),
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::presys::*;
    use crate::user;

    #[test]
    fn test_user_home() {
        let home_str = env::var("HOME").unwrap();
        let home_path = PathBuf::from(home_str);
        let home_dir = home_path.parent().unwrap();
        assert_eq!(home_dir.to_path_buf(), user::home().unwrap().dir().unwrap());
    }
}
