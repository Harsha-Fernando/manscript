use crate::adapters::traits::ConfirmPolicy;
use crate::core::errors::Result;
use crate::core::project::Project;
use crate::core::registry::AdapterRegistry;
use crate::utils::output::Printer;
use std::env;

pub fn execute(registry: &AdapterRegistry, yes: bool) -> Result<()> {
    let printer = Printer::new();
    printer.info("Setup");
    let project = Project::load(&env::current_dir()?)?;
    prepare(
        &project,
        registry,
        ConfirmPolicy::from_yes_flag(yes),
        &printer,
    )
}

pub fn prepare(
    project: &Project,
    registry: &AdapterRegistry,
    confirm: ConfirmPolicy,
    printer: &Printer,
) -> Result<()> {
    let lang = registry.language(project.language())?;
    printer.info(&format!(
        "Setting up {} ({})",
        project.config.name,
        project.language()
    ));
    printer.muted("  fetching a sensible universe for this folder…");
    printer.blank();

    let runtime = {
        let spin = printer.spinner("Preparing runtime");
        let runtime = registry.resolve_runtime(
            project.language(),
            project.language_version(),
            project.config.runtime.provider.as_deref(),
            confirm,
        )?;
        spin.finish_ok(&format!(
            "{} {} ({})",
            capitalize(project.language()),
            runtime.version,
            runtime.source.label()
        ));
        runtime
    };
    if !lang.environment_ready(project) {
        let spin = printer.spinner("Creating environment");
        lang.create_environment(project, &runtime)?;
        spin.finish_ok("Environment created");
    } else {
        printer.check_ok("Environment ready", "");
    }
    {
        let spin = printer.spinner("Installing dependencies");
        lang.install_dependencies(project)?;
        spin.finish_ok("Dependencies installed");
    }
    printer.blank();
    printer.success("Setup complete. The machine is, briefly, on your side.");
    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
