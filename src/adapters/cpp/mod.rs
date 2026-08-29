use crate::adapters::c::compiler_check;
use crate::adapters::toolchain;
use crate::adapters::traits::{DoctorCheck, LanguageAdapter};
use crate::core::environment::Environment;
use crate::core::errors::Result;
use crate::core::project::Project;
use crate::core::runtime::Runtime;
use crate::process::PreparedCommand;

pub mod plain;

pub struct CppAdapter;

impl LanguageAdapter for CppAdapter {
    fn id(&self) -> &'static str {
        "cpp"
    }

    fn package_manager_name(&self) -> &'static str {
        "none"
    }

    fn default_environment_manager(&self) -> &'static str {
        "toolchain"
    }

    fn create_environment(&self, project: &Project, runtime: &Runtime) -> Result<Environment> {
        toolchain::from_runtime(project, runtime, "c++")
    }

    fn environment_ready(&self, project: &Project) -> bool {
        toolchain::toolchain_ready(project, &["c++"])
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

    fn ensure_artifacts(&self, project: &Project) -> Result<bool> {
        toolchain::compile_to_app(project, "c++", "main.cpp")?;
        Ok(true)
    }

    fn doctor_checks(&self) -> Vec<DoctorCheck> {
        compiler_check("C++", "c++")
    }
}
