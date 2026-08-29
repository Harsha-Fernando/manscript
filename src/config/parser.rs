use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::errors::{ManscriptError, Result};

pub const CONFIG_FILE_NAME: &str = "manscript.toml";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    pub name: String,
    pub language: LanguageConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<FrameworkConfig>,
    #[serde(default)]
    pub environment: EnvironmentConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub commands: CommandsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameworkConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EnvironmentConfig {
    #[serde(default)]
    pub manager: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuntimeConfig {
    /// Optional pin: "system", "uv", "mise". Reserved for lock/repro later.
    #[serde(default)]
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CommandsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
}

impl ProjectConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)?;
        parse_toml(&text)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let text = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, text)?;
        Ok(())
    }
}

pub fn parse_toml(text: &str) -> Result<ProjectConfig> {
    Ok(toml::from_str(text)?)
}

pub fn find_project_root(start: &Path) -> Result<PathBuf> {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        let candidate = current.join(CONFIG_FILE_NAME);
        if candidate.is_file() {
            return Ok(current);
        }
        if !current.pop() {
            return Err(ManscriptError::ProjectNotFound);
        }
    }
}

pub fn config_path(root: &Path) -> PathBuf {
    root.join(CONFIG_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_django_example() {
        let toml = r#"
name = "myproject"

[language]
name = "python"
version = "3.13"

[framework]
name = "django"
version = "5.2"

[environment]
manager = "venv"

[commands]
run = "python manage.py runserver"
test = "python manage.py test"
"#;
        let cfg = parse_toml(toml).unwrap();
        assert_eq!(cfg.name, "myproject");
        assert_eq!(cfg.language.name, "python");
        assert_eq!(cfg.framework.as_ref().unwrap().name, "django");
        assert_eq!(
            cfg.commands.run.as_deref(),
            Some("python manage.py runserver")
        );
    }

    #[test]
    fn language_only_toml_omits_framework_section() {
        let cfg = crate::adapters::traits::default_project_config(
            "myapp",
            "python",
            "3.13",
            None,
            "venv",
            CommandsConfig {
                run: Some("python main.py".into()),
                test: None,
                build: None,
            },
        );
        let text = toml::to_string_pretty(&cfg).unwrap();
        assert!(!text.contains("[framework]"));
        assert!(!text.contains("test"));
        assert!(!text.contains("build"));
        assert!(text.contains("[language]"));
        assert!(text.contains("python main.py"));
    }
}
