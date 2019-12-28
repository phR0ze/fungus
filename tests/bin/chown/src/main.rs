use fungus::presys::*;
use fungus::user;

fn main() {
    let tmpdir = PathBuf::from("../../temp");
    let user = user::current().unwrap();
    if !user.is_root() {
        panic!("need to be root to run these tests");
    }

    println!("=====================================================================");
    println!("=                      LIVE TESTING                                 =");
    println!("=====================================================================");
    println!("CWD:       {:?}", env::current_dir().unwrap());
    println!("User Name: {:?}", user.name);
    println!("User Home: {:?}, {:?}", user::home().unwrap(), user.home);
    println!("User  UID: {:?}, {:?}", user::getuid(), user.uid);
    println!("User  GID: {:?}, {:?}", user::getgid(), user.gid);
    println!("Real Name: {:?}", user.realname);
    println!("Real Home: {:?}", user.realhome);
    println!("User RUID: {:?}", user.ruid);
    println!("User RGID: {:?}", user.rgid);

    // Tests
    assert_ne!(user::home().unwrap(), PathBuf::from("/root"));
    test_chown(&tmpdir);
    test_sudo(&tmpdir);
}

fn setup<T: AsRef<Path>>(path: T, target: &str) -> PathBuf {
    let tmpdir = path.as_ref().mash(target);
    assert!(sys::remove_all(&tmpdir).is_ok());
    assert!(sys::mkdir(&tmpdir).is_ok());
    tmpdir
}

fn cleanup<T: AsRef<Path>>(path: T) {
    assert!(sys::remove_all(path).is_ok());
}

fn test_chown<T: AsRef<Path>>(path: T) {
    let tmpdir = setup(path, "bin_chown");
    let file1 = tmpdir.mash("file1");

    // Get the real user behind the sudo mask
    let (ruid, rgid) = user::getrids(0, 0);
    assert_ne!(ruid, 0);
    assert_ne!(rgid, 0);

    // Single file
    {
        // Create a new file owned by root via sudo
        assert!(sys::touch(&file1).is_ok());
        assert_eq!(file1.uid().unwrap(), 0);
        assert_eq!(file1.gid().unwrap(), 0);

        // Chown the file to be owned by the real use
        assert!(sys::chown(&file1, ruid, rgid).is_ok());
        assert_eq!(file1.uid().unwrap(), ruid);
        assert_eq!(file1.gid().unwrap(), rgid);
    }

    // Recurse
    {
        assert!(sys::chown(&tmpdir, 0, 0).is_ok());
        assert_eq!(tmpdir.uid().unwrap(), 0);
        assert_eq!(tmpdir.gid().unwrap(), 0);
        assert_eq!(file1.uid().unwrap(), 0);
        assert_eq!(file1.gid().unwrap(), 0);

        assert!(sys::chown(&tmpdir, ruid, rgid).is_ok());
        assert_eq!(tmpdir.uid().unwrap(), ruid);
        assert_eq!(tmpdir.gid().unwrap(), rgid);
        assert_eq!(file1.uid().unwrap(), ruid);
        assert_eq!(file1.gid().unwrap(), rgid);
    }

    cleanup(tmpdir);
}

fn test_sudo<T: AsRef<Path>>(path: T) {
    // Get the real user behind the sudo mask
    let (ruid, rgid) = user::getrids(0, 0);
    assert_ne!(ruid, 0);
    assert_ne!(rgid, 0);

    // Now create dir and file
    assert!(user::pause_sudo().is_ok());
    let tmpdir = setup(path, "bin_sudo");
    let file1 = tmpdir.mash("file1");
    assert!(sys::touch(&file1).is_ok());

    // Create a new file owned by root via sudo
    assert_eq!(tmpdir.uid().unwrap(), ruid);
    assert_eq!(tmpdir.gid().unwrap(), rgid);
    assert_eq!(file1.uid().unwrap(), ruid);
    assert_eq!(file1.gid().unwrap(), rgid);

    // Now raise sudo and create a new file
    assert!(user::sudo().is_ok());
    let file2 = tmpdir.mash("file2");
    assert!(sys::touch(&file2).is_ok());
    assert_eq!(tmpdir.uid().unwrap(), ruid);
    assert_eq!(tmpdir.gid().unwrap(), rgid);
    assert_eq!(file1.uid().unwrap(), ruid);
    assert_eq!(file1.gid().unwrap(), rgid);
    assert_eq!(file2.uid().unwrap(), 0);
    assert_eq!(file2.gid().unwrap(), 0);

    // Chown files so we can delete them as not root
    assert!(sys::chown(&tmpdir, ruid, rgid).is_ok());
    assert!(user::drop_sudo().is_ok());
    assert!(user::sudo().is_err());
    let file3 = tmpdir.mash("file3");
    assert!(sys::touch(&file3).is_ok());
    assert_eq!(file3.uid().unwrap(), ruid);
    assert_eq!(file3.gid().unwrap(), rgid);

    cleanup(&tmpdir);
    assert_eq!(tmpdir.exists(), false);
}
