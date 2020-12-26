use fungus::prelude::*;

#[test]
fn test_use_syntax() {
    let home = user::home_dir().unwrap();
    assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
}
