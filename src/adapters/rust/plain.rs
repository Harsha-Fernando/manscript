use crate::adapters::traits::{FrameworkAdapter, ScaffoldContext};
use crate::config::CommandsConfig;
use crate::core::errors::Result;
use crate::utils::filesystem::write_file;

pub struct PlainRustFramework;

impl FrameworkAdapter for PlainRustFramework {
    fn id(&self) -> &'static str {
        "rust"
    }

    fn language(&self) -> &'static str {
        "rust"
    }

    fn default_language_version(&self) -> &'static str {
        "1.82"
    }

    fn default_framework_version(&self) -> &'static str {
        ""
    }

    fn language_only(&self) -> bool {
        true
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("cargo run".into()),
            test: Some("cargo test".into()),
            build: Some("cargo build".into()),
        }
    }

    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()> {
        write_file(
            &ctx.project_root.join("Cargo.toml"),
            &cargo_toml_contents(ctx.project_name),
        )?;
        write_file(&ctx.project_root.join("src").join("main.rs"), MAIN_RS)
    }
}

fn normalize_package_name(project_name: &str) -> String {
    let normalized = project_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches(|character| matches!(character, '-' | '_'))
        .to_string();

    match normalized.chars().next() {
        Some(first) if first.is_ascii_alphabetic() => normalized,
        Some(_) => format!("manscript-{normalized}"),
        None => "manscript-app".into(),
    }
}

fn cargo_toml_contents(project_name: &str) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        normalize_package_name(project_name)
    )
}

const MAIN_RS: &str = r#"fn main() {
    println!("Hello from ManScript (Rust, no framework).");
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_package_names() {
        assert_eq!(normalize_package_name("Hello_App"), "hello_app");
        assert_eq!(normalize_package_name(" hello app "), "hello-app");
        assert_eq!(normalize_package_name("123-app"), "manscript-123-app");
        assert_eq!(normalize_package_name("---"), "manscript-app");
    }

    #[test]
    fn scaffold_contents_use_normalized_package() {
        let manifest = cargo_toml_contents("Hello App");
        assert!(manifest.contains("name = \"hello-app\""));
        assert!(manifest.contains("edition = \"2024\""));
        assert!(MAIN_RS.contains("fn main()"));
    }

    #[test]
    fn commands_use_cargo_argv() {
        let commands = PlainRustFramework.default_commands("example");
        assert_eq!(commands.run.as_deref(), Some("cargo run"));
        assert_eq!(commands.build.as_deref(), Some("cargo build"));
    }
}
