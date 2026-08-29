use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::core::registry::AdapterRegistry;
use crate::process::Executor;
use crate::utils::output::Printer;
use std::env;

pub fn execute_run(registry: &AdapterRegistry, args: &[String]) -> Result<()> {
    execute_named(registry, "run", args, true)
}

pub fn execute_test(registry: &AdapterRegistry, args: &[String]) -> Result<()> {
    execute_named(registry, "test", args, true)
}

pub fn execute_build(registry: &AdapterRegistry, args: &[String]) -> Result<()> {
    execute_named(registry, "build", args, true)
}

/// True when `run` should show server chrome (INFO + URL), not a silent script exec.
pub(crate) fn is_framework_dev_server(registry: &AdapterRegistry, project: &Project) -> bool {
    let Some(name) = project.framework_name() else {
        return false;
    };
    registry
        .framework(name)
        .map(|fw| !fw.language_only())
        .unwrap_or(false)
}

pub(crate) fn default_server_url(framework: &str) -> Option<&'static str> {
    match framework {
        "django" | "fastapi" => Some("http://127.0.0.1:8000/"),
        "flask" => Some("http://127.0.0.1:5000/"),
        "rails" => Some("http://127.0.0.1:3000/"),
        "sinatra" => Some("http://127.0.0.1:4567/"),
        _ => None,
    }
}

fn execute_named(
    registry: &AdapterRegistry,
    which: &str,
    args: &[String],
    inherit: bool,
) -> Result<()> {
    let printer = Printer::new();
    let project = Project::load(&env::current_dir()?)?;
    let lang = registry.language(project.language())?;

    let _ = registry.resolve_runtime(
        project.language(),
        project.language_version(),
        project.config.runtime.provider.as_deref(),
        crate::adapters::traits::ConfirmPolicy::AlwaysYes,
    );

    if !lang.environment_ready(&project) {
        return Err(ManscriptError::EnvironmentNotReady(
            project.environment_dir(),
        ));
    }

    if which == "build" && lang.ensure_artifacts(&project)? {
        printer.info("Build finished.");
        return Ok(());
    }

    let command = match which {
        "run" => project.config.commands.run.clone(),
        "test" => project.config.commands.test.clone(),
        "build" => project.config.commands.build.clone(),
        _ => None,
    };
    let Some(command) = command else {
        if which == "build" || which == "test" {
            printer.info(&format!(
                "This project does not define a `{which}` command. Nothing was run."
            ));
            return Ok(());
        }
        return Err(ManscriptError::Message(format!(
            "This project does not define a `{which}` command in `manscript.toml`.\n\nAdd it under `[commands]`, then try again. Example:\n\n    [commands]\n    {which} = \"your-program --arguments\""
        )));
    };

    if which == "run" && is_framework_dev_server(registry, &project) {
        printer.info("Starting development server.");
        if let Some(fw) = project.framework_name() {
            if let Some(url) = default_server_url(fw) {
                printer.url(url);
            }
        }
        printer.blank();
    }

    if which == "run" {
        let _ = lang.ensure_artifacts(&project)?;
    }

    let prepared = lang.resolve_command(&project, &command, args)?;
    if inherit {
        let code = Executor::new().run_inherit(prepared)?;
        if code != 0 {
            std::process::exit(code);
        }
        Ok(())
    } else {
        Executor::new().run_status(prepared)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::registry::default_registry;

    fn project(framework: Option<(&str, &str)>) -> Project {
        Project {
            root: ".".into(),
            config: crate::adapters::traits::default_project_config(
                "x",
                "python",
                "3.13",
                framework,
                "venv",
                Default::default(),
            ),
        }
    }

    #[test]
    fn language_only_is_not_a_dev_server() {
        let r = default_registry();
        assert!(!is_framework_dev_server(&r, &project(None)));
        assert!(!is_framework_dev_server(&r, &project(Some(("python", "")))));
        assert!(is_framework_dev_server(
            &r,
            &project(Some(("django", "5.2")))
        ));
        assert!(default_server_url("python").is_none());
        assert_eq!(default_server_url("django"), Some("http://127.0.0.1:8000/"));
    }
}
