use crate::adapters::traits::ConfirmPolicy;
use crate::core::errors::Result;
use crate::core::project::Project;
use crate::core::registry::AdapterRegistry;
use crate::utils::output::{display_name, Printer};
use std::env;

pub fn execute(registry: &AdapterRegistry, yes: bool) -> Result<()> {
    let printer = Printer::new();
    let project = Project::load(&env::current_dir()?)?;
    printer.command_intro(
        "Setup",
        "Prepare the runtime, isolated environment, and dependencies.",
    );
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
    printer.key_value("Project", &project.config.name);
    printer.key_value(
        "Stack",
        &format!(
            "{} {}",
            display_name(project.language()),
            project.language_version()
        ),
    );
    printer.blank();

    let mut progress = printer.steps(3);
    progress.begin("Preparing runtime");
    let runtime = registry.resolve_runtime(
        project.language(),
        project.language_version(),
        project.config.runtime.provider.as_deref(),
        confirm,
    )?;
    progress.ok(&format!(
        "{} {} ({})",
        display_name(project.language()),
        runtime.version,
        runtime.source.label()
    ));

    progress.begin("Preparing project environment");
    if !lang.environment_ready(project) {
        lang.create_environment(project, &runtime, confirm)?;
        progress.ok("Environment created");
    } else {
        progress.ok("Existing environment is ready");
    }

    progress.begin("Installing dependencies");
    lang.install_dependencies(project)?;
    progress.ok("Dependencies installed");
    progress.finish();

    printer.command_done(
        "Setup complete. The project environment is ready.",
        &["manscript run", "manscript shell"],
    );
    Ok(())
}
