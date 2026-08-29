use std::collections::HashMap;
use std::path::PathBuf;

use crate::adapters::toolchain;
use crate::adapters::traits::{ConfirmPolicy, DoctorCheck, LanguageAdapter};
use crate::core::environment::{Environment, ShellEnvironment};
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::core::runtime::Runtime;
use crate::process::{split_command_line, Executor, PreparedCommand};
use crate::utils::filesystem::ensure_dir;
use crate::utils::platform::exe_name;

pub mod plain;

pub struct PhpAdapter;

impl LanguageAdapter for PhpAdapter {
    fn id(&self) -> &'static str {
        "php"
    }

    fn package_manager_name(&self) -> &'static str {
        "composer"
    }

    fn default_environment_manager(&self) -> &'static str {
        "composer"
    }

    fn create_environment(
        &self,
        project: &Project,
        runtime: &Runtime,
        confirm: ConfirmPolicy,
    ) -> Result<Environment> {
        ensure_dir(&composer_home(project))?;

        let composer = ensure_composer(project, runtime, confirm)?;
        let environment = toolchain::create_toolchain_env(
            project,
            &[
                ("php", runtime.executable.as_path()),
                ("composer", composer.as_path()),
            ],
        )?;
        write_windows_composer_wrapper(project, &composer)?;
        Ok(environment)
    }

    fn environment_ready(&self, project: &Project) -> bool {
        toolchain::toolchain_ready(project, &["php", "composer"]) && composer_home(project).is_dir()
    }

    fn install_dependencies(&self, project: &Project) -> Result<()> {
        if !project.root.join("composer.json").is_file() {
            return Ok(());
        }
        run_composer(project, vec!["install".into(), "--no-interaction".into()])
    }

    fn install_packages(&self, project: &Project, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        validate_packages(packages)?;
        let mut args = vec!["require".into(), "--no-interaction".into()];
        args.extend(packages.iter().cloned());
        run_composer(project, args)
    }

    fn resolve_command(
        &self,
        project: &Project,
        command: &str,
        extra_args: &[String],
    ) -> Result<PreparedCommand> {
        if !self.environment_ready(project) {
            return Err(ManscriptError::EnvironmentNotReady(
                project.environment_dir(),
            ));
        }
        prepare_php_command(project, command, extra_args)
    }

    fn shell_environment(&self, project: &Project) -> Result<ShellEnvironment> {
        Ok(ShellEnvironment {
            path_prepend: php_path(project),
            extra_env: composer_env(project),
        })
    }

    fn doctor_checks(&self) -> Vec<DoctorCheck> {
        vec![
            tool_check(
                "PHP",
                "php",
                "Run `manscript setup` to install isolated PHP 8.4 with mise, or make a compatible `php` available on PATH.",
            ),
            tool_check(
                "PHP package manager",
                "composer",
                "Install Composer from https://getcomposer.org/download/ and make the verified `composer` executable available on PATH. ManScript will not run an unverified bootstrap installer.",
            ),
        ]
    }
}

fn prepare_php_command(
    project: &Project,
    command: &str,
    extra_args: &[String],
) -> Result<PreparedCommand> {
    let mut argv = split_command_line(command)?;
    argv.extend(extra_args.iter().cloned());
    let program_name = argv.remove(0);
    validate_tool_name(&program_name)?;

    let program = match program_name.as_str() {
        "php" => toolchain::env_tool(project, "php")?,
        "composer" => {
            let composer = composer_from_environment(project)?;
            if is_composer_phar(&composer) {
                argv.insert(0, composer.display().to_string());
                toolchain::env_tool(project, "php")?
            } else {
                composer
            }
        }
        name => {
            let candidate = project.root.join("vendor").join("bin").join(exe_name(name));
            if !candidate.is_file() {
                return Err(ManscriptError::InvalidCommand(format!(
                    "could not find `{name}` in the PHP toolchain or `vendor/bin`; run `manscript setup` and check the configured command"
                )));
            }
            candidate
        }
    };

    Ok(PreparedCommand {
        program,
        args: argv,
        cwd: project.root.clone(),
        extra_env: composer_env(project),
        path_prepend: php_path(project),
    })
}

fn run_composer(project: &Project, args: Vec<String>) -> Result<()> {
    let composer = composer_from_environment(project)?;
    let (program, args) = if is_composer_phar(&composer) {
        let mut composer_args = vec![composer.display().to_string()];
        composer_args.extend(args);
        (toolchain::env_tool(project, "php")?, composer_args)
    } else {
        (composer, args)
    };
    let prepared = PreparedCommand {
        program,
        args,
        cwd: project.root.clone(),
        extra_env: composer_env(project),
        path_prepend: php_path(project),
    };
    Executor::new().run_status(prepared)
}

fn composer_from_environment(project: &Project) -> Result<PathBuf> {
    toolchain::env_tool(project, "composer").map_err(|_| composer_missing())
}

fn composer_missing() -> ManscriptError {
    ManscriptError::Message(
        "Composer is required for this PHP project, but a verified Composer executable is not available in the project environment.\n\nRun `manscript setup` to retry the verified project-local installation. You can also install Composer using the official instructions at https://getcomposer.org/download/ and run setup again."
            .into(),
    )
}

fn find_composer() -> Option<PathBuf> {
    which::which("composer").ok()
}

fn ensure_composer(
    project: &Project,
    runtime: &Runtime,
    confirm: ConfirmPolicy,
) -> Result<PathBuf> {
    if let Some(composer) = find_composer() {
        return Ok(composer);
    }
    let composer = project.environment_bin_dir().join("composer.phar");
    if composer.is_file() {
        return Ok(composer);
    }
    if !confirm.confirm(
        "Composer is not installed. ManScript can verify Composer's official installer and place Composer only inside this project (no sudo). Continue?",
    )? {
        return Err(ManscriptError::Cancelled);
    }

    let bootstrap = project.root.join(".manscript").join("bootstrap");
    ensure_dir(&bootstrap)?;
    ensure_dir(&project.environment_bin_dir())?;
    let installer = bootstrap.join("composer-setup.php");
    let signature = bootstrap.join("composer-installer.sha384");
    crate::runtime::download::download_to_file("https://getcomposer.org/installer", &installer)?;
    crate::runtime::download::download_to_file(
        "https://composer.github.io/installer.sig",
        &signature,
    )?;

    verify_composer_installer(runtime, project, &installer, &signature)?;
    let install = PreparedCommand {
        program: runtime.executable.clone(),
        args: vec![
            installer.display().to_string(),
            format!("--install-dir={}", project.environment_bin_dir().display()),
            "--filename=composer.phar".into(),
            "--quiet".into(),
        ],
        cwd: project.root.clone(),
        extra_env: HashMap::new(),
        path_prepend: Vec::new(),
    };
    Executor::new().run_status(install)?;
    if !composer.is_file() {
        return Err(composer_missing());
    }
    make_composer_executable(&composer)?;
    let _ = std::fs::remove_file(installer);
    let _ = std::fs::remove_file(signature);
    Ok(composer)
}

fn verify_composer_installer(
    runtime: &Runtime,
    project: &Project,
    installer: &std::path::Path,
    signature: &std::path::Path,
) -> Result<()> {
    let verification = PreparedCommand {
        program: runtime.executable.clone(),
        args: vec![
            "-r".into(),
            "if (!hash_equals(trim(file_get_contents($argv[2])), hash_file('sha384', $argv[1]))) { fwrite(STDERR, 'Composer installer signature mismatch.\\n'); exit(1); }".into(),
            installer.display().to_string(),
            signature.display().to_string(),
        ],
        cwd: project.root.clone(),
        extra_env: HashMap::new(),
        path_prepend: Vec::new(),
    };
    Executor::new().run_status(verification)
}

fn is_composer_phar(path: &std::path::Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "phar")
}

#[cfg(unix)]
fn make_composer_executable(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_composer_executable(_path: &std::path::Path) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
fn write_windows_composer_wrapper(project: &Project, composer: &std::path::Path) -> Result<()> {
    if !is_composer_phar(composer) {
        return Ok(());
    }
    let composer = composer.to_string_lossy().replace('%', "%%");
    crate::utils::filesystem::write_file(
        &project.environment_bin_dir().join("composer.cmd"),
        &format!("@echo off\r\n\"%~dp0php.cmd\" \"{composer}\" %*\r\n"),
    )
}

#[cfg(not(windows))]
fn write_windows_composer_wrapper(_project: &Project, _composer: &std::path::Path) -> Result<()> {
    Ok(())
}

fn composer_home(project: &Project) -> PathBuf {
    project.root.join(".manscript").join("composer-home")
}

fn vendor_bin(project: &Project) -> PathBuf {
    project.root.join("vendor").join("bin")
}

fn composer_env(project: &Project) -> HashMap<String, String> {
    HashMap::from([(
        "COMPOSER_HOME".into(),
        composer_home(project).display().to_string(),
    )])
}

fn php_path(project: &Project) -> Vec<PathBuf> {
    vec![project.environment_bin_dir(), vendor_bin(project)]
}

fn validate_tool_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.starts_with('-')
        || name.contains('/')
        || name.contains('\\')
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return Err(ManscriptError::InvalidCommand(
            "PHP command names must be plain executable names without paths".into(),
        ));
    }
    Ok(())
}

fn validate_packages(packages: &[String]) -> Result<()> {
    if packages.iter().any(|package| {
        package.is_empty()
            || package.starts_with('-')
            || package.chars().any(char::is_whitespace)
            || package.chars().any(char::is_control)
    }) {
        return Err(ManscriptError::InvalidCommand(
            "Composer package specifications must be non-empty argv values and must not begin with `-`"
                .into(),
        ));
    }
    Ok(())
}

fn tool_check(
    group: &'static str,
    program: &'static str,
    recommendation: &'static str,
) -> DoctorCheck {
    match which::which(program) {
        Ok(path) => {
            let version = crate::runtime::system::probe_version(&path, &["--version"])
                .ok()
                .flatten()
                .unwrap_or_else(|| path.display().to_string());
            DoctorCheck {
                group: group.into(),
                ok: true,
                label: format!("{program} {version} detected"),
                recommendation: None,
            }
        }
        Err(_) => DoctorCheck {
            group: group.into(),
            ok: false,
            label: format!("{program} not found"),
            recommendation: Some(recommendation.into()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_like_vendor_commands() {
        for name in ["../tool", "/tmp/tool", "vendor/bin/tool", "-tool"] {
            assert!(validate_tool_name(name).is_err());
        }
        assert!(validate_tool_name("phpunit").is_ok());
    }

    #[test]
    fn rejects_option_injection_as_a_package() {
        assert!(validate_packages(&["--working-dir=/tmp".into()]).is_err());
        assert!(validate_packages(&["vendor/package:^1.2".into()]).is_ok());
    }

    #[test]
    fn composer_blocker_explains_secure_recovery() {
        let message = composer_missing().to_string();
        assert!(message.contains("verified"));
        assert!(message.contains("getcomposer.org/download"));
        assert!(message.contains("manscript setup"));
    }
}
