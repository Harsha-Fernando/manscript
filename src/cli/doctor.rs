use crate::core::errors::Result;
use crate::core::project::Project;
use crate::core::registry::AdapterRegistry;
use crate::utils::output::Printer;
use crate::utils::platform::platform_label;
use std::env;

pub fn execute(registry: &AdapterRegistry) -> Result<()> {
    let printer = Printer::new();
    printer.command_intro(
        "Doctor",
        "Check runtimes, toolchains, and the current project without changing them.",
    );

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

    if failed {
        printer.blank();
        printer.warn(if recommendations.len() == 1 {
            "Recommended next step"
        } else {
            "Recommended next steps"
        });
        printer.muted("  Resolve the failed checks above, then run `manscript doctor` again.");
        printer.blank();
        for rec in recommendations {
            printer.line(&rec);
            printer.blank();
        }
    } else {
        printer.command_done("Everything looks good. ManScript is ready to use.", &[]);
    }
    Ok(())
}
