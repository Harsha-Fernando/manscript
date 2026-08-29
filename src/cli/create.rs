use crate::adapters::traits::{ConfirmPolicy, GenerateContext};
use crate::config::{config_path, ProjectConfig};
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::core::registry::AdapterRegistry;
use crate::core::runtime::{Runtime, RuntimeSource};
use crate::utils::filesystem::{
    default_gitignore, dir_is_empty_or_missing, ensure_dir, validate_project_name,
};
use crate::utils::output::{display_name, Printer};
use crate::utils::platform::platform_label;
use crate::utils::prompts::{
    framework_choice, language_choice, language_picker_rank, resolve_none_to_language, select,
    text, Choice,
};
use std::env;
use std::path::{Path, PathBuf};

pub fn execute(
    registry: &AdapterRegistry,
    framework: Option<String>,
    name: Option<String>,
    yes: bool,
) -> Result<()> {
    let printer = Printer::new();
    if let Ok(project) = Project::load(&env::current_dir()?) {
        printer.command_intro(
            "Create",
            "Add a framework component to the current project.",
        );
        return create_in_project(registry, project, framework, name, yes, &printer);
    }
    printer.command_intro(
        "Create",
        "Build a new isolated project from a supported stack.",
    );
    let confirm = ConfirmPolicy::from_yes_flag(yes);

    if framework.is_none() {
        printer
            .muted("  Choose a language and framework, then ManScript will prepare the project.");
        printer.blank();
    }

    let (framework_id, project_name) = resolve_inputs(registry, framework, name)?;
    validate_project_name(&project_name)?;

    let fw = registry.framework(&framework_id)?;
    let language = fw.language();
    let lang = registry.language(language)?;
    let dest = PathBuf::from(&project_name);

    if dest.exists() && !dir_is_empty_or_missing(&dest) {
        let ok = confirm.confirm(&format!(
            "Directory `{project_name}` is not empty. Continue and allow ManScript to add or replace project files?"
        ))?;
        if !ok {
            return Err(ManscriptError::ProjectExists(dest));
        }
    }

    printer.key_value("Stack", &display_name(&framework_id));
    printer.key_value("Project", &project_name);
    printer.blank();

    let total = (6 + fw.extra_create_steps()) as u64;
    let mut progress = printer.steps(total);

    progress.begin("Detecting platform");
    progress.ok(&platform_label());

    progress.begin(&format!("Checking {}", display_name(language)));
    let preferred = None;
    let detected = registry
        .providers()
        .iter()
        .find(|p| p.id() == "system")
        .and_then(|p| p.detect(language, fw.default_language_version()).ok())
        .flatten();
    if let Some(ref rt) = detected {
        progress.ok(&format!(
            "{} {} detected ({})",
            display_name(language),
            rt.version,
            rt.source.label()
        ));
    } else {
        progress.note(&format!(
            "{} {} was not found locally; ManScript will prepare it",
            display_name(language),
            fw.default_language_version()
        ));
    }

    progress.begin(&format!(
        "Preparing {} {}",
        display_name(language),
        fw.default_language_version()
    ));
    let runtime =
        registry.resolve_runtime(language, fw.default_language_version(), preferred, confirm)?;
    progress.ok("Runtime prepared");

    ensure_dir(&dest)?;
    write_project_files(
        &dest,
        &project_name,
        fw,
        language,
        lang.default_environment_manager(),
    )?;

    let project = Project {
        root: dest.canonicalize().unwrap_or(dest.clone()),
        config: ProjectConfig::load(&config_path(&dest))?,
    };

    progress.begin("Creating environment");
    lang.create_environment(&project, &runtime, confirm)?;
    progress.ok("Environment created");

    if fw.language_only() {
        progress.begin("Creating project files");
    } else if fw.extra_create_steps() > 0 {
        progress.begin(&format!("Installing {}", display_name(&framework_id)));
    }
    let env = crate::core::environment::Environment {
        root: project.environment_dir(),
        bin_dir: project.environment_bin_dir(),
        kind: crate::core::environment::EnvironmentKind::from_manager(
            lang.default_environment_manager(),
        ),
    };
    let ctx = crate::adapters::traits::ScaffoldContext {
        project_root: &project.root,
        project_name: &project_name,
        runtime: &runtime,
        environment: &env,
        yes,
        printer: &printer,
    };
    fw.scaffold(&ctx)?;
    if fw.language_only() {
        progress.ok("Starter files written");
    } else if fw.extra_create_steps() > 0 {
        progress.ok(&format!("{} installed", display_name(&framework_id)));
        progress.begin("Creating project");
        progress.ok(&format!("{} project created", display_name(&framework_id)));
    }

    progress.begin("Creating ManScript configuration");
    progress.ok("manscript.toml created");
    progress.finish();
    let cd_command = format!("cd {project_name}");
    printer.command_done(
        &format!("Project `{project_name}` is ready."),
        &[&cd_command, "manscript run", "manscript shell"],
    );
    printer.blank();
    printer.muted(
        "  Use `manscript run` for the configured app command, or `manscript shell` for ad-hoc tools.",
    );
    Ok(())
}

fn write_project_files(
    dest: &Path,
    project_name: &str,
    fw: &dyn crate::adapters::traits::FrameworkAdapter,
    language: &str,
    manager: &str,
) -> Result<()> {
    let framework_meta = if fw.language_only() {
        None
    } else {
        Some((fw.id(), fw.default_framework_version()))
    };
    let config = crate::adapters::traits::default_project_config(
        project_name,
        language,
        fw.default_language_version(),
        framework_meta,
        manager,
        fw.default_commands(project_name),
    );
    config.save(&config_path(dest))?;
    crate::utils::filesystem::write_file(&dest.join(".gitignore"), default_gitignore())?;
    Ok(())
}

fn resolve_inputs(
    registry: &AdapterRegistry,
    framework: Option<String>,
    name: Option<String>,
) -> Result<(String, String)> {
    if let (Some(fw), Some(n)) = (framework.clone(), name.clone()) {
        let _ = registry.framework(&fw)?;
        return Ok((fw, n));
    }
    if let Some(fw) = framework {
        let _ = registry.framework(&fw)?;
        let n = text(
            "What should the project be named?",
            "myproject",
            "letters, numbers, hyphens, underscores",
        )?;
        return Ok((fw, n));
    }

    let mut lang_adapters: Vec<_> = registry.languages();
    lang_adapters.sort_by_key(|l| language_picker_rank(l.id()));
    let languages: Vec<_> = lang_adapters
        .iter()
        .map(|l| language_choice(l.id()))
        .collect();
    let lang = select("Which language will this project use?", &languages, 0)?;

    let fws_real = registry.frameworks_for_language(&lang);
    let fw_choice = if fws_real.is_empty() {
        lang.clone()
    } else {
        let mut fws: Vec<_> = fws_real
            .into_iter()
            .map(|f| framework_choice(f.id()))
            .collect();
        fws.push(framework_choice("none"));
        let choice = select("Which framework will this project use?", &fws, 0)?;
        resolve_none_to_language(&choice, &lang)
    };
    let n = text(
        "What should the project be named?",
        "myproject",
        "letters, numbers, hyphens, underscores",
    )?;
    Ok((fw_choice, n))
}

fn create_in_project(
    registry: &AdapterRegistry,
    project: Project,
    first: Option<String>,
    second: Option<String>,
    yes: bool,
    printer: &Printer,
) -> Result<()> {
    let fw_id = project
        .framework_name()
        .unwrap_or_else(|| project.language());
    let fw = registry.framework(fw_id)?;
    if let Some(ref a) = first {
        if registry.framework(a).is_ok() {
            return Err(already_in_project(fw_id));
        }
    }
    let gens = fw.generators();
    if gens.is_empty() {
        printer.blank();
        printer.info("This project has no apps or modules to generate.");
        printer.muted("  Language-only projects do not have framework generators. Add files directly, or create a framework project in a new folder.");
        return Ok(());
    }

    let (kind, name) = resolve_in_project_inputs(registry, fw_id, gens, first, second, yes)?;
    validate_project_name(&name)?;
    if name.contains('-') && matches!(fw.id(), "django" | "flask" | "fastapi") {
        return Err(ManscriptError::InvalidProjectName(name));
    }

    let lang = registry.language(project.language())?;
    if !lang.environment_ready(&project) && matches!(fw.id(), "django" | "rails") {
        return Err(ManscriptError::EnvironmentNotReady(
            project.environment_dir(),
        ));
    }

    let runtime = match registry.resolve_runtime(
        project.language(),
        project.language_version(),
        project.config.runtime.provider.as_deref(),
        ConfirmPolicy::AlwaysYes,
    ) {
        Ok(rt) => rt,
        Err(_) if !matches!(fw.id(), "django" | "rails") => Runtime {
            language: project.language().to_string(),
            version: project.language_version().to_string(),
            executable: PathBuf::from(project.language()),
            source: RuntimeSource::System,
        },
        Err(e) => return Err(e),
    };
    let env = crate::core::environment::Environment {
        root: project.environment_dir(),
        bin_dir: project.environment_bin_dir(),
        kind: crate::core::environment::EnvironmentKind::from_manager(
            lang.default_environment_manager(),
        ),
    };
    printer.key_value("Project", &project.config.name);
    printer.key_value("Adding", &format!("{kind} `{name}`"));
    printer.blank();
    let ctx = GenerateContext {
        project: &project,
        runtime: &runtime,
        environment: &env,
        printer,
        yes,
    };
    fw.generate(&ctx, &kind, &name)?;
    printer.command_done(
        &format!("Added {kind} `{name}` to the project."),
        &["manscript run"],
    );
    Ok(())
}

fn already_in_project(framework: &str) -> ManscriptError {
    ManscriptError::Message(format!(
        "This folder is already a ManScript {framework} project, so ManScript will not create another project inside it.\n\nTo add an app or module to the current project, run:\n\n    manscript create blog\n\nTo create a separate project, change to a folder outside this project first."
    ))
}

fn resolve_in_project_inputs(
    registry: &AdapterRegistry,
    fw_id: &str,
    gens: &[crate::adapters::traits::GeneratorSpec],
    first: Option<String>,
    second: Option<String>,
    yes: bool,
) -> Result<(String, String)> {
    let is_fw = |s: &str| registry.framework(s).is_ok();
    let is_kind = |s: &str| gens.iter().any(|g| g.id == s);

    match (first, second) {
        (Some(a), Some(b)) => {
            if is_fw(&a) {
                return Err(already_in_project(fw_id));
            }
            if is_kind(&a) {
                return Ok((a, b));
            }
            Err(ManscriptError::Message(format!(
                "`{a}` is not a supported generator for this project.\n\nUse:\n\n    manscript create {} {b}",
                gens[0].id
            )))
        }
        (Some(a), None) => {
            if is_fw(&a) {
                return Err(already_in_project(fw_id));
            }
            if is_kind(&a) {
                if yes {
                    return Err(ManscriptError::Message(
                        "That generator requires a name.\n\nExample:\n\n    manscript create app blog"
                            .into(),
                    ));
                }
                let n = text(
                    "What should we call it?",
                    "blog",
                    "letters, numbers, underscores",
                )?;
                return Ok((a, n));
            }
            let kind = if gens.iter().any(|g| g.id == "app") {
                "app".to_string()
            } else {
                pick_kind(gens, yes)?
            };
            Ok((kind, a))
        }
        (None, None) => {
            let kind = pick_kind(gens, yes)?;
            if yes {
                return Err(ManscriptError::Message(
                    "A name is required.\n\nExample:\n\n    manscript create blog".into(),
                ));
            }
            let n = text(
                "What should we call it?",
                "blog",
                "letters, numbers, underscores",
            )?;
            Ok((kind, n))
        }
        (None, Some(_)) => Err(ManscriptError::Message(
            "The name must be the first argument.\n\nExample:\n\n    manscript create blog".into(),
        )),
    }
}

fn pick_kind(gens: &[crate::adapters::traits::GeneratorSpec], yes: bool) -> Result<String> {
    if gens.len() == 1 {
        return Ok(gens[0].id.to_string());
    }
    if yes {
        return Err(ManscriptError::Message(
            "This framework requires a generator kind and name.\n\nExample:\n\n    manscript create scaffold Post"
                .into(),
        ));
    }
    let choices: Vec<Choice<'_>> = gens
        .iter()
        .map(|g| Choice {
            id: g.id,
            label: g.label,
            hint: g.hint,
        })
        .collect();
    select("What are we adding?", &choices, 0)
}
