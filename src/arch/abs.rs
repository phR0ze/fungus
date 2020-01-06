// Arch Linux ABS
// https://wiki.archlinux.org/index.php/Arch_Build_System
// https://wiki.archlinux.org/index.php/Arch_Build_System#Retrieve_PKGBUILD_source_using_Git
//
// # Clone the svntogit repo for a specific package using https://github.com/archlinux/asp
// # asp replaces the abs tool, offering more up to date sources (via the svntogit repositories)
// # using a sparse checkout to cache at ${XDG_CACHE_HOME:-$HOME/.cache}/asp.
// $ asp checkout <pkgname>
//
// asp calls:
// $ git init -q ${XDG_CACHE_HOME:-$HOME/.cache}/asp
// $ git remote add "packages" "https://git.archlinux.org/svntogit/packages.git
// $ git remote add "community" "https://git.archlinux.org/svntogit/community.git
// $ git branch -qf "$branchname" "refs/remotes/$branchname"
// $ git branch -qf "$branchname" "refs/remotes/$branchname"
use git2::build::RepoBuilder;
use git2::Repository;

use crate::prelude::*;

#[cfg(feature = "_arch_")]
pub fn init<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref().abs()?;

    // Ensure the destination dir exists
    sys::mkdir(&path)?;

    // Init the repo at the given destination dir
    let repo = Repository::init(&path)?;

    // Create the refspec for our target branch
    let branch = "packages/pkgfile".to_string();
    let refspec = format!("+refs/heads/{0:}:refs/remotes/origin/{0:}", &branch);

    // Add the target remotes
    //for x in &vec!["packages", "community"] {
    for x in &vec!["packages"] {
        // Add the remote
        let mut remote = repo.remote(x, &format!("https://git.archlinux.org/svntogit/{}.git", x))?;

        // Fetch the remote branch via a refspec
        remote.fetch(&[&refspec], None, None).unwrap();

        //repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        let head = repo.find_reference("FETCH_HEAD")?;
        let oid = head.target().unwrap();
        let commit = repo.find_commit(oid)?;
        repo.branch(&branch, &commit, false)?;
        // println!("{:?}", oid);
        println!("foo");
    }

    // Checkout the branch
    //let builder = git2::build::CheckoutBuilder::new();
    //repo.checkout_head(opts: Option<&mut CheckoutBuilder<'_>>)
    // let head = repo.head()?;
    // let oid = head.target().unwrap();
    // let commit = repo.find_commit(oid)?;
    //repo.branch(&branch, target: &Commit<'_>, force: bool)

    // for x in remote.fetch_refspecs()?.iter() {
    //     println!("{}", x.unwrap().red().bold());
    // }
    //repo.remote_add_fetch(name: &str, spec: &str)
    //remote.fetch(&["master"], Some(opts), None)?;

    Ok(())
}

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
