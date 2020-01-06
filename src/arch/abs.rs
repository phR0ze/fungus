// Arch Linux ABS
// https://wiki.archlinux.org/index.php/Arch_Build_System
// https://wiki.archlinux.org/index.php/Arch_Build_System#Retrieve_PKGBUILD_source_using_Git
use git2::build::RepoBuilder;

use crate::prelude::*;
use failure::bail;

const REPO_PACKAGES_NAME: &str = "packages";
const REPO_COMMUNITY_NAME: &str = "community";

// An repo identifier
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Repo {
    /// Arch Linux packages repository
    Packages,

    /// Arch Linux community repository
    Community,
}

#[cfg(feature = "_arch_")]
pub fn init<T: AsRef<Path>>(path: T) -> Result<()> {
    let pkg = "pkgfile".to_string();
    let path = path.as_ref().abs()?;

    // Ensure the destination dir exists
    sys::mkdir(&path)?;

    // Init the repo at the given destination dir
    let mut found = false;
    let mut repo: git2::Repository;
    for name in vec![REPO_PACKAGES_NAME, REPO_COMMUNITY_NAME] {
        // Configure the branch to checkout
        let branch = format!("{}/{}", &name, &pkg);
        let mut builder = RepoBuilder::new();
        builder.branch(&branch);

        // Create the remote with the --single-branch refspec override so that when clone is
        // called only the indicated branch is downloaded from the server not the whole repo.
        builder.remote_create(|repo, name, url| {
            let refspec = format!("+refs/heads/{0:}:refs/remotes/origin/{0:}", &branch);
            repo.remote_with_fetch(name, url, &refspec)
        });

        // Clone the single branch from teh repo
        match builder.clone(&format!("https://git.archlinux.org/svntogit/{}.git", &name), &path) {
            Ok(x) => {
                found = true;
                repo = x
            }
            Err(_) => continue,
        }
    }

    if !found {
        bail!("failed to find the package in any repo: {}", &pkg);
    }

    Ok(())
}

// /// Get the repo the given `pkg` lives in.
// #[cfg(feature = "_arch_")]
// pub fn repo<T: AsRef<str>>(pkg: T) -> Result<Repo> {
//     let branch = "packages/pkgfile".to_string();
//     let refspec = format!("+refs/heads/{0:}:refs/remotes/origin/{0:}", &branch);

//     let repo = Repository::init(&path)?;
//     for x in &vec![REPO_PACKAGES_NAME, REPO_COMMUNITY_NAME] {
//         let mut remote = repo.remote(x, &format!("https://git.archlinux.org/svntogit/{}.git", x))?;

//         // Fetch the remote branch via a refspec
//         remote.fetch(&[&refspec], None, None).unwrap();

//         //repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
//         let head = repo.find_reference("FETCH_HEAD")?;
//         let oid = head.target().unwrap();
//         let commit = repo.find_commit(oid)?;
//         repo.branch(&branch, &commit, false)?;
//         // println!("{:?}", oid);
//         println!("foo");
//     }
// }

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::arch::abs;
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
    fn test_init() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("abs_init");
        assert!(sys::remove_all(&tmpdir).is_ok());

        assert!(abs::init(&tmpdir).is_ok());

        // assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
