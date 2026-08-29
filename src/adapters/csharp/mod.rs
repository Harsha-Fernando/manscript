use std::collections::HashMap;
use std::path::PathBuf;

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

pub struct CsharpAdapter;

impl LanguageAdapter for CsharpAdapter {
    fn id(&self) -> &'static str {
        "csharp"
    }

    fn package_manager_name(&self) -> &'static str {
        "nuget"
    }

    fn default_environment_manager(&self) -> &'static str {
        "dotnet"
    }

    fn create_environment(
        &self,
        project: &Project,
        runtime: &Runtime,
        _confirm: crate::adapters::traits::ConfirmPolicy,
    ) -> Result<Environment> {
        ensure_dir(&nuget_packages(project))?;
        ensure_dir(&dotnet_cli_home(project))?;
        toolchain::create_toolchain_env(project, &[("dotnet", runtime.executable.as_path())])
    }

    fn environment_ready(&self, project: &Project) -> bool {
        toolchain::toolchain_ready(project, &["dotnet"])
            && nuget_packages(project).is_dir()
            && dotnet_cli_home(project).is_dir()
    }

    fn install_dependencies(&self, project: &Project) -> Result<()> {
        let project_file = resolve_project_file(project)?;
        run_dotnet(
            project,
            vec![
                "restore".into(),
                project_file.display().to_string(),
                "--nologo".into(),
            ],
        )
    }

    fn install_packages(&self, project: &Project, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        let project_file = resolve_project_file(project)?;
        for package in packages {
            let (name, version) = parse_package_spec(package)?;
            let mut args = vec![
                "add".into(),
                project_file.display().to_string(),
                "package".into(),
                name.into(),
                "--no-restore".into(),
            ];
            if let Some(version) = version {
                args.extend(["--version".into(), version.into()]);
            }
            run_dotnet(project, args)?;
        }
        run_dotnet(
            project,
            vec![
                "restore".into(),
                project_file.display().to_string(),
                "--nologo".into(),
            ],
        )
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
        prepare_dotnet_command(project, command, extra_args)
    }

    fn shell_environment(&self, project: &Project) -> Result<ShellEnvironment> {
        Ok(ShellEnvironment {
            path_prepend: dotnet_path(project),
            extra_env: dotnet_env(project),
        })
    }

    fn doctor_checks(&self) -> Vec<DoctorCheck> {
        match which::which(exe_name("dotnet")) {
            Ok(path) => {
                let version = crate::runtime::system::probe_version(&path, &["--version"])
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| path.display().to_string());
                vec![DoctorCheck {
                    group: "C# / .NET".into(),
                    ok: true,
                    label: format!(".NET SDK {version} detected (project uses a shim)"),
                    recommendation: None,
                }]
            }
            Err(_) => vec![DoctorCheck {
                group: "C# / .NET".into(),
                ok: false,
                label: ".NET SDK not found".into(),
                recommendation: Some(
                    "Run `manscript setup` to install an isolated .NET 10 SDK with mise, or make a compatible `dotnet` available on PATH. ManScript keeps NuGet packages and CLI state inside the project."
                        .into(),
                ),
            }],
        }
    }
}

fn prepare_dotnet_command(
    project: &Project,
    command: &str,
    extra_args: &[String],
) -> Result<PreparedCommand> {
    let mut argv = split_command_line(command)?;
    let program_name = argv.remove(0);
    if program_name != "dotnet" {
        return Err(ManscriptError::InvalidCommand(format!(
            "C# project commands must invoke the mapped `dotnet` executable, not `{program_name}`"
        )));
    }

    if argv.first().is_some_and(|arg| arg == "run")
        && !extra_args.is_empty()
        && !argv.iter().any(|arg| arg == "--")
    {
        argv.push("--".into());
    }
    argv.extend(extra_args.iter().cloned());

    Ok(PreparedCommand {
        program: toolchain::env_tool(project, "dotnet")?,
        args: argv,
        cwd: project.root.clone(),
        extra_env: dotnet_env(project),
        path_prepend: dotnet_path(project),
    })
}

fn run_dotnet(project: &Project, args: Vec<String>) -> Result<()> {
    let prepared = PreparedCommand {
        program: toolchain::env_tool(project, "dotnet")?,
        args,
        cwd: project.root.clone(),
        extra_env: dotnet_env(project),
        path_prepend: dotnet_path(project),
    };
    Executor::new().run_status(prepared)
}

fn resolve_project_file(project: &Project) -> Result<PathBuf> {
    let generated = project
        .root
        .join(plain::project_file_name(&project.config.name));
    if generated.is_file() {
        return Ok(generated);
    }

    let mut candidates = std::fs::read_dir(&project.root)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "csproj"))
        .collect::<Vec<_>>();
    candidates.sort();
    match candidates.as_slice() {
        [only] => Ok(only.clone()),
        [] => Err(ManscriptError::Message(
            "No `.csproj` file was found in the C# project root.\n\nAdd a project file, then run `manscript setup` again."
                .into(),
        )),
        _ => Err(ManscriptError::Message(
            "Multiple `.csproj` files were found in the C# project root, so ManScript cannot safely choose one.\n\nKeep the generated project file or configure commands that name the intended project explicitly."
                .into(),
        )),
    }
}

fn parse_package_spec(spec: &str) -> Result<(&str, Option<&str>)> {
    let (name, version) = match spec.split_once(':') {
        Some((name, version)) => (name, Some(version)),
        None => (spec, None),
    };
    let valid_name = !name.is_empty()
        && !name.starts_with('-')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    let valid_version = version.is_none_or(|version| {
        !version.is_empty()
            && !version.starts_with('-')
            && !version.chars().any(char::is_whitespace)
            && !version.chars().any(char::is_control)
    });
    if !valid_name || !valid_version {
        return Err(ManscriptError::InvalidCommand(
            "NuGet package specifications must use `Package.Id` or `Package.Id:version` and must not contain options or whitespace"
                .into(),
        ));
    }
    Ok((name, version))
}

fn nuget_packages(project: &Project) -> PathBuf {
    project.root.join(".manscript").join("nuget-packages")
}

fn dotnet_cli_home(project: &Project) -> PathBuf {
    project.root.join(".manscript").join("dotnet-home")
}

fn dotnet_tools(project: &Project) -> PathBuf {
    dotnet_cli_home(project).join(".dotnet").join("tools")
}

fn dotnet_env(project: &Project) -> HashMap<String, String> {
    HashMap::from([
        (
            "NUGET_PACKAGES".into(),
            nuget_packages(project).display().to_string(),
        ),
        (
            "DOTNET_CLI_HOME".into(),
            dotnet_cli_home(project).display().to_string(),
        ),
        ("DOTNET_NOLOGO".into(), "1".into()),
        ("DOTNET_SKIP_FIRST_TIME_EXPERIENCE".into(), "1".into()),
    ])
}

fn dotnet_path(project: &Project) -> Vec<PathBuf> {
    vec![project.environment_bin_dir(), dotnet_tools(project)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_specs_are_split_without_shell_parsing() {
        assert_eq!(
            parse_package_spec("Serilog:4.0.0").unwrap(),
            ("Serilog", Some("4.0.0"))
        );
        assert_eq!(parse_package_spec("Serilog").unwrap(), ("Serilog", None));
    }

    #[test]
    fn package_specs_reject_option_injection() {
        for spec in ["--interactive", "Serilog:--source", "Bad Package"] {
            assert!(parse_package_spec(spec).is_err());
        }
    }

    #[test]
    fn dotnet_state_paths_stay_project_local() {
        let root = PathBuf::from("workspace").join("app");
        let project = Project {
            root: root.clone(),
            config: crate::adapters::traits::default_project_config(
                "app",
                "csharp",
                "8.0",
                None,
                "dotnet",
                Default::default(),
            ),
        };
        let env = dotnet_env(&project);
        let manscript_root = root.join(".manscript");
        assert!(Path::new(&env["NUGET_PACKAGES"]).starts_with(&manscript_root));
        assert!(Path::new(&env["DOTNET_CLI_HOME"]).starts_with(&manscript_root));
    }
}
