use fungus::prelude::*;

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
    assert!(sys::remove_all(&tmpdir).is_ok());

    assert!(git::clone_term_progress("https://github.com/phR0ze/cyberlinux", &tmpdir).is_ok());

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
