cfgblock! {
    #[cfg(any(feature = "_net_", feature = "_arch_"))]
    use git2;
    const TMPDIR: &str = "git";
}

use crate::prelude::*;

/// Returns true if the remote `repo` `branch` exists.
/// Does not clone repo as is meant to be as lite as possible.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(net::git::remote_branch_exists("https://git.archlinux.org/svntogit/packages.git", "pkgfile"));
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn remote_branch_exists<T: AsRef<str>, U: AsRef<str>>(remote: T, branch: U) -> bool {
    match remote_branch_exists_err(remote, branch) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Returns Ok(()) if the remote `repo` `branch` exists else an Error.
/// Does not clone repo as is meant to be as lite as possible.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(net::git::remote_branch_exists_err("https://git.archlinux.org/svntogit/packages.git", "pkgfile").is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn remote_branch_exists_err<T: AsRef<str>, U: AsRef<str>>(remote: T, branch: U) -> Result<()> {
    // Create the temp dir and set a finally to clean it up
    let tmpdir = user::temp_dir(TMPDIR)?;
    let _f = finally(|| sys::remove_all(&tmpdir).unwrap());

    // Create the bare temp repo in the tmpdir with our target remote
    let repo = git2::Repository::init_bare(&tmpdir)?;
    let branch = format!("{}/{}", remote_name(remote.as_ref())?, branch.as_ref());
    let mut remote = repo.remote(&remote_name(remote.as_ref())?, remote.as_ref())?;

    // Test for the remote branch
    let refspec = format!("+refs/heads/{0:}:refs/remotes/origin/{0:}", &branch);
    remote.fetch(&[&refspec], None, None)?;
    repo.find_reference("FETCH_HEAD")?;
    Ok(())
}

/// Get the remote name from the repo URL
pub fn remote_name<T: AsRef<str>>(repo: T) -> Result<String> {
    PathBuf::from(repo.as_ref()).name()
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(any(feature = "_net_", feature = "_arch_"))]
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_remote_branch_exists() {
        assert_eq!(net::git::remote_branch_exists("https://git.archlinux.org/svntogit/packages.git", "foobar"), false);
        assert_eq!(net::git::remote_branch_exists("https://git.archlinux.org/svntogit/packages.git", "pkgfile"), true);
    }

    #[test]
    fn test_remote_branch_exists_err() {
        assert!(net::git::remote_branch_exists_err("https://git.archlinux.org/svntogit/packages.git", "foobar").is_err());
        assert!(net::git::remote_branch_exists_err("https://git.archlinux.org/svntogit/packages.git", "pkgfile").is_ok());
    }

    #[test]
    fn test_remote_name() {
        assert_eq!(net::git::remote_name("https://git.archlinux.org/svntogit/").unwrap(), "svntogit".to_string());
        assert_eq!(net::git::remote_name("https://git.archlinux.org/svntogit/packages").unwrap(), "packages".to_string());
        assert_eq!(net::git::remote_name("https://git.archlinux.org/svntogit/packages.git").unwrap(), "packages".to_string());
    }
}
