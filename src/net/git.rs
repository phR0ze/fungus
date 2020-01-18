cfgblock! {
    #[cfg(any(feature = "_net_", feature = "_arch_"))]
    use git2::{self, FetchOptions, Progress, RemoteCallbacks};
    use git2::build::{CheckoutBuilder, RepoBuilder};
    const TMPDIR: &str = "git";
}

use crate::prelude::*;

/// Clone the repo locally. Clones the entire repo.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_clone_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("README.md");
/// assert_eq!(tmpfile.exists(), false);
/// assert!(git::clone("https://github.com/phR0ze/alpine-base.git", &tmpdir).is_ok());
/// assert_eq!(tmpfile.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn clone<T: AsRef<str>, U: AsRef<Path>>(url: T, dst: U) -> Result<PathBuf> {
    let mut builder = RepoBuilder::new();
    builder.clone(url.as_ref(), dst.as_ref())?;
    Ok(dst.as_ref().to_path_buf())
}

/// Clone the repo locally for the given branch only. Avoids cloning the entire repo
/// by targeting a specific remote fetch refspec.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_clone_branch_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("README.md");
/// assert_eq!(tmpfile.exists(), false);
/// assert!(git::clone_branch("https://github.com/phR0ze/alpine-base.git", "master", &tmpdir).is_ok());
/// assert_eq!(tmpfile.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn clone_branch<T: AsRef<str>, U: AsRef<str>, V: AsRef<Path>>(url: T, branch: U, dst: V) -> Result<PathBuf> {
    let mut builder = RepoBuilder::new();
    builder.branch(branch.as_ref());

    // Create the remote with the --single-branch refspec override. refspec is <src>:<dst> where
    // <src> is the pattern for referencing the remote side and <dst> is the local saved data
    // stored under the remotes path.
    //
    // Example:
    // $ git init -q test; cd test
    // $ git remote add origin https://github.com/phR0ze/alpine-base.git
    // $ git ls-remote origin
    // b61d09a78f8eaf5f0e505f03ac6301845d96d602	HEAD
    // b61d09a78f8eaf5f0e505f03ac6301845d96d602	refs/heads/master
    //
    // # Create a refspec to only download 'master' and store as 'refs/remotes/origin/master'
    // refspec="+refs/heads/master:refs/remotes/origin/master"
    builder.remote_create(|repo, name, url| {
        let refspec = format!("+refs/heads/{0:}:refs/remotes/origin/{0:}", branch.as_ref());
        repo.remote_with_fetch(name, url, &refspec)
    });

    // Clone the single branch from the repo if it exists
    builder.clone(url.as_ref(), dst.as_ref())?;

    Ok(dst.as_ref().to_path_buf())
}

/// Returns true if the remote `repo` `branch` exists.
/// Does not clone repo as is meant to be as lite as possible.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(git::remote_branch_exists("https://github.com/phR0ze/alpine-base.git", "master"));
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn remote_branch_exists<T: AsRef<str>, U: AsRef<str>>(url: T, branch: U) -> bool {
    match remote_branch_exists_err(url, branch) {
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
/// assert!(git::remote_branch_exists_err("https://github.com/phR0ze/alpine-base.git", "master").is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn remote_branch_exists_err<T: AsRef<str>, U: AsRef<str>>(url: T, branch: U) -> Result<()> {
    // Create the temp dir and set a finally to clean it up
    let tmpdir = user::temp_dir(TMPDIR)?;
    let _f = finally(|| sys::remove_all(&tmpdir).unwrap());

    // Create the bare temp repo in the tmpdir with our target remote
    let repo = git2::Repository::init_bare(&tmpdir)?;
    let mut remote = repo.remote("origin", url.as_ref())?;

    // Test for the remote branch
    let refspec = format!("+refs/heads/{0:}:refs/remotes/origin/{0:}", branch.as_ref());
    remote.fetch(&[&refspec], None, None)?;
    repo.find_reference("FETCH_HEAD")?;
    Ok(())
}

/// Clone the repo locally with progress callback. Clones the entire repo.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_clone_with_progress_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("README.md");
/// assert_eq!(tmpfile.exists(), false);
/// assert!(git::clone("https://github.com/phR0ze/alpine-base.git", &tmpdir).is_ok());
/// assert_eq!(tmpfile.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn clone_with_progress<T: AsRef<str>, U: AsRef<Path>>(url: T, dst: U, fetchopts: FetchOptions, checkout: CheckoutBuilder) -> Result<PathBuf> {
    RepoBuilder::new().fetch_options(fetchopts).with_checkout(checkout).clone(url.as_ref(), dst.as_ref())?;
    Ok(dst.as_ref().to_path_buf())
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(any(feature = "_net_", feature = "_arch_"))]
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // Test setup
    fn setup<T: AsRef<Path>>(path: T) -> PathBuf {
        let temp = PathBuf::from("tests/temp").abs().unwrap();
        sys::mkdir(&temp).unwrap();
        temp.mash(path.as_ref())
    }

    #[test]
    fn test_clone_with_progress() {
        let tmpdir = setup("git_cone_with_progress");
        let repo1 = tmpdir.mash("repo1");
        let repo1file = repo1.mash("README.md");
        assert!(sys::remove_all(&tmpdir).is_ok());

        // // Transfer progress callback
        // let mut cb = git::RemoteCallbacks::new();
        // cb.transfer_progress(|stats| {
        //     println!("Total Objects: {:?}", stats.total_objects());
        //     println!("Indexed Objects: {:?}", stats.indexed_objects());
        //     println!("Received Objects: {:?}", stats.received_objects());
        //     println!("Local Objects: {:?}", stats.local_objects());
        //     println!("Total Deltas: {:?}", stats.total_deltas());
        //     println!("Indexed Deltas: {:?}", stats.indexed_deltas());
        //     println!("Received Bytes: {:?}", stats.received_bytes());
        //     true
        // });
        // let mut fo = git::FetchOptions::new();
        // fo.remote_callbacks(cb);

        // // Checkout progress callback
        // let mut co = git::CheckoutBuilder::new();
        // co.progress(|path, cur, total| {
        //     //
        // });

        // // Clone repo 1
        // assert_eq!(repo1file.exists(), false);
        // assert!(git::clone_with_progress("https://github.com/phR0ze/alpine-base.git", &repo1, fo, co).is_ok());
        // assert_eq!(sys::readlines(&repo1file).unwrap()[0], "alpine-base".to_string());
        // assert_eq!(repo1file.exists(), true);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_clone() {
        let tmpdir = setup("git_clone");
        let repo1 = tmpdir.mash("repo1");
        let repo2 = tmpdir.mash("repo2");
        let repo1file = repo1.mash("README.md");
        let repo2file = repo2.mash("README.md");
        assert!(sys::remove_all(&tmpdir).is_ok());

        // Clone repo 1
        assert_eq!(repo1file.exists(), false);
        assert!(git::clone("https://github.com/phR0ze/alpine-base.git", &repo1).is_ok());
        assert_eq!(sys::readlines(&repo1file).unwrap()[0], "alpine-base".to_string());
        assert_eq!(repo1file.exists(), true);

        // Clone repo 2
        assert_eq!(repo2file.exists(), false);
        assert!(git::clone("https://github.com/phR0ze/alpine-core.git", &repo2).is_ok());
        assert_eq!(sys::readlines(&repo2file).unwrap()[0], "alpine-core".to_string());
        assert_eq!(repo2file.exists(), true);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_clone_branch() {
        let tmpdir = setup("git_clone_branch");
        let repo1 = tmpdir.mash("repo1");
        let repo2 = tmpdir.mash("repo2");
        let repo1file = repo1.mash("README.md");
        let repo2file = repo2.mash("trunk/PKGBUILD");
        assert!(sys::remove_all(&tmpdir).is_ok());

        // Clone single branch only repo 1
        assert_eq!(repo1file.exists(), false);
        assert!(git::clone_branch("https://github.com/phR0ze/alpine-base.git", "master", &repo1).is_ok());
        assert_eq!(sys::readlines(&repo1file).unwrap()[0], "alpine-base".to_string());
        assert_eq!(repo1file.exists(), true);

        // Clone single branch only repo 2
        assert_eq!(repo2file.exists(), false);
        assert!(git::clone_branch("https://git.archlinux.org/svntogit/packages.git", "packages/pkgfile", &repo2).is_ok());
        assert_eq!(repo2file.exists(), true);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_remote_branch_exists() {
        assert_eq!(git::remote_branch_exists("https://git.archlinux.org/svntogit/packages.git", "packages/foobar"), false);
        assert_eq!(git::remote_branch_exists("https://git.archlinux.org/svntogit/packages.git", "packages/pkgfile"), true);
        assert_eq!(git::remote_branch_exists("https://github.com/phR0ze/alpine-base.git", "master"), true);
        assert_eq!(git::remote_branch_exists("https://git.archlinux.org/svntogit/community.git", "packages/acme"), true);
    }

    #[test]
    fn test_remote_branch_exists_err() {
        assert!(git::remote_branch_exists_err("https://github.com/phR0ze/alpine-base.git", "master").is_ok());
        assert!(git::remote_branch_exists_err("https://git.archlinux.org/svntogit/packages.git", "packages/foobar").is_err());
        assert!(git::remote_branch_exists_err("https://git.archlinux.org/svntogit/packages.git", "packages/pkgfile").is_ok());
        assert!(git::remote_branch_exists_err("https://git.archlinux.org/svntogit/community.git", "packages/acme").is_ok());
    }
}
