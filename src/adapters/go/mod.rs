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

pub struct GoAdapter;

impl LanguageAdapter for GoAdapter {
    fn id(&self) -> &'static str {
        "go"
    }

    fn package_manager_name(&self) -> &'static str {
        "go modules"
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
        let environment = toolchain::from_runtime(project, runtime, "go")?;
        for directory in go_environment_directories(&project.root) {
            ensure_dir(&directory)?;
        }
        Ok(environment)
    }

    fn environment_ready(&self, project: &Project) -> bool {
        toolchain::toolchain_ready(project, &["go"])
    }

    fn install_dependencies(&self, project: &Project) -> Result<()> {
        if !project.root.join("go.mod").is_file() {
            return Ok(());
        }
        run_go(project, &["mod".into(), "download".into()])
    }

    fn install_packages(&self, project: &Project, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        validate_packages(packages)?;
        let mut args = vec!["get".into()];
        args.extend(packages.iter().cloned());
        run_go(project, &args)
    }

    fn resolve_command(
        &self,
        project: &Project,
        command: &str,
        extra_args: &[String],
    ) -> Result<PreparedCommand> {
        prepare_go_command(project, command, extra_args)
    }

    fn shell_environment(&self, project: &Project) -> Result<ShellEnvironment> {
        if !self.environment_ready(project) {
            return Err(ManscriptError::EnvironmentNotReady(
                project.environment_dir(),
            ));
        }
        Ok(ShellEnvironment {
            path_prepend: vec![project.environment_bin_dir()],
            extra_env: go_environment(&project.root),
        })
    }

    fn doctor_checks(&self) -> Vec<DoctorCheck> {
        match which::which(exe_name("go")) {
            Ok(path) => {
                let version = crate::runtime::system::probe_version(&path, &["version"])
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| path.display().to_string());
                vec![DoctorCheck {
                    group: "Go".into(),
                    ok: true,
                    label: format!("Go {version} detected (system toolchain; project uses a shim)"),
                    recommendation: None,
                }]
            }
            Err(_) => vec![DoctorCheck {
                group: "Go".into(),
                ok: false,
                label: "Go toolchain not found".into(),
                recommendation: Some(
                    "Run `manscript setup` to install an isolated Go toolchain with mise, or install Go so `go` is on PATH."
                        .into(),
                ),
            }],
        }
    }
}

fn go_environment(project_root: &Path) -> HashMap<String, String> {
    let environment_root = project_root.join(".manscript").join("environment");
    HashMap::from([
        (
            "GOMODCACHE".into(),
            environment_root.join("go-mod-cache").display().to_string(),
        ),
        (
            "GOCACHE".into(),
            environment_root
                .join("go-build-cache")
                .display()
                .to_string(),
        ),
        (
            "GOPATH".into(),
            environment_root.join("go-path").display().to_string(),
        ),
    ])
}

fn go_environment_directories(project_root: &Path) -> Vec<std::path::PathBuf> {
    go_environment(project_root)
        .into_values()
        .map(Into::into)
        .collect()
}

fn prepare_go_command(
    project: &Project,
    command: &str,
    extra_args: &[String],
) -> Result<PreparedCommand> {
    if !toolchain::toolchain_ready(project, &["go"]) {
        return Err(ManscriptError::EnvironmentNotReady(
            project.environment_dir(),
        ));
    }
    let mut argv = split_command_line(command)?;
    argv.extend(extra_args.iter().cloned());
    let program_name = argv.remove(0);
    if program_name != "go" {
        return Err(ManscriptError::InvalidCommand(format!(
            "Go project commands must invoke the mapped `go` executable, not `{program_name}`"
        )));
    }
    let program = toolchain::env_tool(project, "go")?;
    Ok(PreparedCommand {
        program,
        args: argv,
        cwd: project.root.clone(),
        extra_env: go_environment(&project.root),
        path_prepend: vec![project.environment_bin_dir()],
    })
}

fn run_go(project: &Project, args: &[String]) -> Result<()> {
    let prepared = prepare_go_command(project, "go", args)?;
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
            "Go module specifications must be non-empty argv values and must not begin with `-`"
                .into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn go_environment_stays_under_project() {
        let root = PathBuf::from("project");
        let environment = go_environment(&root);

        for value in environment.values() {
            assert!(Path::new(value).starts_with(&root));
        }
        assert!(environment["GOMODCACHE"].ends_with("go-mod-cache"));
        assert!(environment["GOCACHE"].ends_with("go-build-cache"));
        assert!(environment["GOPATH"].ends_with("go-path"));
    }

    #[test]
    fn go_packages_reject_option_injection() {
        assert!(validate_packages(&["--modfile=/tmp/other.mod".into()]).is_err());
        assert!(validate_packages(&["example.com/module@v1.2.3".into()]).is_ok());
    }
}
