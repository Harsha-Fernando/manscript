use crate::adapters::toolchain;
use crate::adapters::traits::{DoctorCheck, LanguageAdapter};
use crate::core::environment::{Environment, ShellEnvironment};
use crate::core::errors::Result;
use crate::core::project::Project;
use crate::core::runtime::Runtime;
use crate::process::PreparedCommand;
use std::collections::HashMap;

pub mod plain;

pub struct CAdapter;

impl LanguageAdapter for CAdapter {
    fn id(&self) -> &'static str {
        "c"
    }

    fn package_manager_name(&self) -> &'static str {
        "none"
    }

    fn default_environment_manager(&self) -> &'static str {
        "toolchain"
    }

    fn create_environment(&self, project: &Project, runtime: &Runtime) -> Result<Environment> {
        toolchain::from_runtime(project, runtime, "cc")
    }

    fn environment_ready(&self, project: &Project) -> bool {
        toolchain::toolchain_ready(project, &["cc"])
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
        toolchain::compile_to_app(project, "cc", "main.c")?;
        Ok(true)
    }

    fn doctor_checks(&self) -> Vec<DoctorCheck> {
        compiler_check("C", "cc")
    }
}

pub(crate) fn compiler_check(group: &'static str, program: &str) -> Vec<DoctorCheck> {
    match which::which(crate::utils::platform::exe_name(program)) {
        Ok(p) => {
            let ver = crate::runtime::system::probe_version(&p, &["--version"])
                .ok()
                .flatten()
                .unwrap_or_else(|| p.display().to_string());
            vec![DoctorCheck {
                group: group.into(),
                ok: true,
                label: format!("{program} {ver} detected (system compiler; project uses a shim)"),
                recommendation: None,
            }]
        }
        Err(_) => vec![DoctorCheck {
            group: group.into(),
            ok: false,
            label: format!("{program} not found"),
            recommendation: Some(format!(
                "Install a C/C++ compiler so `{program}` is on PATH (Xcode CLT, gcc, or clang). ManScript will not download one."
            )),
        }],
    }
}
