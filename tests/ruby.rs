use manscript::core::registry::default_registry;
use manscript::core::runtime::version_matches;

#[test]
fn rails_is_ruby() {
    let r = default_registry();
    assert_eq!(r.framework("rails").unwrap().language(), "ruby");
    assert_eq!(r.framework("sinatra").unwrap().language(), "ruby");
    assert!(r.framework("ruby").unwrap().language_only());
}

#[test]
fn ruby_version_prefix() {
    assert!(version_matches("3.4.1", "3.4"));
    assert!(!version_matches("3.3.0", "3.4"));
}
