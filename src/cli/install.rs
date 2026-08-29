use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::core::registry::AdapterRegistry;
use crate::utils::output::Printer;
use std::env;

pub fn execute(registry: &AdapterRegistry) -> Result<()> {
    let printer = Printer::new();
    printer.info("Install");
    let project = Project::load(&env::current_dir()?)?;
    let lang = registry.language(project.language())?;
    if !lang.environment_ready(&project) {
        return Err(ManscriptError::EnvironmentNotReady(
            project.environment_dir(),
        ));
    }
    {
        let spin = printer.spinner("Installing dependencies");
        lang.install_dependencies(&project)?;
        spin.finish_ok("Dependencies installed");
    }
    printer.success("Project dependencies are installed.");
    Ok(())
}
