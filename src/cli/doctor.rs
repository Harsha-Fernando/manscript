use crate::core::errors::Result;
use crate::core::project::Project;
use crate::core::registry::AdapterRegistry;
use crate::utils::output::Printer;
use crate::utils::platform::platform_label;
use std::env;

pub fn execute(registry: &AdapterRegistry) -> Result<()> {
    let printer = Printer::new();
    printer.info("ManScript Doctor");

    printer.section("Platform");
    printer.check_ok(&platform_label(), "");

    printer.section("ManScript");
    printer.check_ok(env!("CARGO_PKG_VERSION"), "");

    let mut failed = false;
    let mut recommendations = Vec::new();
    let mut last_group = String::new();

    for lang in registry.languages() {
        for check in lang.doctor_checks() {
            if check.group != last_group {
                printer.section(&check.group);
                last_group = check.group.clone();
            }
            if check.ok {
                printer.check_ok(&check.label, "");
            } else {
                failed = true;
                printer.check_fail(&check.label, "");
                if let Some(rec) = check.recommendation {
                    recommendations.push(rec);
                }
            }
        }
    }

    if let Ok(project) = Project::load(&env::current_dir().unwrap_or_else(|_| ".".into())) {
        printer.section("Current project");
        printer.check_ok(
            &format!(
                "{} ({}/{})",
                project.config.name,
                project.language(),
                project.framework_name().unwrap_or("none")
            ),
            "",
        );
        let lang = registry.language(project.language()).ok();
        if let Some(lang) = lang {
            if lang.environment_ready(&project) {
                printer.check_ok("Project environment ready", "");
            } else {
                failed = true;
                printer.check_fail("Project environment not ready", "");
                recommendations.push("Run:\n\n    manscript setup".into());
            }
        }
    }

    printer.blank();
    if failed {
        printer.warn("Recommendation");
        for rec in recommendations {
            printer.line(&rec);
            printer.blank();
        }
    } else {
        printer.success("Everything looks good. Go forth and ship.");
    }
    Ok(())
}
