use manscript::adapters::toolchain::compile_to_app;
use manscript::core::project::Project;
use manscript::core::registry::default_registry;
use manscript::process::executor::split_command_line;

#[test]
fn c_cpp_java_are_language_only() {
    let r = default_registry();
    for id in ["c", "cpp", "java"] {
        let fw = r.framework(id).unwrap();
        assert!(fw.language_only());
        assert_eq!(fw.language(), id);
        assert!(fw.generators().is_empty());
        assert!(r.frameworks_for_language(id).is_empty());
    }
}

#[test]
fn java_run_command_splits_without_a_shell() {
    let argv = split_command_line("java -cp . Main").unwrap();
    assert_eq!(argv, ["java", "-cp", ".", "Main"]);
}

#[test]
fn compile_rejects_path_in_source_name() {
    let dir = tempfile::tempdir().unwrap();
    let project = Project {
        root: dir.path().to_path_buf(),
        config: manscript::adapters::traits::default_project_config(
            "x",
            "c",
            "any",
            None,
            "toolchain",
            Default::default(),
        ),
    };
    assert!(compile_to_app(&project, "cc", "../x.c").is_err());
    assert!(compile_to_app(&project, "cc", "sub/main.c").is_err());
}
