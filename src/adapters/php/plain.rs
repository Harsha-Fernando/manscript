use crate::adapters::traits::{FrameworkAdapter, ScaffoldContext};
use crate::config::CommandsConfig;
use crate::core::errors::Result;
use crate::utils::filesystem::write_file;

pub struct PlainPhpFramework;

impl FrameworkAdapter for PlainPhpFramework {
    fn id(&self) -> &'static str {
        "php"
    }

    fn language(&self) -> &'static str {
        "php"
    }

    fn default_language_version(&self) -> &'static str {
        "8.4"
    }

    fn default_framework_version(&self) -> &'static str {
        ""
    }

    fn language_only(&self) -> bool {
        true
    }

    fn default_commands(&self, _project_name: &str) -> CommandsConfig {
        CommandsConfig {
            run: Some("php main.php".into()),
            test: None,
            build: None,
        }
    }

    fn scaffold(&self, ctx: &ScaffoldContext<'_>) -> Result<()> {
        write_file(
            &ctx.project_root.join("main.php"),
            r#"<?php

declare(strict_types=1);

echo "Hello from ManScript (PHP, no framework)." . PHP_EOL;
"#,
        )?;
        write_file(
            &ctx.project_root.join("composer.json"),
            &composer_json(ctx.project_name),
        )
    }
}

fn composer_json(project_name: &str) -> String {
    let package = sanitize_package_segment(project_name);
    format!(
        r#"{{
    "name": "manscript/{package}",
    "description": "A plain PHP project created by ManScript",
    "type": "project",
    "require": {{}}
}}
"#
    )
}

fn sanitize_package_segment(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            let c = c.to_ascii_lowercase();
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '-'
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches(|c| matches!(c, '-' | '_' | '.'));
    if sanitized.is_empty() {
        "app".into()
    } else {
        sanitized.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composer_name_is_safe_and_lowercase() {
        assert_eq!(sanitize_package_segment("My_App-2"), "my_app-2");
        assert_eq!(sanitize_package_segment("../"), "app");
        assert!(composer_json("Hello").contains("\"manscript/hello\""));
    }
}
