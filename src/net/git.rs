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

/// Git repository
#[derive(Default)]
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub struct RepoGroup<'a> {
    repos: Vec<Repo<'a>>,
    style: Option<ProgressStyle>,
    progress: Option<MultiProgress>,
}

#[cfg(any(feature = "_net_", feature = "_arch_"))]
impl<'a> RepoGroup<'a> {
    /// Create a new repo group instance.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let _group = git::RepoGroup::new();
    /// ```
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    /// Add the given `repo` to the repo group
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let group = git::RepoGroup::new();
    /// group.add(git::Repo::new("foo").unwrap());
    /// ```
    pub fn add(mut self, repo: Repo<'a>) -> Self {
        self.repos.push(repo);
        self
    }

    /// Enable terminal progress bars using the indicatif crate
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let group = git::RepoGroup::new().with_progress(true);
    /// ```
    pub fn with_progress(mut self, yes: bool) -> Self {
        if yes {
            let progress = MultiProgress::new();
            let mut style = ProgressStyle::default_bar();
            style = style.progress_chars("=>-").template("[{elapsed_precise}][{bar:50.cyan/blue}] {pos:>7}/{len:7} ({eta}) - {msg}");
            self.progress = Some(progress);
            self.style = Some(style);
        }
        self
    }

    /// Clone the repos locally. This method spins off threads to handle cloning all the
    /// repos in parallel. Calling this function consumes any progress callbacks you may have set.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_repo_clone_many_doc");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let repo1 = tmpdir.mash("repo1");
    /// let repo2 = tmpdir.mash("repo2");
    /// let repo1file = repo1.mash("README.md");
    /// let repo2file = repo2.mash("README.md");
    /// assert!(sys::mkdir(&tmpdir).is_ok());
    /// let repos = git::RepoGroup::new()
    ///    .with_progress(true)
    ///    .add(git::Repo::new(&repo1).unwrap().url("https://github.com/phR0ze/alpine-base"))
    ///    .add(git::Repo::new(&repo2).unwrap().url("https://github.com/phR0ze/alpine-core"));
    /// assert!(repos.clone().is_ok());
    /// assert_eq!(repo1file.exists(), true);
    /// assert_eq!(repo2file.exists(), true);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    pub fn clone(&self) -> Result<()> {
        let mut threads = Vec::new();
        for repo in &self.repos {
            // Note: I had to make 'path' and 'url' owned types for the thread lifetime to work
            let path = repo.path_val().to_path_buf();
            let url = repo.url_val().ok_or_else(|| GitError::UrlNotSet)?.to_string();

            if self.progress.is_none() {
                threads.push(thread::spawn(move || {
                    Repo::new(path).unwrap().url(url).clone().unwrap();
                }));
            } else {
                let progress = self.progress.as_ref().unwrap();
                let progress_bar = progress.add(ProgressBar::new(0).with_style(self.style.as_ref().unwrap().clone()));
                progress_bar.set_message(&url);
                let msg = url.clone();

                thread::spawn(move || {
                    let mut xfer_init = false;
                    let mut check_init = false;
                    Repo::new(path)
                        .unwrap()
                        .url(url)
                        .xfer_progress(|total, cur| {
                            if !xfer_init {
                                progress_bar.set_length(total);
                                xfer_init = true;
                            }
                            progress_bar.set_position(cur);
                        })
                        .checkout_progress(|total, cur| {
                            if !check_init {
                                progress_bar.set_length(total);
                                check_init = true;
                            }
                            progress_bar.set_position(cur);
                        })
                        .clone()
                        .unwrap();

                    progress_bar.finish_with_message(&msg);
                });
            }
        }

        // Wait for other threads to finish.
        if self.progress.is_none() {
            for thread in threads {
                thread.join().unwrap();
            }
        } else {
            let progress = self.progress.as_ref().unwrap();
            progress.join()?;
        }
        Ok(())
    }
}

/// Git repository
#[derive(Default)]
#[cfg(any(feature = "_net_", feature = "_arch_"))]
pub struct Repo<'a> {
    path: PathBuf,                                            // Repo location on disk
    url: Option<String>,                                      // Repo location on the network
    branch_only: bool,                                        // Clone only the given branch
    branch: Option<String>,                                   // Specific branch to work with
    xfer_progress: Option<Box<dyn FnMut(u64, u64) + 'a>>,     // Transfer progress callback
    update_progress: Option<Box<dyn FnMut(u64, u64) + 'a>>,   // Update progress callback
    checkout_progress: Option<Box<dyn FnMut(u64, u64) + 'a>>, // Checkout progress callback
}

#[cfg(any(feature = "_net_", feature = "_arch_"))]
impl<'a> Repo<'a> {
    // ---------------------------------------------------------------------------------------------
    // Field getters/setters
    // ---------------------------------------------------------------------------------------------

    /// Returns the target branch for this repo. Defaults to `master` internally when not set.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(git::Repo::new("foo").unwrap().branch("foobar").branch_val(), Some("foobar"));
    /// ```
    pub fn branch_val(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    /// Returns the branch flag's value for this repo.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(git::Repo::new("foo").unwrap().branch_only(true).branch_only_val(), true);
    /// ```
    pub fn branch_only_val(&self) -> bool {
        self.branch_only
    }

    /// Returns the local location on disk for this repo
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(git::Repo::new("foo").unwrap().path_val(), Path::new("foo").abs().unwrap().as_path());
    /// ```
    pub fn path_val(&self) -> &Path {
        &self.path
    }

    /// Returns the remote location for this repo
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(git::Repo::new("foo").unwrap().url("foobar").url_val(), Some("foobar"));
    /// ```
    pub fn url_val(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Set the branch to target for this repo
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(git::Repo::new("foo").unwrap().branch("foobar").branch_val(), Some("foobar"));
    /// ```
    pub fn branch<T>(mut self, branch: T) -> Self
    where
        T: AsRef<str>,
    {
        self.branch = Some(branch.as_ref().to_string());
        self
    }

    /// Set to target the branch only and not the entire repo.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(git::Repo::new("foo").unwrap().branch_only(true).branch_only_val(), true);
    /// ```
    pub fn branch_only(mut self, yes: bool) -> Self {
        self.branch_only = yes;
        self
    }

    /// Set the remote location for this repo
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// git::Repo::new("foo").unwrap().url("foobar");
    /// ```
    pub fn url<T>(mut self, url: T) -> Self
    where
        T: AsRef<str>,
    {
        self.url = Some(url.as_ref().to_string());
        self
    }

    /// Set the transfer progress callback to use.
    ///
    /// ### Examples
    /// ```ignore
    /// use fungus::prelude::*;
    ///
    /// git::Repo::new("foo").unwrap().url("foobar");
    /// ```
    pub fn xfer_progress<T>(mut self, func: T) -> Self
    where
        T: FnMut(u64, u64) + 'a,
    {
        self.xfer_progress = Some(Box::new(func));
        self
    }

    /// Set the checkout progress callback to use.
    ///
    /// ### Examples
    /// ```ignore
    /// use fungus::prelude::*;
    ///
    /// git::Repo::new("foo").unwrap().url("foobar");
    /// ```
    pub fn checkout_progress<T>(mut self, func: T) -> Self
    where
        T: FnMut(u64, u64) + 'a,
    {
        self.checkout_progress = Some(Box::new(func));
        self
    }

    /// Set the update progress callback to use.
    ///
    /// ### Examples
    /// ```ignore
    /// use fungus::prelude::*;
    ///
    /// git::Repo::new("foo").unwrap().url("foobar");
    /// ```
    pub fn update_progress<T>(mut self, func: T) -> Self
    where
        T: FnMut(u64, u64) + 'a,
    {
        self.update_progress = Some(Box::new(func));
        self
    }

    // ---------------------------------------------------------------------------------------------
    // Functions/Methods
    // ---------------------------------------------------------------------------------------------

    /// Create a new repo instance based on the given path
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert!(git::Repo::new("foo").is_ok());
    /// ```
    pub fn new<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref().abs()?;
        Ok(Self { path: path, ..Default::default() })
    }

    /// Returns the message from the head commit.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_repo_last_msg_doc");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// let tarball = tmpdir.mash("../../alpine-base.tgz");
    /// assert!(tar::extract_all(&tarball, &tmpdir).is_ok());
    /// assert_eq!(git::Repo::new(&tmpdir).unwrap().last_msg().unwrap(), "Use the workflow name for the badge".to_string());
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    pub fn last_msg(&self) -> Result<String> {
        let repo = Repository::open(self.path_val())?;
        let head = repo.head()?.peel_to_commit()?;
        let msg = head.message().ok_or_else(|| GitError::NoMessageWasFound)?;
        Ok(msg.trim_end().to_string())
    }

    /// Clone the repo locally. Clones the entire repo unless branch_only is set to true.
    /// Calling this function consumes any progress callbacks you may have set.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("git_repo_clone_doc");
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// assert!(sys::mkdir(&tmpdir).is_ok());
    /// let tmpfile = tmpdir.mash("README.md");
    /// assert_eq!(tmpfile.exists(), false);
    /// assert!(git::Repo::new(&tmpdir).unwrap().url("https://github.com/phR0ze/alpine-base").clone().is_ok());
    /// assert_eq!(tmpfile.exists(), true);
    /// assert!(sys::remove_all(&tmpdir).is_ok());
    /// ```
    pub fn clone(mut self) -> Result<PathBuf> {
        let mut builder = RepoBuilder::new();

        // Clone only the target branch if set
        if self.branch_only {
            let branch = match &self.branch {
                Some(x) => x.clone(),
                None => "master".to_string(),
            };
            builder.branch(&branch);

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
            builder.remote_create(move |repo, name, url| {
                let refspec = format!("+refs/heads/{0:}:refs/remotes/origin/{0:}", &branch);
                repo.remote_with_fetch(name, url, &refspec)
            });
        }

        // Transfer progress callback
        if self.xfer_progress.is_some() {
            let mut xfer = self.xfer_progress.take().unwrap();
            let mut callback = RemoteCallbacks::new();
            callback.transfer_progress(move |stats| {
                xfer(stats.total_objects() as u64, stats.indexed_objects() as u64);
                true
            });
            let mut fetchopts = FetchOptions::new();
            fetchopts.remote_callbacks(callback);
            builder.fetch_options(fetchopts);
        }

        // Checkout progress callback
        if self.checkout_progress.is_some() {
            let mut checkout = self.checkout_progress.take().unwrap();
            let mut checkout_bldr = CheckoutBuilder::new();
            checkout_bldr.progress(move |_, cur, total| checkout(total as u64, cur as u64));
            builder.with_checkout(checkout_bldr);
        }

        let url = self.url_val().ok_or_else(|| GitError::UrlNotSet)?;
        let path = self.path_val();
        builder.clone(url, path)?;
        Ok(self.path.clone())
    }
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
        Repo::new(&path)?.url(url).xfer_progress(xfer).checkout_progress(checkout).clone()?;
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
    fn test_repogroup() {
        let group = git::RepoGroup::new();
        group.add(git::Repo::new("foo").unwrap());
    }

    #[test]
    fn test_repo_branch() {
        // chained
        assert_eq!(git::Repo::new("foo").unwrap().branch("foobar").branch_val(), Some("foobar"));

        // unchained
        let mut repo = git::Repo::new("foo").unwrap();
        assert_eq!(repo.branch_val(), None);
        repo = repo.branch("foobar");
        assert_eq!(repo.branch_val(), Some("foobar"));
    }

    #[test]
    fn test_repo_branch_only() {
        // chained
        assert_eq!(git::Repo::new("foo").unwrap().branch_only(true).branch_only_val(), true);

        // unchained
        let mut repo = git::Repo::new("foo").unwrap();
        assert_eq!(repo.branch_only_val(), false);
        repo = repo.branch_only(true);
        assert_eq!(repo.branch_only_val(), true);
    }

    #[test]
    fn test_repo_path() {
        assert_eq!(git::Repo::new("foo").unwrap().path_val(), Path::new("foo").abs().unwrap().as_path());
    }

    #[test]
    fn test_repo_url() {
        // chained
        assert_eq!(git::Repo::new("foo").unwrap().url("foobar").url_val(), Some("foobar"));

        // unchained
        let mut repo = git::Repo::new("foo").unwrap();
        assert_eq!(repo.url_val(), None);
        repo = repo.url("foobar");
        assert_eq!(repo.url_val(), Some("foobar"));
    }

    #[test]
    fn test_repo_clone() {
        let tmpdir = setup("git_repo_clone");
        let repo1 = tmpdir.mash("repo1");
        let repo2 = tmpdir.mash("repo2");
        let repo1file = repo1.mash("README.md");
        let repo2file = repo2.mash("README.md");
        assert!(sys::remove_all(&tmpdir).is_ok());

        // Clone repo 1
        assert_eq!(repo1file.exists(), false);
        assert!(git::Repo::new(&repo1).unwrap().url("https://github.com/phR0ze/alpine-base.git").clone().is_ok());
        assert_eq!(sys::readlines(&repo1file).unwrap()[0], "alpine-base".to_string());
        assert_eq!(repo1file.exists(), true);

        // Clone repo 2
        assert_eq!(repo2file.exists(), false);
        assert!(git::Repo::new(&repo2).unwrap().url("https://github.com/phR0ze/alpine-core.git").clone().is_ok());
        assert_eq!(sys::readlines(&repo2file).unwrap()[0], "alpine-core".to_string());
        assert_eq!(repo2file.exists(), true);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_repo_clone_branch() {
        let tmpdir = setup("git_repo_clone_branch");
        let repo1 = tmpdir.mash("repo1");
        let repo2 = tmpdir.mash("repo2");
        let repo1file = repo1.mash("README.md");
        let repo2file = repo2.mash("trunk/PKGBUILD");
        assert!(sys::remove_all(&tmpdir).is_ok());

        // Clone single branch only repo 1
        assert_eq!(repo1file.exists(), false);
        assert!(git::Repo::new(&repo1).unwrap().url("https://github.com/phR0ze/alpine-base.git").branch("master").branch_only(true).clone().is_ok());
        assert_eq!(sys::readlines(&repo1file).unwrap()[0], "alpine-base".to_string());
        assert_eq!(repo1file.exists(), true);

        // Clone single branch only repo 2
        assert_eq!(repo2file.exists(), false);
        assert!(git::Repo::new(&repo2)
            .unwrap()
            .url("https://git.archlinux.org/svntogit/packages.git")
            .branch("packages/pkgfile")
            .branch_only(true)
            .clone()
            .is_ok());
        assert_eq!(repo2file.exists(), true);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_repo_clone_with_progress() {
        let tmpdir = setup("git_repo_clone_with_progress");
        let readme = tmpdir.mash("README.md");
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert!(git::Repo::new(&tmpdir)
            .unwrap()
            .url("https://github.com/phR0ze/alpine-base")
            //.xfer_progress(|total, cur| println!("Xfer: {}, {}", total, cur))
            .xfer_progress(|total, cur| { let _ = total + cur; } )
            //.checkout_progress(|total, cur| println!("Checkout: {}, {}", total, cur))
            .checkout_progress(|total, cur| { let _ = total + cur; } )
            .clone()
            .is_ok());
        assert_eq!(readme.exists(), true);
        assert_eq!(sys::readlines(&readme).unwrap()[0].starts_with("alpine-base"), true);
        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_repo_clone_many() {
        let tmpdir = setup("git_repo_clone_many");
        let repo1 = tmpdir.mash("repo1");
        let repo2 = tmpdir.mash("repo2");
        let repo1file = repo1.mash("README.md");
        let repo2file = repo2.mash("README.md");
        assert!(sys::remove_all(&tmpdir).is_ok());

        let repos = git::RepoGroup::new()
            .add(git::Repo::new(&repo1).unwrap().url("https://github.com/phR0ze/alpine-base"))
            .add(git::Repo::new(&repo2).unwrap().url("https://github.com/phR0ze/alpine-core"));
        assert!(repos.clone().is_ok());

        assert_eq!(repo1file.exists(), true);
        assert_eq!(repo2file.exists(), true);
        assert_eq!(sys::readlines(&repo1file).unwrap()[0].starts_with("alpine-"), true);
        assert_eq!(sys::readlines(&repo2file).unwrap()[0].starts_with("alpine-"), true);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }

    #[test]
    fn test_repo_clone_many_with_progress() {
        let tmpdir = setup("git_repo_clone_many_with_progress");
        let repo1 = tmpdir.mash("repo1");
        let repo2 = tmpdir.mash("repo2");
        let repo1file = repo1.mash("README.md");
        let repo2file = repo2.mash("README.md");
        assert!(sys::remove_all(&tmpdir).is_ok());

        let repos = git::RepoGroup::new()
            .with_progress(true)
            .add(git::Repo::new(&repo1).unwrap().url("https://github.com/phR0ze/alpine-base"))
            .add(git::Repo::new(&repo2).unwrap().url("https://github.com/phR0ze/alpine-core"));
        assert!(repos.clone().is_ok());

        assert_eq!(repo1file.exists(), true);
        assert_eq!(repo2file.exists(), true);
        assert_eq!(sys::readlines(&repo1file).unwrap()[0].starts_with("alpine-"), true);
        assert_eq!(sys::readlines(&repo2file).unwrap()[0].starts_with("alpine-"), true);

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
        let tmpdir = setup("git_repo_last_msg");
        let tarball = tmpdir.mash("../../alpine-base.tgz");
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(tar::extract_all(&tarball, &tmpdir).is_ok());

        assert_eq!(git::Repo::new(&tmpdir).unwrap().last_msg().unwrap(), "Use the workflow name for the badge".to_string());

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
        let repo = git::Repo::new(&tmpdir).unwrap();
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(tar::extract_all(&tarball, &tmpdir).is_ok());
        assert_eq!(repo.last_msg().unwrap(), "Use the workflow name for the badge".to_string());
        assert!(git::update("https://github.com/phR0ze/alpine-base.git", &tmpdir).is_ok());
        assert_ne!(repo.last_msg().unwrap(), "Use the workflow name for the badge".to_string());

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
        let repo = git::Repo::new(&tmpdir).unwrap();
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert!(tar::extract_all(&tarball, &tmpdir).is_ok());
        assert_eq!(repo.last_msg().unwrap(), "Use the workflow name for the badge".to_string());
        assert!(git::update_with_progress("https://github.com/phR0ze/alpine-base", &tmpdir, |_, _| {}, |_, _| {}, |_, _| {},).is_ok());
        assert_ne!(repo.last_msg().unwrap(), "Use the workflow name for the badge".to_string());

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
        assert_eq!(git::Repo::new(&tmpdir).unwrap().last_msg().unwrap(), "Use the workflow name for the badge".to_string());
        let mut repos = HashMap::new();
        repos.insert("https://github.com/phR0ze/alpine-base", &repo1);
        assert!(git::update_term_progress(&repos).is_ok());
        assert_ne!(git::Repo::new(&repo1).unwrap().last_msg().unwrap(), "Use the workflow name for the badge".to_string());

        assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
