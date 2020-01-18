use fungus::prelude::*;
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("=====================================================================");
    println!("=                         LIVE TESTING                              =");
    println!("=====================================================================");
    println!("HOME: {:?}", user::home_dir().unwrap());
    test_git_clone_term_progress()?;
    test_libc()?;
    test_crypto()?;
    Ok(())
}

fn test_git_clone_term_progress() -> Result<()> {
    let tmpdir = setup("git_clone_term_progress");
    let repo1 = tmpdir.mash("repo1");
    let repo2 = tmpdir.mash("repo2");
    let repo3 = tmpdir.mash("repo3");
    assert!(sys::remove_all(&tmpdir).is_ok());

    let mut repos = HashMap::new();
    repos.insert("https://github.com/phR0ze/alpine-base", &repo1);
    repos.insert("https://github.com/phR0ze/alpine-core", &repo2);
    repos.insert("https://github.com/phR0ze/cyberlinux", &repo3);
    assert!(git::clone_term_progress(&repos).is_ok());

    // Clean up
    assert!(sys::remove_all(&tmpdir).is_ok());
    println!("Git tests passed!");
    Ok(())
}

fn test_libc() -> Result<()> {
    println!("User ID: {}", user::getuid());
    Ok(())
}

fn test_crypto() -> Result<()> {
    let tmpfile = Path::new("~/.bashrc").abs()?;
    println!("Digest: {}", enc::hex::encode(sys::digest(&tmpfile)?));
    Ok(())
}

// Test setup
fn setup<T: AsRef<Path>>(path: T) -> PathBuf {
    let temp = PathBuf::from("../../temp").abs().unwrap();
    sys::mkdir(&temp).unwrap();
    temp.mash(path.as_ref())
}
