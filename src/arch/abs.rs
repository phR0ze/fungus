// Arch Linux ABS
// https://wiki.archlinux.org/index.php/Arch_Build_System
// https://wiki.archlinux.org/index.php/Arch_Build_System#Retrieve_PKGBUILD_source_using_Git
//
// Inspired by the `asp` tool. Modify `/usr/bin/asp` add `set -x` to top before running
cfgblock! {
    #[cfg(feature = "_arch_")]
    use git2::{self, build::RepoBuilder};
    const TMPDIR: &str = "abs";
    const REPO_PACKAGES_NAME: &str = "packages";
    const REPO_COMMUNITY_NAME: &str = "community";
    const REPO_BASE: &str = "https://git.archlinux.org/svntogit";
}

use crate::prelude::*;

// An repo identifier
#[cfg(feature = "_arch_")]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Repo {
    /// Arch Linux packages repository
    Packages,

    /// Arch Linux community repository
    Community,
}

#[cfg(feature = "_arch_")]
impl Repo {
    pub fn from<T: AsRef<str>>(repo: T) -> Result<Repo> {
        match repo.as_ref() {
            REPO_PACKAGES_NAME => Ok(Repo::Packages),
            REPO_COMMUNITY_NAME => Ok(Repo::Community),
            _ => Err(ArchError::repo_not_found(repo.as_ref().to_string()).into()),
        }
    }
}

/// Get the linux kernel version for the standard `linux` package
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// println!("curren linux kernel version: {:?}", abs::kernel_ver().unwrap());
/// ```
#[cfg(feature = "_arch_")]
pub fn kernel_ver() -> Result<String> {
    // Download source to tmpdir
    let tmpdir = user::temp_dir(TMPDIR)?;
    let _f = finally(|| sys::remove_all(&tmpdir).unwrap());
    let src = source("linux", &tmpdir)?;

    // Extract the kernel version
    lazy_static! {
        static ref RX: Regex = Regex::new(r"(?m)^pkgver=(\d+\.\d+\.\d+).*").unwrap();
    }
    sys::extract_string(src.mash("PKGBUILD"), &RX)
}

/// Get the repo the given `pkg` lives in.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(abs::repo("pkgfile").unwrap(), abs::Repo::Packages);
/// ```
#[cfg(feature = "_arch_")]
pub fn repo<T: AsRef<str>>(pkg: T) -> Result<Repo> {
    for name in &vec![REPO_PACKAGES_NAME, REPO_COMMUNITY_NAME] {
        let url = format!("{}/{}.git", REPO_BASE, name);
        let branch = format!("packages/{}", pkg.as_ref());
        if git::remote_branch_exists(url, branch) {
            return Repo::from(name);
        }
    }
    Err(ArchError::package_not_found(pkg).into())
}

/// Download the package source for `pkg` to `dst`.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("abs_soure_doc");
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// assert!(sys::mkdir(&tmpdir).is_ok());
///
/// assert!(abs::source("pkgfile", &tmpdir).is_ok());
/// assert_eq!(tmpdir.is_dir(), true);
/// assert_eq!(tmpdir.mash("PKGBUILD").exists(), true);
///
/// assert!(sys::remove_all(&tmpdir).is_ok());
/// ```
#[cfg(feature = "_arch_")]
pub fn source<T: AsRef<str>, U: AsRef<Path>>(pkg: T, dst: U) -> Result<PathBuf> {
    for name in &vec![REPO_PACKAGES_NAME, REPO_COMMUNITY_NAME] {
        let url = format!("{}/{}.git", REPO_BASE, name);
        let branch = format!("packages/{}", pkg.as_ref());

        // Clone the single branch from the repo if it exists
        let tmpdir = user::temp_dir(TMPDIR)?;
        let _f = finally(|| sys::remove_all(&tmpdir).unwrap());
        if let Ok(_) = git::clone_branch(url, branch, &tmpdir) {
            // Copy out the target source in <tmpdir>/trunk/* to dst
            let dir = sys::mkdir(&dst)?;
            sys::copy(tmpdir.mash("trunk/*"), &dir)?;
            return Ok(dir);
        }
    }
    Err(ArchError::package_not_found(pkg).into())
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(feature = "_arch_")]
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // Reusable teset setup
    struct Setup {
        temp: PathBuf,
    }
    impl Setup {
        fn init() -> Self {
            let setup = Self { temp: PathBuf::from("tests/temp").abs().unwrap() };
            sys::mkdir(&setup.temp).unwrap();
            setup
        }
    }

    #[test]
    fn test_kernel_ver() {
        assert!(abs::kernel_ver().is_ok());
    }

    #[test]
    fn test_repo() {
        assert!(abs::repo("foobar").is_err());
        assert_eq!(abs::repo("pkgfile").unwrap(), abs::Repo::Packages);
        assert_eq!(abs::repo("acme").unwrap(), abs::Repo::Community);
        assert_eq!(abs::repo("linux").unwrap(), abs::Repo::Packages);
    }

    #[test]
    fn test_source() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("abs_source");
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert!(abs::source("pkgfile", &tmpdir).is_ok());
        assert_eq!(tmpdir.is_dir(), true);
        assert_eq!(tmpdir.mash("PKGBUILD").exists(), true);
        assert!(abs::source("foobar", &tmpdir).is_err());

        assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
