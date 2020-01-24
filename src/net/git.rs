cfgblock! {
    #[cfg(any(feature = "_net_", feature = "_arch_"))]
    use git2::{self, FetchOptions, RemoteCallbacks, Repository};
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
pub fn clone<T, U>(url: T, path: U) -> Result<PathBuf>
where
    T: AsRef<str>,
    U: AsRef<Path>,
{
    let path = path.as_ref().abs()?;
    let mut builder = RepoBuilder::new();
    builder.clone(url.as_ref(), path.as_ref())?;
    Ok(path)
}

/// Clone the repo locally. Clones the entire repo. Provides progress callbacks for the clone
/// transfer and the final checkout.
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
/// assert!(git::clone_with_progress("https://github.com/phR0ze/alpine-base", &tmpdir, |_, _| {}, |_, _| {},).is_ok());
/// assert_eq!(tmpfile.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn clone_with_progress<T, U, V, W>(url: T, dst: U, mut xfer: V, mut checkout: W) -> Result<PathBuf>
where
    T: AsRef<str>,
    U: AsRef<Path>,
    V: FnMut(u64, u64),
    W: FnMut(u64, u64),
{
    let dst = dst.as_ref().abs()?;
    let url = url.as_ref().to_string();

    // Tracking transfer with indexed as this seems smoother and more logical for doneness
    let mut callback = RemoteCallbacks::new();
    callback.transfer_progress(|stats| {
        xfer(stats.total_objects() as u64, stats.indexed_objects() as u64);
        true
    });
    let mut fetchopts = FetchOptions::new();
    fetchopts.remote_callbacks(callback);

    // Tracking checkout separately
    let mut checkout_bldr = CheckoutBuilder::new();
    checkout_bldr.progress(|_, cur, total| checkout(total as u64, cur as u64));

    RepoBuilder::new().fetch_options(fetchopts).with_checkout(checkout_bldr).clone(&url, &dst).unwrap();
    Ok(dst)
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
pub fn clone_branch<T, U, V>(url: T, branch: U, dst: V) -> Result<PathBuf>
where
    T: AsRef<str>,
    U: AsRef<str>,
    V: AsRef<Path>,
{
    let dst = dst.as_ref().abs()?;
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

    Ok(dst)
}

/// Clone the given repos emitting terminal progress. Clones the entire repo.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_clone_term_progress_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let repo1 = tmpdir.mash("repo1");
/// let repo2 = tmpdir.mash("repo2");
/// let repo1file = repo1.mash("README.md");
/// let repo2file = repo2.mash("README.md");
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
pub fn clone_term_progress<T, U>(repos: &HashMap<T, U>) -> Result<()>
where
    T: AsRef<str>,
    U: AsRef<Path>,
{
    let progress = MultiProgress::new();
    let mut style = ProgressStyle::default_bar();
    style = style.progress_chars("=>-").template("[{elapsed_precise}][{bar:50.cyan/blue}] {pos:>7}/{len:7} ({eta}) - {msg}");

    // Spin off the cloning into separate threads leaving the main thread for multi-progress
    for (url, dst) in repos {
        let xfer_bar = progress.add(ProgressBar::new(0).with_style(style.clone()));
        let dst = dst.as_ref().abs()?;
        let url = url.as_ref().to_string();
        xfer_bar.set_message(&url);
        let msg = url.clone();

        thread::spawn(move || {
            let mut xfer_init = false;
            let mut check_init = false;
            clone_with_progress(
                url,
                dst,
                |total, cur| {
                    if !xfer_init {
                        xfer_bar.set_length(total);
                        xfer_init = true;
                    }
                    xfer_bar.set_position(cur);
                },
                |total, cur| {
                    if !check_init {
                        xfer_bar.set_length(total);
                        check_init = true;
                    }
                    xfer_bar.set_position(cur);
                },
            )
            .unwrap();

            xfer_bar.finish_with_message(&msg);
        });
    }
    progress.join()?;
    Ok(())
}

/// Returns true if the `path` directory is a repositiory
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_is_repo_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert_eq!(git::is_repo(&tmpdir), false);
/// assert!(git2::Repository::init(&tmpdir).is_ok());
/// assert_eq!(git::is_repo(&tmpdir), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn is_repo<T>(path: T) -> bool
where
    T: AsRef<Path>,
{
    sys::is_dir(path.as_ref().mash(".git"))
}

/// Returns the message from the head commit.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_last_msg_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let tarball = tmpdir.mash("../../alpine-base.tgz");
/// assert!(tar::extract_all(&tarball, &tmpdir).is_ok());
/// assert_eq!(git::last_msg(&tmpdir).unwrap(), "Use the workflow name for the badge".to_string());
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn last_msg<T>(path: T) -> Result<String>
where
    T: AsRef<Path>,
{
    let path = path.as_ref().abs()?;

    let repo = Repository::open(&path)?;
    let head = repo.head()?.peel_to_commit()?;
    let msg = head.message().ok_or_else(|| GitError::NoMessageWasFound)?;

    Ok(msg.trim_end().to_string())
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
pub fn remote_branch_exists<T, U>(url: T, branch: U) -> bool
where
    T: AsRef<str>,
    U: AsRef<str>,
{
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
pub fn remote_branch_exists_err<T, U>(url: T, branch: U) -> Result<()>
where
    T: AsRef<str>,
    U: AsRef<str>,
{
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

/// Update the given repo, cloning the repo if it doesn't exist.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_update_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("README.md");
/// assert_eq!(tmpfile.exists(), false);
/// assert!(git::update("https://github.com/phR0ze/alpine-base.git", &tmpdir).is_ok());
/// assert_eq!(tmpfile.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn update<T, U>(url: T, path: U) -> Result<PathBuf>
where
    T: AsRef<str>,
    U: AsRef<Path>,
{
    let path = path.as_ref().abs()?;

    // Clone instead of update
    if !is_repo(&path) {
        let mut builder = RepoBuilder::new();
        builder.clone(url.as_ref(), path.as_ref())?;
    } else {
        let repo = Repository::open(&path)?;

        // Fetch the latest from origin/master
        repo.find_remote("origin")?.fetch(&["master"], None, None)?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;

        // Check if we need to update or not
        if analysis.is_up_to_date() {
            return Ok(path);
        } else if analysis.is_fast_forward() {
            let refname = "refs/heads/master";
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        } else {
            return Err(GitError::FastForwardOnly.into());
        }
    }
    Ok(path)
}

/// Update the given repo, cloning the repo if it doesn't exist. Accepts progress callbacks.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_update_with_progress_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let tmpfile = tmpdir.mash("README.md");
/// assert_eq!(tmpfile.exists(), false);
/// assert!(git::update_with_progress("https://github.com/phR0ze/alpine-base", &tmpdir, |_, _| {}, |_, _| {}, |_, _| {},).is_ok());
/// assert_eq!(tmpfile.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn update_with_progress<T, U, V, W, X>(url: T, path: U, xfer: V, checkout: W, mut update: X) -> Result<PathBuf>
where
    T: AsRef<str>,
    U: AsRef<Path>,
    V: FnMut(u64, u64),
    W: FnMut(u64, u64),
    X: FnMut(u64, u64),
{
    let path = path.as_ref().abs()?;

    // Clone instead of update
    if !is_repo(&path) {
        clone_with_progress(url.as_ref(), &path, xfer, checkout)?;
    } else {
        let repo = Repository::open(&path)?;

        // Tracking transfer with indexed as this seems smoother and more logical for doneness
        let mut callback = RemoteCallbacks::new();
        callback.transfer_progress(|stats| {
            update(stats.total_objects() as u64, stats.indexed_objects() as u64);
            true
        });
        let mut fetchopts = FetchOptions::new();
        fetchopts.remote_callbacks(callback);

        // Fetch the latest from origin/master
        repo.find_remote("origin")?.fetch(&["master"], Some(&mut fetchopts), None)?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;

        // Check if we need to update or not
        if analysis.is_up_to_date() {
            return Ok(path);
        } else if analysis.is_fast_forward() {
            let refname = "refs/heads/master";
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        } else {
            return Err(GitError::FastForwardOnly.into());
        }
    }
    Ok(path)
}

/// Update the given repos emitting terminal progress. Clones the entire repo if necessary.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_update_term_progress_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// let repo1 = tmpdir.mash("repo1");
/// let repo2 = tmpdir.mash("repo2");
/// let repo1file = repo1.mash("README.md");
/// let repo2file = repo2.mash("README.md");
/// assert!(sys::mkdir(&tmpdir).is_ok());
/// let mut repos = HashMap::new();
/// repos.insert("https://github.com/phR0ze/alpine-base", &repo1);
/// repos.insert("https://github.com/phR0ze/alpine-core", &repo2);
/// assert!(git::update_term_progress(&repos).is_ok());
/// assert_eq!(repo1file.exists(), true);
/// assert_eq!(repo2file.exists(), true);
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub fn update_term_progress<T, U>(repos: &HashMap<T, U>) -> Result<()>
where
    T: AsRef<str>,
    U: AsRef<Path>,
{
    let progress = MultiProgress::new();
    let mut style = ProgressStyle::default_bar();
    style = style.progress_chars("=>-").template("[{elapsed_precise}][{bar:50.cyan/blue}] {pos:>7}/{len:7} ({eta}) - {msg}");

    // Spin off the updating into separate threads leaving the main thread for multi-progress
    for (url, dst) in repos {
        let bar = progress.add(ProgressBar::new(0).with_style(style.clone()));
        let dst = dst.as_ref().abs()?;
        let url = url.as_ref().to_string();
        bar.set_message(&url);
        let msg = url.clone();

        thread::spawn(move || {
            let mut xfer_init = false;
            let mut check_init = false;
            let mut update_init = false;
            update_with_progress(
                url,
                dst,
                |total, cur| {
                    if !xfer_init {
                        bar.set_length(total);
                        xfer_init = true;
                    }
                    bar.set_position(cur);
                },
                |total, cur| {
                    if !check_init {
                        bar.set_length(total);
                        check_init = true;
                    }
                    bar.set_position(cur);
                },
                |total, cur| {
                    if !update_init {
                        bar.set_length(total);
                        update_init = true;
                    }
                    bar.set_position(cur);
                },
            )
            .unwrap();

            bar.finish_with_message(&msg);
        });
    }
    progress.join()?;
    Ok(())
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(any(feature = "_net_", feature = "_arch_"))]
#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use git2::build::{CheckoutBuilder, RepoBuilder};
    use git2::{self, FetchOptions, RemoteCallbacks, Repository};

    // Test setup
    fn setup<T: AsRef<Path>>(path: T) -> PathBuf {
        let temp = PathBuf::from("tests/temp").abs().unwrap();
        sys::mkdir(&temp).unwrap();
        temp.mash(path.as_ref())
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
    fn test_clone_with_progress() {
        let tmpdir = setup("git_clone_with_progress");
        let readme = tmpdir.mash("README.md");
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert!(git::clone_with_progress(
            "https://github.com/phR0ze/alpine-base",
            &tmpdir,
            |_total, _cur| {
                //println!("Xfer Total: {}, Cur: {}", total, cur);
            },
            |_total, _cur| {
                //println!("Checkout Total: {}, Cur: {}", total, cur);
            },
        )
        .is_ok());
        assert_eq!(readme.exists(), true);
        assert_eq!(sys::readlines(&readme).unwrap()[0].starts_with("alpine-base"), true);
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_clone_term_progress() {
        let tmpdir = setup("git_clone_term_progress");
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
    fn test_is_repo() {
        let tmpdir = setup("git_is_repo");
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert_eq!(git::is_repo(&tmpdir), false);
        assert!(git2::Repository::init(&tmpdir).is_ok());
        assert_eq!(git::is_repo(&tmpdir), true);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_last_msg() {
        let tmpdir = setup("git_last_msg");
        let tarball = tmpdir.mash("../../alpine-base.tgz");
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(tar::extract_all(&tarball, &tmpdir).is_ok());

        assert_eq!(git::last_msg(&tmpdir).unwrap(), "Use the workflow name for the badge".to_string());

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

    #[test]
    fn test_update() {
        let tmpdir = setup("git_update");
        let tarball = tmpdir.mash("../../alpine-base.tgz");
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert_eq!(git::is_repo(&tmpdir), false);
        assert!(git::update("https://github.com/phR0ze/alpine-base.git", &tmpdir).is_ok());
        assert_eq!(git::is_repo(&tmpdir), true);
        assert!(git::update("https://github.com/phR0ze/alpine-base.git", &tmpdir).is_ok());

        // Now wipe it out and extract a tarball of the repo that needs updated
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(tar::extract_all(&tarball, &tmpdir).is_ok());
        assert_eq!(git::last_msg(&tmpdir).unwrap(), "Use the workflow name for the badge".to_string());
        assert!(git::update("https://github.com/phR0ze/alpine-base.git", &tmpdir).is_ok());
        assert_ne!(git::last_msg(&tmpdir).unwrap(), "Use the workflow name for the badge".to_string());

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_update_with_progress() {
        let tmpdir = setup("git_update_with_progress");
        let tarball = tmpdir.mash("../../alpine-base.tgz");
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert_eq!(git::is_repo(&tmpdir), false);
        assert!(git::update_with_progress("https://github.com/phR0ze/alpine-base", &tmpdir, |_, _| {}, |_, _| {}, |_, _| {},).is_ok());
        assert_eq!(git::is_repo(&tmpdir), true);
        assert!(git::update_with_progress("https://github.com/phR0ze/alpine-base", &tmpdir, |_, _| {}, |_, _| {}, |_, _| {},).is_ok());

        // Now wipe it out and extract a tarball of the repo that needs updated
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(tar::extract_all(&tarball, &tmpdir).is_ok());
        assert_eq!(git::last_msg(&tmpdir).unwrap(), "Use the workflow name for the badge".to_string());
        assert!(git::update_with_progress("https://github.com/phR0ze/alpine-base", &tmpdir, |_, _| {}, |_, _| {}, |_, _| {},).is_ok());
        assert_ne!(git::last_msg(&tmpdir).unwrap(), "Use the workflow name for the badge".to_string());

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_update_term_progress() {
        let tmpdir = setup("git_update_term_progress");
        let tarball = tmpdir.mash("../../alpine-base.tgz");
        let repo1 = tmpdir.mash("repo1");
        let repo2 = tmpdir.mash("repo2");
        let repo1file = repo1.mash("README.md");
        let repo2file = repo2.mash("README.md");

        assert!(sys::remove_all(&tmpdir).is_ok());
        let mut repos = HashMap::new();
        repos.insert("https://github.com/phR0ze/alpine-base", &repo1);
        repos.insert("https://github.com/phR0ze/alpine-core", &repo2);
        assert!(git::update_term_progress(&repos).is_ok());

        assert_eq!(repo1file.exists(), true);
        assert_eq!(repo2file.exists(), true);
        assert_eq!(sys::readlines(&repo1file).unwrap()[0].starts_with("alpine-"), true);
        assert_eq!(sys::readlines(&repo2file).unwrap()[0].starts_with("alpine-"), true);

        // Now wipe it out and extract a tarball of the repo that needs updated
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(tar::extract_all(&tarball, &tmpdir).is_ok());
        assert_eq!(git::last_msg(&tmpdir).unwrap(), "Use the workflow name for the badge".to_string());
        let mut repos = HashMap::new();
        repos.insert("https://github.com/phR0ze/alpine-base", &repo1);
        assert!(git::update_term_progress(&repos).is_ok());
        assert_ne!(git::last_msg(&repo1).unwrap(), "Use the workflow name for the badge".to_string());

        assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
