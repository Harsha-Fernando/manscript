use crate::adapters::traits::ConfirmPolicy;
use crate::config::{config_path, CONFIG_FILE_NAME};
use crate::core::errors::Result;
use crate::core::registry::AdapterRegistry;
use crate::utils::output::Printer;
use crate::utils::prompts::{
    framework_choice, language_choice, language_picker_rank, resolve_none_to_language, select,
};
use std::env;

pub fn execute(registry: &AdapterRegistry, yes: bool) -> Result<()> {
    let printer = Printer::new();
    printer.info("Init");
    let cwd = env::current_dir()?;
    let dest = config_path(&cwd);
    if dest.exists() {
        printer.info(&format!("{CONFIG_FILE_NAME} already exists."));
        return Ok(());
    }

    let mut lang_adapters: Vec<_> = registry.languages();
    lang_adapters.sort_by_key(|l| language_picker_rank(l.id()));
    let languages: Vec<_> = lang_adapters
        .iter()
        .map(|l| language_choice(l.id()))
        .collect();
    let language = if yes {
        "python".to_string()
    } else {
        select(
            "Which language are we borrowing confidence from?",
            &languages,
            0,
        )?
    };
    let fws_real = registry.frameworks_for_language(&language);
    let fw = if fws_real.is_empty() {
        language.clone()
    } else {
        let mut fws: Vec<_> = fws_real
            .into_iter()
            .map(|f| framework_choice(f.id()))
            .collect();
        fws.push(framework_choice("none"));
        if yes {
            fws.first()
                .map(|c| c.id.to_string())
                .unwrap_or_else(|| "none".into())
        } else {
            let choice = select("And which framework gets the starring role?", &fws, 0)?;
            resolve_none_to_language(&choice, &language)
        }
    };

    let name = cwd
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string();

    let adapter = registry.framework(&fw)?;
    let (language_version, manager, commands, framework) = (
        adapter.default_language_version().to_string(),
        registry
            .language(adapter.language())?
            .default_environment_manager()
            .to_string(),
        adapter.default_commands(&name),
        if adapter.language_only() {
            None
        } else {
            Some((adapter.id(), adapter.default_framework_version()))
        },
    );

    let config = crate::adapters::traits::default_project_config(
        &name,
        &language,
        &language_version,
        framework,
        &manager,
        commands,
    );
    config.save(&dest)?;
    printer.success(&format!("wrote {CONFIG_FILE_NAME}"));
    printer.blank();
    printer.muted("  Next:");
    printer.hint_command("manscript setup");
    let _ = ConfirmPolicy::AlwaysYes;
    Ok(())
}
