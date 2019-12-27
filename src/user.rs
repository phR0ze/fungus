#[cfg(feature = "user")]
use libc;
#[cfg(feature = "user")]
use std::ffi::{CStr, CString, OsStr, OsString};
#[cfg(feature = "user")]
use std::io;
#[cfg(feature = "user")]
use std::mem;
#[cfg(feature = "user")]
use std::os::unix::ffi::OsStrExt;
#[cfg(feature = "user")]
use std::ptr;

use std::env;
use std::path::PathBuf;

use crate::core::*;

/// User provides options for a specific user.
#[cfg(feature = "user")]
#[derive(Debug, Clone)]
pub struct User {
    uid: u32,  // user id
    gid: u32,  // user group id
    euid: u32, // effective user id
    egid: u32, // effective user group id
    ruid: u32, // real user id behind sudo
    rgid: u32, // real user group id behind sudo
}

#[cfg(feature = "user")]
impl User {
    /// Get the user's id
    pub fn uid(&self) -> u32 {
        self.uid
    }

    /// Get the user's group id
    pub fn gid(&self) -> u32 {
        self.gid
    }

    /// Get the user's effective id
    pub fn euid(&self) -> u32 {
        self.euid
    }

    /// Get the user's effective group id
    pub fn egid(&self) -> u32 {
        self.egid
    }

    /// Get the user's real id
    pub fn ruid(&self) -> u32 {
        self.ruid
    }

    /// Get the user's real group id
    pub fn rgid(&self) -> u32 {
        self.rgid
    }

    /// Returns true if the user is root
    pub fn is_root(&self) -> bool {
        self.uid == 0
    }
}

/// Get the current user
#[cfg(feature = "user")]
pub fn current() -> User {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    let (ruid, rgid) = realids(uid, gid);
    User { uid: uid, gid: gid, euid: unsafe { libc::geteuid() }, egid: unsafe { libc::getegid() }, ruid: ruid, rgid: rgid }
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

/// Returns the effective user ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("effective user id of the current user: {:?}", user::geteuid());
/// ```
#[cfg(feature = "user")]
pub fn geteuid() -> u32 {
    unsafe { libc::geteuid() }
}

/// Returns the effective group ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("effective group id of the current user: {:?}", user::getegid());
/// ```
#[cfg(feature = "user")]
pub fn getegid() -> u32 {
    unsafe { libc::getegid() }
}

/// Returns the real user ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("real user id of the current user: {:?}", user::getruid());
/// ```
#[cfg(feature = "user")]
pub fn getruid() -> u32 {
    if is_root() {
        match env::var("SUDO_UID") {
            Ok(uid) => uid.parse::<u32>().unwrap(),
            Err(_) => getuid(),
        }
    } else {
        getuid()
    }
}

/// Returns the real group ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("real group id of the current user: {:?}", user::getrgid());
/// ```
#[cfg(feature = "user")]
pub fn getrgid() -> u32 {
    if is_root() {
        match env::var("SUDO_GID") {
            Ok(gid) => gid.parse::<u32>().unwrap(),
            Err(_) => getgid(),
        }
    } else {
        getgid()
    }
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

/// Lookup a user by user id
#[cfg(feature = "user")]
pub fn lookup(uid: u32) -> Result<User> {
    // Get the libc::passwd by user id
    let mut buf = Vec::with_capacity(1024);
    let mut passwd = unsafe { mem::zeroed::<libc::passwd>() };
    let mut res = ptr::null_mut::<libc::passwd>();
    unsafe {
        libc::getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut res);
    }
    if res.is_null() || res != &mut passwd {
        return Err(UserError::does_not_exist_by_id(uid).into());
    }

    // Create a user object from the libc::passwd object
    let name = String::from(unsafe {
        OsStr::from_bytes(CStr::from_ptr(passwd.pw_name).to_bytes()).to_os_string().to_str().ok_or_else(|| UserError::failed_to_string(uid))?
    });

    let (ruid, rgid) = realids(uid, passwd.pw_gid);
    Ok(User { uid: uid, gid: passwd.pw_gid, euid: unsafe { libc::geteuid() }, egid: unsafe { libc::getegid() }, ruid: ruid, rgid: rgid })
}

/// Lookup a user by user name
/// TODO
#[cfg(feature = "user")]
pub fn lookup_by_name<T: AsRef<str>>(name: T) -> User {
    panic!("Not implemented");
}

/// Returns the current user's name.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("current user name: {:?}", user::name().unwrap());
/// ```
#[cfg(feature = "user")]
pub fn name() -> Result<String> {
    panic!("Not implemented");
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

// Private helper function to get the real ids for the given user
fn realids(uid: u32, gid: u32) -> (u32, u32) {
    if uid == 0 {
        match (env::var("SUDO_UID"), env::var("SUDO_GID")) {
            (Ok(suid), Ok(sgid)) => (suid.parse::<u32>().unwrap(), sgid.parse::<u32>().unwrap()),
            _ => (uid, gid),
        }
    } else {
        (uid, gid)
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
