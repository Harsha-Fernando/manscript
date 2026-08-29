use crate::core::errors::Result;
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
        printer.info("Environment is not ready yet.");
        printer.blank();
        printer.hint_command("manscript setup");
        return Ok(());
    }
    {
        let spin = printer.spinner("Installing dependencies");
        lang.install_dependencies(&project)?;
        spin.finish_ok("Dependencies installed");
    }
    printer.success("Packages: acquired. Ego: inflated.");
    Ok(())
}
