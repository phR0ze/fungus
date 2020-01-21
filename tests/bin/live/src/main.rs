use fungus::prelude::*;

fn main() -> Result<()> {
    println!("=====================================================================");
    println!("=                         LIVE TESTING                              =");
    println!("=====================================================================");
    println!("HOME: {:?}", user::home_dir().unwrap());
    test_tar();
    // test_git_clone_term_progress()?;
    // test_libc()?;
    // test_crypto()?;
    Ok(())
}

fn test_tar() {
    let tmpdir = setup("tar_create");
    let file1 = tmpdir.mash("file1");
    assert!(sys::remove_all(&tmpdir).is_ok());
    assert!(sys::mkdir(&tmpdir).is_ok());

    // Single file tarball
    assert!(sys::touch(file1).is_ok());
    //assert!(tar::create().is_ok());

    // assert!(sys::remove_all(&tmpdir).is_ok());
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
