use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::adapters::traits::{DoctorCheck, LanguageAdapter};
use crate::core::environment::{Environment, EnvironmentKind, ShellEnvironment};
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::core::runtime::Runtime;
use crate::process::{split_command_line, Executor, PreparedCommand};
use crate::runtime::uv::find_uv;
use crate::utils::filesystem::ensure_dir;
use crate::utils::platform::{env_bin_dir, exe_name, python_bin_name};

pub mod frameworks;
pub mod package_manager;
pub mod runtime;

pub struct PythonAdapter;

impl LanguageAdapter for PythonAdapter {
    fn id(&self) -> &'static str {
        "python"
    }

    fn package_manager_name(&self) -> &'static str {
        "pip"
    }

    fn default_environment_manager(&self) -> &'static str {
        "venv"
    }

    fn create_environment(&self, project: &Project, runtime: &Runtime) -> Result<Environment> {
        let root = project.environment_dir();
        ensure_dir(root.parent().unwrap_or(Path::new(".")))?;
        let prepared = PreparedCommand {
            program: runtime.executable.clone(),
            args: vec!["-m".into(), "venv".into(), root.display().to_string()],
            cwd: project.root.clone(),
            extra_env: HashMap::new(),
            path_prepend: Vec::new(),
        };
        Executor::new().run_status(prepared)?;
        Ok(Environment {
            bin_dir: env_bin_dir(&root),
            root,
            kind: EnvironmentKind::Venv,
        })
    }

    fn environment_ready(&self, project: &Project) -> bool {
        project
            .environment_bin_dir()
            .join(python_bin_name())
            .is_file()
    }

    fn install_dependencies(&self, project: &Project) -> Result<()> {
        let python = env_python(project)?;
        let requirements = project.root.join("requirements.txt");
        let req_dir = project.root.join("requirements").join("requirements.txt");
        if requirements.is_file() {
            pip_install(
                project,
                &python,
                &["-r".into(), requirements.display().to_string()],
            )
        } else if req_dir.is_file() {
            pip_install(
                project,
                &python,
                &["-r".into(), req_dir.display().to_string()],
            )
        } else if project.root.join("pyproject.toml").is_file() {
            pip_install(project, &python, &["-e".into(), ".".into()])
        } else if project.root.join("Gemfile").exists() {
            Ok(())
        } else {
            Ok(())
        }
    }

    fn install_packages(&self, project: &Project, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        let python = env_python(project)?;
        pip_install(project, &python, packages)
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
        let mut argv = split_command_line(command)?;
        argv.extend(extra_args.iter().cloned());
        let program_name = argv.remove(0);
        let program = resolve_python_program(project, &program_name)?;
        Ok(PreparedCommand {
            program,
            args: argv,
            cwd: project.root.clone(),
            extra_env: HashMap::new(),
            path_prepend: vec![project.environment_bin_dir()],
        })
    }

    fn shell_environment(&self, project: &Project) -> Result<ShellEnvironment> {
        Ok(ShellEnvironment {
            path_prepend: vec![project.environment_bin_dir()],
            extra_env: HashMap::new(),
        })
    }

    fn doctor_checks(&self) -> Vec<DoctorCheck> {
        let mut checks = Vec::new();
        match which::which("python3").or_else(|_| which::which("python")) {
            Ok(p) => {
                let ver = crate::runtime::system::probe_version(&p, &["-V"])
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| p.display().to_string());
                checks.push(DoctorCheck {
                    group: "Python".into(),
                    ok: true,
                    label: format!("Python {ver} detected"),
                    recommendation: None,
                });
            }
            Err(_) => checks.push(DoctorCheck {
                group: "Python".into(),
                ok: false,
                label: "Python not found on PATH".into(),
                recommendation: Some(
                    "ManScript can install an isolated Python runtime for this project.\n\nRun:\n\n    manscript setup".into(),
                ),
            }),
        }

        let venv_ok = which::which("python3")
            .or_else(|_| which::which("python"))
            .ok()
            .and_then(|p| {
                std::process::Command::new(p)
                    .args(["-m", "venv", "-h"])
                    .output()
                    .ok()
            })
            .map(|o| o.status.success())
            .unwrap_or(false);
        checks.push(if venv_ok {
            DoctorCheck {
                group: "Python environment".into(),
                ok: true,
                label: "Virtual environment available".into(),
                recommendation: None,
            }
        } else {
            DoctorCheck {
                group: "Python environment".into(),
                ok: false,
                label: "venv module not available".into(),
                recommendation: Some(
                    "Install Python with the venv module, or run manscript setup.".into(),
                ),
            }
        });

        let pip_ok = which::which("pip3")
            .or_else(|_| which::which("pip"))
            .is_ok()
            || find_uv().is_some();
        checks.push(if pip_ok {
            DoctorCheck {
                group: "Package manager".into(),
                ok: true,
                label: if find_uv().is_some() {
                    "pip / uv detected".into()
                } else {
                    "pip detected".into()
                },
                recommendation: None,
            }
        } else {
            DoctorCheck {
                group: "Package manager".into(),
                ok: false,
                label: "pip not detected".into(),
                recommendation: Some(
                    "pip is provided by the project virtual environment after manscript setup."
                        .into(),
                ),
            }
        });

        checks
    }
}

fn env_python(project: &Project) -> Result<PathBuf> {
    let path = project.environment_bin_dir().join(python_bin_name());
    if path.is_file() {
        Ok(path)
    } else {
        Err(ManscriptError::EnvironmentNotReady(
            project.environment_dir(),
        ))
    }
}

fn resolve_python_program(project: &Project, name: &str) -> Result<PathBuf> {
    let bin = project.environment_bin_dir();
    if name == "python" || name == "python3" || name == python_bin_name() {
        return env_python(project);
    }
    let in_env = bin.join(exe_name(name));
    if in_env.is_file() {
        return Ok(in_env);
    }
    let in_project = project.root.join(name);
    if in_project.is_file() {
        return Ok(in_project);
    }
    Err(ManscriptError::InvalidCommand(format!(
        "cannot resolve '{name}' inside the project environment. Use the environment interpreter or a project script."
    )))
}

fn pip_install(project: &Project, python: &Path, args: &[String]) -> Result<()> {
    if let Some(uv) = find_uv() {
        let mut uv_args = vec![
            "pip".into(),
            "install".into(),
            "--python".into(),
            python.display().to_string(),
        ];
        uv_args.extend(args.iter().cloned());
        let prepared = PreparedCommand {
            program: uv,
            args: uv_args,
            cwd: project.root.clone(),
            extra_env: HashMap::new(),
            path_prepend: vec![project.environment_bin_dir()],
        };
        return Executor::new().run_status(prepared);
    }

    let mut pip_args = vec!["-m".into(), "pip".into(), "install".into()];
    pip_args.extend(args.iter().cloned());
    let prepared = PreparedCommand {
        program: python.to_path_buf(),
        args: pip_args,
        cwd: project.root.clone(),
        extra_env: HashMap::new(),
        path_prepend: vec![project.environment_bin_dir()],
    };
    Executor::new().run_status(prepared)
}
