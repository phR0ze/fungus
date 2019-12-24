use fungus::presys::*;

#[test]
fn test_use_syntax() {
    let home = env::var("HOME").unwrap();
    assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
}
