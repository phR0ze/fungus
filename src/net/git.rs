cfgblock! {
    #[cfg(any(feature = "_net_", feature = "_arch_"))]
    use git2::{self, FetchOptions, RemoteCallbacks};
    use git2::build::{CheckoutBuilder, RepoBuilder};
    const TMPDIR: &str = "git";
}
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::thread;

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

/// Clone the given repos emitting terminal progress. Clones the entire repo.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_clone_term_progress_doc");
/// let repo1 = tmpdir.mash("repo1");
/// let repo2 = tmpdir.mash("repo2");
/// let repo1file = repo1.mash("README.md");
/// let repo2file = repo2.mash("README.md");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let mut repos = HashMap::new();
/// repos.insert("https://github.com/phR0ze/alpine-base", &repo1);
/// repos.insert("https://github.com/phR0ze/alpine-core", &repo2);
/// assert!(git::clone_term_progress(&repos).is_ok());
/// assert_eq!(repo1file.exists(), true);
/// assert_eq!(repo2file.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn clone_term_progress<T: AsRef<str>, U: AsRef<Path>>(repos: &HashMap<T, U>) -> Result<()> {
    let progress = MultiProgress::new();
    let mut style = ProgressStyle::default_bar();
    style = style.progress_chars("=>-").template("[{elapsed_precise}][{bar:50.cyan/blue}] {pos:>7}/{len:7} ({eta}) - {msg}");

    // Spin off the cloning into separate threads leaving the main thread for multi-progress
    for (url, dst) in repos {
        let url = url.as_ref().to_string();
        let dst = dst.as_ref().to_path_buf();
        let xfer_bar = progress.add(ProgressBar::new(0).with_style(style.clone()));

        thread::spawn(move || {
            let mut xfer_init = false;
            let mut check_init = false;

            // Tracking transfer with indexed as this seems smoother and more logical for doneness
            let mut callback = RemoteCallbacks::new();
            callback.transfer_progress(|stats| {
                if !xfer_init {
                    xfer_bar.set_length(stats.total_objects() as u64);
                    xfer_bar.set_message(&url);
                    xfer_init = true;
                }
                xfer_bar.set_position(stats.indexed_objects() as u64);
                true
            });
            let mut fetchopts = FetchOptions::new();
            fetchopts.remote_callbacks(callback);

            // Tracking checkout separately
            let mut checkout = CheckoutBuilder::new();
            checkout.progress(|_, cur, total| {
                if !check_init {
                    xfer_bar.set_length(total as u64);
                    check_init = true;
                }
                xfer_bar.set_position(cur as u64);
            });

            RepoBuilder::new().fetch_options(fetchopts).with_checkout(checkout).clone(&url, &dst).unwrap();
            xfer_bar.finish_with_message(&url);
        });
    }
    progress.join()?;
    Ok(())
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
    fn test_clone_term_progress() {
        let tmpdir = setup("git_cone_term_progress");
        let repo1 = tmpdir.mash("repo1");
        let repo2 = tmpdir.mash("repo2");
        let repo1file = repo1.mash("README.md");
        let repo2file = repo2.mash("README.md");
        assert!(sys::remove_all(&tmpdir).is_ok());

        let mut repos = HashMap::new();
        repos.insert("https://github.com/phR0ze/alpine-base", &repo1);
        repos.insert("https://github.com/phR0ze/alpine-core", &repo2);
        assert!(git::clone_term_progress(&repos).is_ok());

        assert_eq!(repo1file.exists(), true);
        assert_eq!(repo2file.exists(), true);
        assert_eq!(sys::readlines(&repo1file).unwrap()[0].starts_with("alpine-"), true);
        assert_eq!(sys::readlines(&repo2file).unwrap()[0].starts_with("alpine-"), true);

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
