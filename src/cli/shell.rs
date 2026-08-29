use std::env;
use std::path::PathBuf;

use crate::core::environment::ShellEnvironment;
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::core::registry::AdapterRegistry;
use crate::process::{Executor, PreparedCommand};
use crate::utils::output::Printer;

pub fn execute(registry: &AdapterRegistry) -> Result<()> {
    let project = Project::load(&env::current_dir()?)?;
    let language = registry.language(project.language())?;

    if !language.environment_ready(&project) {
        return Err(ManscriptError::EnvironmentNotReady(
            project.environment_dir(),
        ));
    }

    let shell_environment = language.shell_environment(&project)?;
    print_banner(&project);
    let code = Executor::new().run_inherit(prepare_shell_launch(
        &project,
        shell_environment,
        resolve_shell_program(),
    ))?;
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

fn resolve_shell_program() -> PathBuf {
    if let Some(shell) = env::var_os("MANSCRIPT_SHELL") {
        return shell.into();
    }

    #[cfg(windows)]
    {
        env::var_os("COMSPEC")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("powershell.exe"))
    }

    #[cfg(not(windows))]
    {
        env::var_os("SHELL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/bin/sh"))
    }
}

fn prepare_shell_launch(
    project: &Project,
    mut shell_environment: ShellEnvironment,
    program: PathBuf,
) -> PreparedCommand {
    shell_environment.extra_env.insert(
        "PS1".into(),
        format!("{} $ ", shell_prompt_name(&project.config.name)),
    );

    PreparedCommand {
        program,
        args: Vec::new(),
        cwd: project.root.clone(),
        extra_env: shell_environment.extra_env,
        path_prepend: shell_environment.path_prepend,
    }
}

fn print_banner(project: &Project) {
    let printer = Printer::new();
    let project_name = terminal_text(&project.config.name);
    let language = terminal_text(project.language());
    let language_version = terminal_text(project.language_version());
    printer.info("ManScript development shell");
    printer.line(&format!("  Project     {project_name}"));
    printer.line(&format!(
        "  {}{}{}",
        display_name(&language),
        spacing_after(&language),
        language_version
    ));
    if let Some(framework) = &project.config.framework {
        let framework_name = terminal_text(&framework.name);
        printer.line(&format!(
            "  {}{}{}",
            display_name(&framework_name),
            spacing_after(&framework_name),
            terminal_text(&framework.version)
        ));
    }
    printer.line("  Environment .manscript/environment");
    printer.blank();
    printer.muted("  Type `exit` to return to your normal shell.");
    printer.blank();
}

fn display_name(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn spacing_after(name: &str) -> String {
    " ".repeat(12usize.saturating_sub(name.len()))
}

fn shell_prompt_name(name: &str) -> String {
    let safe: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        .collect();
    if safe.is_empty() {
        "ManScript".into()
    } else {
        safe
    }
}

fn terminal_text(value: &str) -> String {
    value.chars().filter(|c| !c.is_control()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::traits::default_project_config;
    use crate::config::CommandsConfig;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn project() -> Project {
        Project {
            root: PathBuf::from("/tmp/example"),
            config: default_project_config(
                "Example",
                "python",
                "3.13",
                None,
                "venv",
                CommandsConfig::default(),
            ),
        }
    }

    #[test]
    fn prepares_child_only_shell_environment() {
        let project = project();
        let project_bin = project.environment_bin_dir();
        let mut extra_env = HashMap::new();
        extra_env.insert("EXISTING".into(), "preserved".into());

        let prepared = prepare_shell_launch(
            &project,
            ShellEnvironment {
                path_prepend: vec![project_bin.clone()],
                extra_env,
            },
            PathBuf::from("/bin/test-shell"),
        );

        assert_eq!(prepared.program, PathBuf::from("/bin/test-shell"));
        assert_eq!(prepared.cwd, project.root);
        assert!(prepared.args.is_empty());
        assert_eq!(prepared.path_prepend, vec![project_bin]);
        assert_eq!(prepared.extra_env.get("EXISTING").unwrap(), "preserved");
        assert_eq!(prepared.extra_env.get("PS1").unwrap(), "Example $ ");
    }

    #[test]
    fn shell_override_is_used_for_testable_process_execution() {
        let _guard = ENV_LOCK.lock().unwrap();
        let previous: Option<OsString> = env::var_os("MANSCRIPT_SHELL");
        env::set_var("MANSCRIPT_SHELL", "/tmp/manscript-test-shell");

        assert_eq!(
            resolve_shell_program(),
            PathBuf::from("/tmp/manscript-test-shell")
        );

        match previous {
            Some(value) => env::set_var("MANSCRIPT_SHELL", value),
            None => env::remove_var("MANSCRIPT_SHELL"),
        }
    }

    #[test]
    fn prompt_does_not_interpolate_untrusted_project_config() {
        assert_eq!(shell_prompt_name("safe-project_1"), "safe-project_1");
        assert_eq!(shell_prompt_name("$(touch /tmp/owned)"), "touchtmpowned");
        assert_eq!(shell_prompt_name("\u{1b}]52;clipboard\u{7}"), "52clipboard");
    }
}
