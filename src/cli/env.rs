use crate::core::errors::Result;
use crate::core::project::Project;
use crate::core::registry::AdapterRegistry;
use crate::utils::output::Printer;
use crate::utils::platform::{exe_name, python_bin_name};
use std::env;

pub fn execute(registry: &AdapterRegistry) -> Result<()> {
    let printer = Printer::new();
    let project = Project::load(&env::current_dir()?)?;
    let lang = registry.language(project.language()).ok();
    let has_framework = project.config.framework.is_some();

    printer.info("This project");
    printer.line(&format!("  Name        {}", project.config.name));
    printer.line(&format!("  Root        {}", project.root.display()));
    printer.line(&format!(
        "  Language    {} {}",
        project.language(),
        project.language_version()
    ));
    if let Some(fw) = &project.config.framework {
        printer.line(&format!("  Framework   {} {}", fw.name, fw.version));
    }
    printer.line(&format!(
        "  Environment {}",
        project.environment_dir().display()
    ));
    printer.line(&format!(
        "  Tool bin    {}",
        project.environment_bin_dir().display()
    ));
    if let Some(p) = &project.config.runtime.provider {
        printer.line(&format!("  Provider    {p}"));
    }

    if has_framework {
        printer.blank();
        printer.info("System tools are still on PATH. This app uses the isolated copies.");
        printer.muted("  Prefer:");
        printer.hint_command("manscript run");
        printer.hint_command("manscript test");
        printer.hint_command("manscript shell");
    } else {
        printer.blank();
        printer.muted("  Prefer:");
        printer.hint_command("manscript run");
        printer.hint_command("manscript shell");
    }

    printer.blank();
    printer.muted("  Project tools:");
    match project.language() {
        "python" => {
            let py = project.environment_bin_dir().join(python_bin_name());
            printer.hint_command(&py.display().to_string());
        }
        "ruby" => {
            let ruby = project.environment_bin_dir().join(exe_name("ruby"));
            printer.hint_command(&ruby.display().to_string());
        }
        "c" => print_env_tool(&printer, &project, "cc"),
        "cpp" => print_env_tool(&printer, &project, "c++"),
        "java" => print_env_tool(&printer, &project, "java"),
        other => {
            printer.hint_command(other);
        }
    }
    printer.blank();
    if let Some(lang) = lang {
        if lang.environment_ready(&project) {
            printer.success("Environment is ready. You may now write code of questionable wisdom.");
        } else {
            printer.info("Environment is not ready yet.");
            printer.hint_command("manscript setup");
        }
    }
    Ok(())
}

fn print_env_tool(printer: &Printer, project: &Project, name: &str) {
    match crate::adapters::toolchain::env_tool(project, name) {
        Ok(path) => printer.hint_command(&path.display().to_string()),
        Err(_) => printer.hint_command(name),
    }
}
