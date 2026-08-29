use assert_cmd::Command;
use manscript::core::registry::default_registry;
use predicates::prelude::*;
use std::io::ErrorKind;
use std::process::Command as ProcCommand;

fn bin() -> Command {
    Command::cargo_bin("manscript").unwrap()
}

fn write_python_project(root: &std::path::Path) {
    std::fs::write(
        root.join("manscript.toml"),
        r#"
name = "ShellTest"
[language]
name = "python"
version = "3.13"
"#,
    )
    .unwrap();
}

fn make_python_environment_ready(root: &std::path::Path) {
    let bin_dir =
        root.join(".manscript/environment")
            .join(if cfg!(windows) { "Scripts" } else { "bin" });
    std::fs::create_dir_all(&bin_dir).unwrap();
    std::fs::write(
        bin_dir.join(if cfg!(windows) {
            "python.exe"
        } else {
            "python"
        }),
        "",
    )
    .unwrap();
}

#[cfg(unix)]
fn write_shell_stub(root: &std::path::Path) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let stub = root.join("shell-stub");
    std::fs::write(&stub, "#!/bin/sh\nprintf 'CHILD_PATH=%s\\n' \"$PATH\"\n").unwrap();
    let mut permissions = std::fs::metadata(&stub).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&stub, permissions).unwrap();
    stub
}

#[cfg(unix)]
fn write_failing_shell_stub(root: &std::path::Path, code: i32) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let stub = root.join("failing-shell-stub");
    std::fs::write(&stub, format!("#!/bin/sh\nexit {code}\n")).unwrap();
    let mut permissions = std::fs::metadata(&stub).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&stub, permissions).unwrap();
    stub
}

fn has_bin(name: &str) -> bool {
    command_succeeds(name, &["--version"])
}

fn command_succeeds(name: &str, args: &[&str]) -> bool {
    match ProcCommand::new(name).args(args).output() {
        Err(e) if e.kind() == ErrorKind::NotFound => false,
        Err(_) => false,
        Ok(output) => output.status.success(),
    }
}

fn system_runtime_available(language: &str, version: &str) -> bool {
    default_registry()
        .providers()
        .iter()
        .find(|provider| provider.id() == "system")
        .and_then(|provider| provider.detect(language, version).ok().flatten())
        .is_some()
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
    let ok = ProcCommand::new(compiler)
        .args(["-o", out, source_name])
        .current_dir(dir.path())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    ok && dir.path().join(out).is_file()
}

#[test]
fn help_lists_commands() {
    bin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO"))
        .stdout(predicate::str::contains("███"))
        .stdout(predicate::str::contains("Start here"))
        .stdout(predicate::str::contains("Create a new project"))
        .stdout(predicate::str::contains(
            "Add ManScript to an existing project",
        ))
        .stdout(predicate::str::contains("Prepare a cloned project"))
        .stdout(predicate::str::contains("Use your project"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("shell"))
        .stdout(predicate::str::contains("completions"))
        .stdout(predicate::str::contains("╭").not());
}

#[test]
fn shell_help_prints() {
    bin()
        .args(["shell", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO"))
        .stdout(predicate::str::contains("shell"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn shell_requires_a_manscript_project() {
    let dir = tempfile::tempdir().unwrap();

    bin()
        .current_dir(dir.path())
        .arg("shell")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("could not find `manscript.toml`"))
        .stderr(predicate::str::contains("manscript init"))
        .stderr(predicate::str::contains("manscript create"));
}

#[test]
fn shell_requires_a_ready_environment() {
    let dir = tempfile::tempdir().unwrap();
    write_python_project(dir.path());

    bin()
        .current_dir(dir.path())
        .arg("shell")
        .assert()
        .failure()
        .stderr(predicate::str::contains("environment is not ready"))
        .stderr(predicate::str::contains("manscript setup"));
}

#[cfg(unix)]
#[test]
fn shell_prepends_project_bin_and_preserves_parent_environment() {
    let dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    write_python_project(dir.path());
    let project_root = dir.path().canonicalize().unwrap();
    let environment_bin = project_root.join(".manscript/environment/bin");
    std::fs::create_dir_all(&environment_bin).unwrap();
    std::fs::write(environment_bin.join("python"), "").unwrap();
    let stub = write_shell_stub(dir.path());
    let existing_path = std::env::join_paths(["/system-one", "/system-two"]).unwrap();
    let expected_path = std::env::join_paths([
        environment_bin.as_path(),
        "/system-one".as_ref(),
        "/system-two".as_ref(),
    ])
    .unwrap();
    let parent_path = std::env::var_os("PATH");

    bin()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("PATH", &existing_path)
        .env("MANSCRIPT_SHELL", &stub)
        .arg("shell")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "CHILD_PATH={}",
            expected_path.to_string_lossy()
        )))
        .stdout(predicate::str::contains(
            "Development shell closed. Your original terminal environment is unchanged. Goodbye.",
        ));

    assert_eq!(std::env::var_os("PATH"), parent_path);
    assert!(!home.path().join(".bashrc").exists());
    assert!(!home.path().join(".zshrc").exists());
    assert!(!home.path().join(".profile").exists());
}

#[cfg(unix)]
#[test]
fn shell_reports_and_preserves_nonzero_exit_code() {
    let dir = tempfile::tempdir().unwrap();
    write_python_project(dir.path());
    let environment_bin = dir.path().join(".manscript/environment/bin");
    std::fs::create_dir_all(&environment_bin).unwrap();
    std::fs::write(environment_bin.join("python"), "").unwrap();
    let stub = write_failing_shell_stub(dir.path(), 42);

    bin()
        .current_dir(dir.path())
        .env("MANSCRIPT_SHELL", stub)
        .arg("shell")
        .assert()
        .code(42)
        .stdout(predicate::str::contains(
            "Development shell closed with exit code 42",
        ))
        .stdout(predicate::str::contains("Goodbye.").not());
}

#[test]
fn install_requires_a_ready_environment() {
    let dir = tempfile::tempdir().unwrap();
    write_python_project(dir.path());

    bin()
        .current_dir(dir.path())
        .arg("install")
        .assert()
        .failure()
        .stderr(predicate::str::contains("project environment is not ready"))
        .stderr(predicate::str::contains("manscript setup"));
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
        .stdout(predicate::str::contains("███"))
        .stdout(predicate::str::contains("INFO"))
        .stdout(predicate::str::contains(format!(
            "ManScript  v{}",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn init_and_install_use_consistent_command_chrome() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .current_dir(dir.path())
        .args(["init", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO  Init"))
        .stdout(predicate::str::contains("Project"))
        .stdout(predicate::str::contains("DONE"))
        .stdout(predicate::str::contains("manscript setup"));

    make_python_environment_ready(dir.path());
    bin()
        .current_dir(dir.path())
        .arg("install")
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO  Install"))
        .stdout(predicate::str::contains("Environment"))
        .stdout(predicate::str::contains("DONE"))
        .stdout(predicate::str::contains("manscript run"));
}

#[test]
fn env_and_empty_test_show_status_and_next_action() {
    let dir = tempfile::tempdir().unwrap();
    write_python_project(dir.path());

    bin()
        .current_dir(dir.path())
        .arg("env")
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO  Environment"))
        .stdout(predicate::str::contains("WARN"))
        .stdout(predicate::str::contains("manscript setup"));

    make_python_environment_ready(dir.path());
    bin()
        .current_dir(dir.path())
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO  Test"))
        .stdout(predicate::str::contains("No `test` command is configured"))
        .stdout(predicate::str::contains("DONE"));
}

#[test]
fn setup_uses_a_numbered_three_step_pipeline() {
    if cfg!(windows) || !compiler_can_link("cc", "t.c", "int main(void) { return 0; }\n") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("manscript.toml"),
        r#"
name = "SetupTest"
[language]
name = "c"
version = "any"
"#,
    )
    .unwrap();

    bin()
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .args(["setup", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO  Setup"))
        .stdout(predicate::str::contains("[1/3]"))
        .stdout(predicate::str::contains("[2/3]"))
        .stdout(predicate::str::contains("[3/3]"))
        .stdout(predicate::str::contains("DONE"));
}

#[test]
fn completions_zsh_lists_commands() {
    bin()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_manscript"))
        .stdout(predicate::str::contains("autoload -Uz compinit"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("django"));
}

#[test]
fn doctor_runs() {
    bin()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO  Doctor"))
        .stdout(predicate::str::contains(
            "Check runtimes, toolchains, and the current project",
        ))
        .stdout(predicate::str::contains("Platform"));
}

#[test]
fn unknown_framework_fails() {
    bin()
        .args(["create", "cobol", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "not a supported framework or language",
        ))
        .stderr(predicate::str::contains("python"))
        .stderr(predicate::str::contains("django"));
}

#[test]
fn create_none_is_not_a_cli_alias() {
    bin()
        .args(["create", "none", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "not a supported framework or language",
        ));
}

#[test]
fn unknown_subcommand_is_friendly() {
    bin()
        .arg("docter")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "`docter` is not a ManScript command",
        ))
        .stderr(predicate::str::contains("manscript --help"))
        .stderr(predicate::str::contains("manscript doctor"));
}

#[test]
fn short_unknown_subcommand_does_not_suggest_an_unrelated_command() {
    bin()
        .arg("h")
        .assert()
        .failure()
        .stderr(predicate::str::contains("`h` is not a ManScript command"))
        .stderr(predicate::str::contains("Did you mean").not())
        .stderr(predicate::str::contains("manscript --help"));
}

#[test]
fn completions_without_a_shell_explains_the_optional_argument() {
    bin()
        .arg("completions")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "`manscript completions` needs the name of your shell",
        ))
        .stderr(predicate::str::contains(
            "bash, zsh, fish, powershell, elvish",
        ))
        .stderr(predicate::str::contains("manscript completions zsh"))
        .stderr(predicate::str::contains("This command is optional"));
}

#[test]
fn clap_errors_use_the_manscript_error_layout() {
    bin()
        .args(["completions", "unsupported-shell"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "ManScript could not complete that request",
        ))
        .stderr(predicate::str::contains("invalid value"))
        .stderr(predicate::str::contains("--help"));
}

#[test]
fn no_color_help_contains_no_escape_sequences() {
    bin()
        .env("NO_COLOR", "1")
        .env("TERM", "dumb")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Start here"))
        .stdout(predicate::str::contains("\u{1b}").not());
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
    if cfg!(windows) {
        return;
    }
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
    if cfg!(windows) {
        return;
    }
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

#[test]
fn create_go_language_only_when_go_exists() {
    if !system_runtime_available("go", "1.25") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    bin()
        .current_dir(dir.path())
        .args(["create", "go", "hello", "-y"])
        .assert()
        .success();
    assert!(dir.path().join("hello/go.mod").is_file());
    assert!(dir.path().join("hello/main.go").is_file());
    bin()
        .current_dir(dir.path().join("hello"))
        .arg("run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from ManScript (Go"));
}

#[test]
fn create_rust_language_only_when_rust_exists() {
    if !system_runtime_available("rust", "stable") || !has_bin("cargo") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    bin()
        .current_dir(dir.path())
        .args(["create", "rust", "hello", "-y"])
        .assert()
        .success();
    assert!(dir.path().join("hello/Cargo.toml").is_file());
    assert!(dir.path().join("hello/src/main.rs").is_file());
    bin()
        .current_dir(dir.path().join("hello"))
        .arg("run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from ManScript (Rust"));
}

#[test]
fn create_php_language_only_when_php_and_composer_exist() {
    if !system_runtime_available("php", "8.4") || !has_bin("composer") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    bin()
        .current_dir(dir.path())
        .args(["create", "php", "hello", "-y"])
        .assert()
        .success();
    assert!(dir.path().join("hello/composer.json").is_file());
    assert!(dir.path().join("hello/main.php").is_file());
    bin()
        .current_dir(dir.path().join("hello"))
        .arg("run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from ManScript (PHP"));
}

#[test]
fn create_csharp_language_only_when_dotnet_exists() {
    if !has_bin("dotnet") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    bin()
        .current_dir(dir.path())
        .args(["create", "csharp", "hello", "-y"])
        .assert()
        .success();
    assert!(dir.path().join("hello/hello.csproj").is_file());
    assert!(dir.path().join("hello/Program.cs").is_file());
    bin()
        .current_dir(dir.path().join("hello"))
        .arg("run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from ManScript (C#"));
}
