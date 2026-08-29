use std::fs;
use std::path::{Path, PathBuf};

use crate::core::errors::{ManscriptError, Result};

pub fn manscript_home() -> PathBuf {
    if let Ok(custom) = std::env::var("MANSCRIPT_HOME") {
        return PathBuf::from(custom);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".manscript")
}

pub fn tools_dir() -> PathBuf {
    manscript_home().join("tools")
}

pub fn project_environment_dir(project_root: &Path) -> PathBuf {
    project_root.join(".manscript").join("environment")
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn write_file(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

pub fn append_unique(path: &Path, snippet: &str) -> Result<()> {
    let existing = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    if existing.contains(snippet.trim()) {
        return Ok(());
    }
    let mut next = existing;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next.push('\n');
    next.push_str(snippet);
    if !snippet.ends_with('\n') {
        next.push('\n');
    }
    write_file(path, &next)
}

pub fn dir_is_empty_or_missing(path: &Path) -> bool {
    if !path.exists() {
        return true;
    }
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    }
}

pub fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ManscriptError::InvalidProjectName(name.to_string()));
    }
    Ok(())
}

pub fn default_gitignore() -> &'static str {
    r#"# ManScript
.manscript/environment/

# Python
__pycache__/
*.py[cod]
.Python
*.egg-info/
.pytest_cache/
.mypy_cache/

# Ruby
/.bundle/
vendor/bundle/
tmp/
log/
*.gem

# PHP
/vendor/

# .NET
/bin/
/obj/

# OS
.DS_Store
"#
}
