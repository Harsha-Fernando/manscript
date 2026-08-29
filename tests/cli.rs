use assert_cmd::Command;
use predicates::prelude::*;
use std::io::ErrorKind;
use std::process::Command as ProcCommand;

fn bin() -> Command {
    Command::cargo_bin("manscript").unwrap()
}

fn has_bin(name: &str) -> bool {
    match ProcCommand::new(name).arg("--version").output() {
        Err(e) if e.kind() == ErrorKind::NotFound => false,
        Err(_) => false,
        Ok(_) => true,
    }
}

/// Git-for-Windows and similar stubs ship `cc.exe` without `cc1`. `--version` is not enough.
fn compiler_can_link(compiler: &str, source_name: &str, source: &str) -> bool {
    let Ok(dir) = tempfile::tempdir() else {
        return false;
    };
    if std::fs::write(dir.path().join(source_name), source).is_err() {
        return false;
    }
    let out = if cfg!(windows) { "t.exe" } else { "t" };
    ProcCommand::new(compiler)
        .args(["-o", out, source_name])
        .current_dir(dir.path())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn help_lists_commands() {
    bin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO"))
        .stdout(predicate::str::contains("Available commands:"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("╭").not());
}

#[test]
fn create_help_is_laravel_style() {
    bin()
        .args(["create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("╭").not());
}

#[test]
fn version_prints() {
    bin()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO"))
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn doctor_runs() {
    bin()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("ManScript Doctor"))
        .stdout(predicate::str::contains("Platform"));
}

#[test]
fn unknown_framework_fails() {
    bin()
        .args(["create", "cobol", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a framework ManScript knows"))
        .stderr(predicate::str::contains("python"))
        .stderr(predicate::str::contains("django"));
}

#[test]
fn create_none_is_not_a_cli_alias() {
    bin()
        .args(["create", "none", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a framework ManScript knows"));
}

#[test]
fn unknown_subcommand_is_friendly() {
    bin()
        .arg("docter")
        .assert()
        .failure()
        .stderr(predicate::str::contains("There is no command like"))
        .stderr(predicate::str::contains("manscript -h"))
        .stderr(predicate::str::contains("manscript doctor"));
}

#[test]
fn invalid_project_name_fails() {
    bin()
        .args(["create", "django", "../escape"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a valid project name"));
}

#[test]
fn language_only_run_does_not_pretend_to_be_a_server() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("manscript.toml"),
        r#"
name = "x"
[language]
name = "python"
version = "3.13"
[commands]
run = "python main.py"
"#,
    )
    .unwrap();
    bin()
        .current_dir(dir.path())
        .arg("run")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Starting development server").not())
        .stdout(predicate::str::contains("from zero to running").not())
        .stderr(predicate::str::contains("Starting development server").not())
        .stderr(predicate::str::contains("environment is not ready"));
}

#[test]
fn in_project_language_only_create_does_not_start_a_new_folder() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("manscript.toml"),
        r#"
name = "x"
[language]
name = "python"
version = "3.13"
"#,
    )
    .unwrap();
    bin()
        .current_dir(dir.path())
        .args(["create", "blog"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no apps or modules to generate"));
    assert!(!dir.path().join("blog").exists());
}

#[test]
fn create_c_language_only_when_cc_exists() {
    if !compiler_can_link("cc", "t.c", "int main(void) { return 0; }\n") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    bin()
        .current_dir(dir.path())
        .args(["create", "c", "hello", "-y"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ready"));
    assert!(dir.path().join("hello/main.c").is_file());
    let toml = std::fs::read_to_string(dir.path().join("hello/manscript.toml")).unwrap();
    assert!(toml.contains("name = \"c\""));
    assert!(!toml.contains("[framework]"));
    bin()
        .current_dir(dir.path().join("hello"))
        .arg("run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from ManScript (C"));
}

#[test]
fn create_cpp_language_only_when_cxx_exists() {
    if !compiler_can_link("c++", "t.cpp", "int main() { return 0; }\n") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    bin()
        .current_dir(dir.path())
        .args(["create", "cpp", "hello", "-y"])
        .assert()
        .success();
    assert!(dir.path().join("hello/main.cpp").is_file());
    bin()
        .current_dir(dir.path().join("hello"))
        .arg("run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from ManScript (C++"));
}

#[test]
fn create_java_language_only_when_javac_exists() {
    if !has_bin("javac") || !has_bin("java") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    bin()
        .current_dir(dir.path())
        .args(["create", "java", "hello", "-y"])
        .assert()
        .success();
    assert!(dir.path().join("hello/Main.java").is_file());
    bin()
        .current_dir(dir.path().join("hello"))
        .arg("run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from ManScript (Java"));
}
