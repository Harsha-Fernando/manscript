use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::core::registry::AdapterRegistry;
use crate::utils::output::Printer;
use std::env;

pub fn execute(registry: &AdapterRegistry) -> Result<()> {
    let printer = Printer::new();
    let project = Project::load(&env::current_dir()?)?;
    let lang = registry.language(project.language())?;
    if !lang.environment_ready(&project) {
        return Err(ManscriptError::EnvironmentNotReady(
            project.environment_dir(),
        ));
    }
    printer.command_intro(
        "Install",
        "Synchronize dependencies inside the project environment.",
    );
    printer.key_value("Project", &project.config.name);
    printer.key_value("Environment", ".manscript/environment");
    printer.blank();
    {
        let spin = printer.spinner("Installing dependencies");
        lang.install_dependencies(&project)?;
        spin.finish_ok("Dependencies installed");
    }
    printer.command_done(
        "Project dependencies are installed.",
        &["manscript run", "manscript shell"],
    );
    Ok(())
}
