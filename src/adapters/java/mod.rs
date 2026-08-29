use crate::adapters::toolchain;
use crate::adapters::traits::{DoctorCheck, LanguageAdapter};
use crate::core::environment::{Environment, ShellEnvironment};
use crate::core::errors::Result;
use crate::core::project::Project;
use crate::core::runtime::Runtime;
use crate::process::PreparedCommand;
use crate::utils::platform::exe_name;
use std::collections::HashMap;

pub mod plain;

pub struct JavaAdapter;

impl LanguageAdapter for JavaAdapter {
    fn id(&self) -> &'static str {
        "java"
    }

    fn package_manager_name(&self) -> &'static str {
        "none"
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
        let java = toolchain::sibling_or_which(&runtime.executable, "java")?;
        toolchain::create_toolchain_env(
            project,
            &[
                ("javac", runtime.executable.as_path()),
                ("java", java.as_path()),
            ],
        )
    }

    fn environment_ready(&self, project: &Project) -> bool {
        toolchain::toolchain_ready(project, &["javac", "java"])
    }

    fn install_dependencies(&self, _project: &Project) -> Result<()> {
        Ok(())
    }

    fn install_packages(&self, _project: &Project, _packages: &[String]) -> Result<()> {
        toolchain::no_packages()
    }

    fn resolve_command(
        &self,
        project: &Project,
        command: &str,
        extra_args: &[String],
    ) -> Result<PreparedCommand> {
        toolchain::resolve_env_command(project, command, extra_args)
    }

    fn shell_environment(&self, project: &Project) -> Result<ShellEnvironment> {
        Ok(ShellEnvironment {
            path_prepend: vec![project.environment_bin_dir()],
            extra_env: HashMap::new(),
        })
    }

    fn ensure_artifacts(&self, project: &Project) -> Result<bool> {
        toolchain::javac_main(project)?;
        Ok(true)
    }

    fn doctor_checks(&self) -> Vec<DoctorCheck> {
        match which::which(exe_name("javac")) {
            Ok(p) => {
                let ver = crate::runtime::system::probe_version(&p, &["-version"])
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| p.display().to_string());
                vec![DoctorCheck {
                    group: "Java".into(),
                    ok: true,
                    label: format!("javac {ver} detected (system JDK; project uses shims)"),
                    recommendation: None,
                }]
            }
            Err(_) => vec![DoctorCheck {
                group: "Java".into(),
                ok: false,
                label: "javac not found".into(),
                recommendation: Some(
                    "Install a JDK 17 or newer so `javac` and `java` are on PATH. ManScript will not download a JDK."
                        .into(),
                ),
            }],
        }
    }
}
