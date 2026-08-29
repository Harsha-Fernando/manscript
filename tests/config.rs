use manscript::config::{find_project_root, parse_toml, ProjectConfig};
use manscript::utils::filesystem::validate_project_name;
use std::fs;

#[test]
fn finds_toml_in_parent() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("a").join("b");
    fs::create_dir_all(&nested).unwrap();
    let cfg = parse_toml(
        r#"
name = "x"
[language]
name = "python"
version = "3.13"
"#,
    )
    .unwrap();
    assert!(cfg.framework.is_none());
    cfg.save(&dir.path().join("manscript.toml")).unwrap();
    let found = find_project_root(&nested).unwrap();
    assert_eq!(
        found.canonicalize().unwrap(),
        dir.path().canonicalize().unwrap()
    );
}

#[test]
fn rejects_bad_names() {
    assert!(validate_project_name("ok").is_ok());
    assert!(validate_project_name("../x").is_err());
    assert!(validate_project_name("has space").is_err());
}

#[test]
fn roundtrip_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manscript.toml");
    let original = ProjectConfig {
        name: "demo".into(),
        language: manscript::config::LanguageConfig {
            name: "ruby".into(),
            version: "3.4".into(),
        },
        framework: Some(manscript::config::FrameworkConfig {
            name: "rails".into(),
            version: "8.0".into(),
        }),
        environment: Default::default(),
        runtime: Default::default(),
        commands: Default::default(),
    };
    original.save(&path).unwrap();
    let loaded = ProjectConfig::load(&path).unwrap();
    assert_eq!(loaded, original);
}
