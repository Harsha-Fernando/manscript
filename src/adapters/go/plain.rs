use crate::adapters::traits::{FrameworkAdapter, ScaffoldContext};
use crate::config::CommandsConfig;
use crate::core::errors::Result;
use crate::utils::filesystem::write_file;

pub struct PlainGoFramework;

impl FrameworkAdapter for PlainGoFramework {
    fn id(&self) -> &'static str {
        "go"
    }

    fn language(&self) -> &'static str {
        "go"
    }

    fn default_language_version(&self) -> &'static str {
        "1.25"
    }

    fn default_framework_version(&self) -> &'static str {
        ""
    }

    fn language_only(&self) -> bool {
        true
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("go run .".into()),
            test: Some("go test ./...".into()),
            build: Some("go build".into()),
        }
    }

    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()> {
        write_file(
            &ctx.project_root.join("go.mod"),
            &go_mod_contents(ctx.project_name),
        )?;
        write_file(&ctx.project_root.join("main.go"), MAIN_GO)
    }
}

fn normalize_module_name(project_name: &str) -> String {
    let normalized = project_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches(|character| matches!(character, '-' | '_' | '.'))
        .to_string();

    if normalized.is_empty() {
        "manscript-app".into()
    } else {
        normalized
    }
}

fn go_mod_contents(project_name: &str) -> String {
    format!(
        "module {}\n\ngo 1.25\n",
        normalize_module_name(project_name)
    )
}

const MAIN_GO: &str = r#"package main

import "fmt"

func main() {
	fmt.Println("Hello from ManScript (Go, no framework).")
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_module_names() {
        assert_eq!(normalize_module_name("Hello_App"), "hello_app");
        assert_eq!(normalize_module_name(" hello app "), "hello-app");
        assert_eq!(normalize_module_name("---"), "manscript-app");
    }

    #[test]
    fn scaffold_contents_use_normalized_module() {
        assert_eq!(
            go_mod_contents("Hello App"),
            "module hello-app\n\ngo 1.25\n"
        );
        assert!(MAIN_GO.contains("package main"));
        assert!(MAIN_GO.contains("func main()"));
    }

    #[test]
    fn commands_use_go_argv() {
        let commands = PlainGoFramework.default_commands("example");
        assert_eq!(commands.run.as_deref(), Some("go run ."));
        assert_eq!(commands.build.as_deref(), Some("go build"));
    }
}
