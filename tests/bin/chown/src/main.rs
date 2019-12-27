use fungus::presys::*;
use fungus::user;

fn main() {
    let tmpdir = PathBuf::from("../../temp");
    let curr = user::current();

    println!("=====================================================================");
    println!("=                      LIVE TESTING                                 =");
    println!("=====================================================================");
    println!("CWD:       {:?}", env::current_dir().unwrap());
    println!("User Home: {:?}", user::home().unwrap());
    println!("User RUID: {:?}, {:?}", user::getruid(), curr.ruid());
    println!("User RGID: {:?}, {:?}", user::getrgid(), curr.rgid());
    println!("User  UID: {:?}, {:?}", user::getuid(), curr.uid());
    println!("User EUID: {:?}, {:?}", user::geteuid(), curr.euid());
    println!("User  GID: {:?}, {:?}", user::getgid(), curr.gid());
    println!("User EGID: {:?}, {:?}", user::getegid(), curr.egid());

    // Tests
    assert_ne!(user::home().unwrap(), PathBuf::from("/root"));
    test_chown(&tmpdir);
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

    assert!(sys::touch(&file1).is_ok());
    //assert!(sys::chown(&file1, user::getuid(), user::getgid()).is_ok());

    //cleanup(tmpdir);
}
