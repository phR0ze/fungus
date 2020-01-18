use fungus::prelude::*;

fn main() -> Result<()> {
    println!("=====================================================================");
    println!("=                         LIVE TESTING                              =");
    println!("=====================================================================");
    println!("HOME: {:?}", user::home_dir().unwrap());
    test_git_clone_with_progress()?;
    test_libc()?;
    test_crypto()?;
    Ok(())
}

fn test_git_clone_with_progress() -> Result<()> {
    let tmpdir = setup("git_clone_with_progress");
    assert!(sys::remove_all(&tmpdir).is_ok());

    let mut cb = git::RemoteCallbacks::new();
    cb.transfer_progress(|stats| {
        println!("Total Objects: {:?}", stats.total_objects());
        println!("Indexed Objects: {:?}", stats.indexed_objects());
        println!("Received Objects: {:?}", stats.received_objects());
        println!("Local Objects: {:?}", stats.local_objects());
        println!("Total Deltas: {:?}", stats.total_deltas());
        println!("Indexed Deltas: {:?}", stats.indexed_deltas());
        println!("Received Bytes: {:?}", stats.received_bytes());
        true
    });
    let mut fo = git::FetchOptions::new();
    fo.remote_callbacks(cb);

    let mut co = git::CheckoutBuilder::new();
    co.progress(|path, cur, total| {
        //
    });

    // Clone repo 1
    assert_eq!(repo1file.exists(), false);
    assert!(git::clone_with_progress("https://github.com/phR0ze/alpine-base.git", &repo1, fo, co).is_ok());

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
