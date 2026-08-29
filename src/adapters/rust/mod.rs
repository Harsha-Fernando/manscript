use std::collections::HashMap;
use std::path::Path;

use crate::adapters::toolchain;
use crate::adapters::traits::{DoctorCheck, LanguageAdapter};
use crate::core::environment::{Environment, ShellEnvironment};
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::core::runtime::Runtime;
use crate::process::{split_command_line, Executor, PreparedCommand};
use crate::utils::filesystem::ensure_dir;
use crate::utils::platform::exe_name;

pub mod plain;

pub struct RustAdapter;

impl LanguageAdapter for RustAdapter {
    fn id(&self) -> &'static str {
        "rust"
    }

    fn package_manager_name(&self) -> &'static str {
        "cargo"
    }

    fn default_environment_manager(&self) -> &'static str {
        "toolchain"
    }

    fn create_environment(
        &self,
        project: &Project,
        runtime: &Runtime,
        _confirm: crate::adapters::traits::ConfirmPolicy,
    ) -> Result<Environment> {
        let cargo = toolchain::find_cargo_for_rustc(&runtime.executable)?;
        let environment = toolchain::create_toolchain_env(
            project,
            &[
                ("rustc", runtime.executable.as_path()),
                ("cargo", cargo.as_path()),
            ],
        )?;
        for directory in rust_environment_directories(&project.root) {
            ensure_dir(&directory)?;
        }
        Ok(environment)
    }

    fn environment_ready(&self, project: &Project) -> bool {
        toolchain::toolchain_ready(project, &["rustc", "cargo"])
    }

    fn install_dependencies(&self, project: &Project) -> Result<()> {
        if !project.root.join("Cargo.toml").is_file() {
            return Ok(());
        }
        run_cargo(project, &["fetch".into()])
    }

    fn install_packages(&self, project: &Project, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        validate_packages(packages)?;
        let mut args = vec!["add".into()];
        args.extend(packages.iter().cloned());
        run_cargo(project, &args)
    }

    fn resolve_command(
        &self,
        project: &Project,
        command: &str,
        extra_args: &[String],
    ) -> Result<PreparedCommand> {
        prepare_rust_command(project, command, extra_args)
    }

    fn shell_environment(&self, project: &Project) -> Result<ShellEnvironment> {
        if !self.environment_ready(project) {
            return Err(ManscriptError::EnvironmentNotReady(
                project.environment_dir(),
            ));
        }
        Ok(ShellEnvironment {
            path_prepend: vec![project.environment_bin_dir()],
            extra_env: rust_environment(&project.root),
        })
    }

    fn doctor_checks(&self) -> Vec<DoctorCheck> {
        vec![
            rust_tool_check(
                "Rust",
                "rustc",
                "Run `manscript setup` to install an isolated Rust toolchain with mise, or install Rust so `rustc` is on PATH.",
            ),
            rust_tool_check(
                "Rust package manager",
                "cargo",
                "Install Cargo so `cargo` is alongside `rustc` or on PATH.",
            ),
        ]
    }
}

fn rust_environment(project_root: &Path) -> HashMap<String, String> {
    let environment_root = project_root.join(".manscript").join("environment");
    HashMap::from([
        (
            "CARGO_HOME".into(),
            environment_root.join("cargo-home").display().to_string(),
        ),
        (
            "CARGO_TARGET_DIR".into(),
            environment_root.join("cargo-target").display().to_string(),
        ),
    ])
}

fn rust_environment_directories(project_root: &Path) -> Vec<std::path::PathBuf> {
    rust_environment(project_root)
        .into_values()
        .map(Into::into)
        .collect()
}

fn prepare_rust_command(
    project: &Project,
    command: &str,
    extra_args: &[String],
) -> Result<PreparedCommand> {
    if !toolchain::toolchain_ready(project, &["rustc", "cargo"]) {
        return Err(ManscriptError::EnvironmentNotReady(
            project.environment_dir(),
        ));
    }
    let mut argv = split_command_line(command)?;
    argv.extend(extra_args.iter().cloned());
    let program_name = argv.remove(0);
    if !matches!(program_name.as_str(), "cargo" | "rustc") {
        return Err(ManscriptError::InvalidCommand(format!(
            "Rust project commands must invoke mapped `cargo` or `rustc`, not `{program_name}`"
        )));
    }
    let program = toolchain::env_tool(project, &program_name)?;
    Ok(PreparedCommand {
        program,
        args: argv,
        cwd: project.root.clone(),
        extra_env: rust_environment(&project.root),
        path_prepend: vec![project.environment_bin_dir()],
    })
}

fn run_cargo(project: &Project, args: &[String]) -> Result<()> {
    let prepared = prepare_rust_command(project, "cargo", args)?;
    Executor::new().run_status(prepared)
}

fn validate_packages(packages: &[String]) -> Result<()> {
    if packages.iter().any(|package| {
        package.is_empty()
            || package.starts_with('-')
            || package.chars().any(char::is_whitespace)
            || package.chars().any(char::is_control)
    }) {
        return Err(ManscriptError::InvalidCommand(
            "Cargo package specifications must be non-empty argv values and must not begin with `-`"
                .into(),
        ));
    }
    Ok(())
}

fn rust_tool_check(group: &str, program: &str, recommendation: &str) -> DoctorCheck {
    match which::which(exe_name(program)) {
        Ok(path) => {
            let version = crate::runtime::system::probe_version(&path, &["--version"])
                .ok()
                .flatten()
                .unwrap_or_else(|| path.display().to_string());
            DoctorCheck {
                group: group.into(),
                ok: true,
                label: format!(
                    "{program} {version} detected (system toolchain; project uses a shim)"
                ),
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
    use std::path::PathBuf;

    #[test]
    fn rust_environment_stays_under_project() {
        let root = PathBuf::from("project");
        let environment = rust_environment(&root);

        for value in environment.values() {
            assert!(Path::new(value).starts_with(&root));
        }
        assert!(environment["CARGO_HOME"].ends_with("cargo-home"));
        assert!(environment["CARGO_TARGET_DIR"].ends_with("cargo-target"));
    }

    #[test]
    fn cargo_packages_reject_option_injection() {
        assert!(validate_packages(&["--path=/tmp/other".into()]).is_err());
        assert!(validate_packages(&["serde@1".into()]).is_ok());
    }
}
