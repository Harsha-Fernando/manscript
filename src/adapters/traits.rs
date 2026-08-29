use crate::config::{CommandsConfig, ProjectConfig};
use crate::core::environment::{Environment, ShellEnvironment};
use crate::core::errors::Result;
use crate::core::project::Project;
use crate::core::runtime::Runtime;
use crate::process::PreparedCommand;
use crate::utils::output::Printer;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmPolicy {
    AlwaysYes,
    Prompt,
}

impl ConfirmPolicy {
    pub fn from_yes_flag(yes: bool) -> Self {
        if yes {
            Self::AlwaysYes
        } else {
            Self::Prompt
        }
    }

    pub fn confirm(&self, message: &str) -> Result<bool> {
        match self {
            Self::AlwaysYes => Ok(true),
            Self::Prompt => {
                if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                    return Ok(false);
                }
                let ans = crate::utils::prompts::confirm(message, true)?;
                Ok(ans)
            }
        }
    }
}

#[derive(Clone)]
pub struct ScaffoldContext<'a> {
    pub project_root: &'a Path,
    pub project_name: &'a str,
    pub runtime: &'a Runtime,
    pub environment: &'a Environment,
    pub yes: bool,
    pub printer: &'a Printer,
}

#[derive(Debug, Clone)]
pub struct DoctorCheck {
    pub group: String,
    pub ok: bool,
    pub label: String,
    pub recommendation: Option<String>,
}

pub trait LanguageAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn package_manager_name(&self) -> &'static str;
    fn default_environment_manager(&self) -> &'static str;
    fn create_environment(&self, project: &Project, runtime: &Runtime) -> Result<Environment>;
    fn environment_ready(&self, project: &Project) -> bool;
    fn install_dependencies(&self, project: &Project) -> Result<()>;
    fn install_packages(&self, project: &Project, packages: &[String]) -> Result<()>;
    fn resolve_command(
        &self,
        project: &Project,
        command: &str,
        extra_args: &[String],
    ) -> Result<PreparedCommand>;
    fn shell_environment(&self, project: &Project) -> Result<ShellEnvironment>;
    fn doctor_checks(&self) -> Vec<DoctorCheck>;

    /// Compile or otherwise prepare artifacts before `run` / `build`.
    /// Returns true when this adapter handled the build (no extra toml command).
    fn ensure_artifacts(&self, _project: &Project) -> Result<bool> {
        Ok(false)
    }
}

pub trait FrameworkAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn language(&self) -> &'static str;
    fn default_language_version(&self) -> &'static str;
    fn default_framework_version(&self) -> &'static str;
    fn default_commands(&self, project_name: &str) -> CommandsConfig;
    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()>;
    fn extra_create_steps(&self) -> usize {
        0
    }

    /// True for language-only adapters (`python`, `ruby`, `c`, …) — no [framework] in toml.
    fn language_only(&self) -> bool {
        false
    }

    /// Things this framework can add inside an existing project (`startapp`, blueprint, …).
    fn generators(&self) -> &'static [GeneratorSpec] {
        &[]
    }

    fn generate(&self, _ctx: &GenerateContext<'_>, kind: &str, _name: &str) -> Result<()> {
        Err(crate::core::errors::ManscriptError::Message(format!(
            "This framework does not support the `{kind}` generator.\n\nRun `manscript create` without arguments to see the available generator types."
        )))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratorSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub hint: &'static str,
}

#[derive(Clone)]
pub struct GenerateContext<'a> {
    pub project: &'a Project,
    pub runtime: &'a Runtime,
    pub environment: &'a Environment,
    pub printer: &'a Printer,
    pub yes: bool,
}

pub fn default_project_config(
    name: &str,
    language: &str,
    language_version: &str,
    framework: Option<(&str, &str)>,
    manager: &str,
    commands: CommandsConfig,
) -> ProjectConfig {
    ProjectConfig {
        name: name.to_string(),
        language: crate::config::LanguageConfig {
            name: language.to_string(),
            version: language_version.to_string(),
        },
        framework: framework.map(|(n, v)| crate::config::FrameworkConfig {
            name: n.to_string(),
            version: v.to_string(),
        }),
        environment: crate::config::EnvironmentConfig {
            manager: Some(manager.to_string()),
        },
        runtime: crate::config::RuntimeConfig::default(),
        commands,
    }
}
