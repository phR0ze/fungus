#[cfg(feature = "user")]
use libc;
#[cfg(feature = "user")]
use std::io;
#[cfg(feature = "user")]
use std::mem;
#[cfg(feature = "user")]
use std::ptr;

use std::env;
use std::path::PathBuf;

use crate::core::*;

/// User provides options for a specific user.
#[cfg(feature = "user")]
#[derive(Debug, Clone, Default)]
pub struct User {
    pub uid: u32,           // user id
    pub gid: u32,           // user group id
    pub name: String,       // user name
    pub home: PathBuf,      // user home
    pub shell: PathBuf,     // user shell
    pub ruid: u32,          // real user id behind sudo
    pub rgid: u32,          // real user group id behind sudo
    pub realname: String,   // real user name behind sudo
    pub realhome: PathBuf,  // real user home behind sudo
    pub realshell: PathBuf, // real user shell behind sudo
}

#[cfg(feature = "user")]
impl User {
    /// Returns true if the user is root
    pub fn is_root(&self) -> bool {
        self.uid == 0
    }
}

/// Get the current user
#[cfg(feature = "user")]
pub fn current() -> Result<User> {
    let user = lookup(unsafe { libc::getuid() })?;
    // if user.home.empty() {
    //     user.home = home()?;
    // }
    Ok(user)
}

/// Switches back to the original user under the sudo mask with no way to go back.
///
/// ### Examples
/// ```ignore
/// use fungus::user;
///
/// user::drop_sudo().unwrap();
/// ```
#[cfg(feature = "user")]
pub fn drop_sudo() -> Result<()> {
    match getuid() {
        0 => {
            let (ruid, rgid) = getrids(0, 0);
            switchuser(ruid, ruid, ruid, rgid, rgid, rgid)
        }
        _ => Ok(()),
    }
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

/// Returns the real IDs for the given user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("real user ids of the given user: {:?}", user::getrids(0, 0));
/// ```
#[cfg(feature = "user")]
pub fn getrids(uid: u32, gid: u32) -> (u32, u32) {
    match uid {
        0 => match (env::var("SUDO_UID"), env::var("SUDO_GID")) {
            (Ok(u), Ok(g)) => match (u.parse::<u32>(), g.parse::<u32>()) {
                (Ok(u), Ok(g)) => (u, g),
                _ => (uid, gid),
            },
            _ => (uid, gid),
        },
        _ => (uid, gid),
    }
}

/// Return true if the current user is the root user.
///
/// ### Examples
/// ```
/// use fungus::user;
///
/// println!("is the user root: {:?}", user::is_root());
/// ```
#[cfg(feature = "user")]
pub fn is_root() -> bool {
    getuid() == 0
}

/// Lookup a user by user id
///
/// ### Examples
/// ```ignore
/// use fungus::user;
///
/// println!("lookup the given user: {:?}", user::lookup(1000).unwrap);
/// ```
#[cfg(feature = "user")]
pub fn lookup(uid: u32) -> Result<User> {
    // Get the libc::passwd by user id
    let mut buf = vec![0; 2048];
    let mut res = ptr::null_mut::<libc::passwd>();
    let mut passwd = unsafe { mem::zeroed::<libc::passwd>() };
    unsafe {
        libc::getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut res);
    }
    if res.is_null() || res != &mut passwd {
        return Err(UserError::does_not_exist_by_id(uid).into());
    }

    // Convert libc::passwd object into a User object
    //----------------------------------------------------------------------------------------------
    let gid = passwd.pw_gid;

    // User name for the lookedup user. We always want this and it should always exist.
    let username = unsafe { crate::libc::to_string(passwd.pw_name)? };

    // Will almost always be a single 'x' as the passwd is in the shadow database
    //let userpwd = unsafe { crate::libc::to_string(passwd.pw_passwd)? };

    // User home directory e.g. '/home/<user>'. Might be a null pointer indicating the system default should be used
    let userhome = unsafe { crate::libc::to_string(passwd.pw_dir) }.unwrap_or_default();

    // User shell e.g. '/bin/bash'. Might be a null pointer indicating the system default should be used
    let usershell = unsafe { crate::libc::to_string(passwd.pw_shell) }.unwrap_or_default();

    // A string container user contextual information, possibly real name or phone number.
    //let usergecos = unsafe { crate::libc::to_string(passwd.pw_gecos)? };

    // Get the user's real ids as well if applicable
    let (ruid, rgid) = getrids(uid, gid);
    let realuser = if uid != ruid {
        lookup(ruid)?
    } else {
        User {
            uid: uid,
            gid: gid,
            name: username.to_string(),
            home: PathBuf::from(&userhome),
            shell: PathBuf::from(&usershell),
            ..Default::default()
        }
    };
    Ok(User {
        uid: uid,
        gid: gid,
        name: username.to_string(),
        home: PathBuf::from(&userhome),
        shell: PathBuf::from(&usershell),
        ruid: ruid,
        rgid: rgid,
        realname: realuser.name,
        realhome: realuser.home,
        realshell: realuser.shell,
    })
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
/// ```ignore
/// use fungus::user;
///
/// println!("current user name: {:?}", user::name().unwrap());
/// ```
#[cfg(feature = "user")]
pub fn name() -> Result<String> {
    Ok(lookup(unsafe { libc::getuid() })?.name)
}

/// Switches back to the original user under the sudo mask. Preserves the ability to raise sudo
/// again.
///
/// ### Examples
/// ```ignore
/// use fungus::user;
///
/// user::pause_sudo().unwrap();
/// ```
#[cfg(feature = "user")]
pub fn pause_sudo() -> Result<()> {
    match getuid() {
        0 => {
            let (ruid, rgid) = getrids(0, 0);
            switchuser(ruid, ruid, 0, rgid, rgid, 0)
        }
        _ => Ok(()),
    }
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

/// Switches back to sudo root. Returns and error if not allowed.
///
/// ### Examples
/// ```ignore
/// use fungus::user;
///
/// user:sudo().unwrap();
/// ```
#[cfg(feature = "user")]
pub fn sudo() -> Result<()> {
    switchuser(0, 0, 0, 0, 0, 0)
}

/// Switches to another use by setting the real, effective and saved user and group ids.
///
/// ### Examples
/// ```ignore
/// use fungus::user;
///
/// // Switch to user 1000 but preserve root priviledeges to switch again
/// user::switchuser(1000, 1000, 0, 1000, 1000, 0);
///
/// // Switch to user 1000 and drop root priviledges permanantely
/// user::switchuser(1000, 1000, 1000, 1000, 1000, 1000);
/// ```
#[cfg(feature = "user")]
pub fn switchuser(ruid: u32, euid: u32, suid: u32, rgid: u32, egid: u32, sgid: u32) -> Result<()> {
    // Best practice to drop the group first
    match unsafe { libc::setresgid(rgid, egid, sgid) } {
        0 => match unsafe { libc::setresuid(ruid, euid, suid) } {
            0 => Ok(()),
            _ => Err(io::Error::last_os_error().into()),
        },
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
