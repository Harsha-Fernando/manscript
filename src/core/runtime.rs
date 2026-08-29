use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Runtime {
    pub language: String,
    pub version: String,
    pub executable: PathBuf,
    pub source: RuntimeSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeSource {
    System,
    Provider(String),
}

impl RuntimeSource {
    pub fn label(&self) -> String {
        match self {
            Self::System => "system".into(),
            Self::Provider(id) => id.clone(),
        }
    }
}

/// Normalize a language version string into a semver-ish comparator.
/// "3.13" means any 3.13.x (and we also accept a matching prefix if parse fails).
pub fn version_matches(installed: &str, required: &str) -> bool {
    let installed_n = normalize_version(installed);
    let required_n = normalize_version(required);
    if let (Ok(inst), Ok(req)) = (
        semver::Version::parse(&installed_n),
        semver::Version::parse(&pad_patch(&required_n)),
    ) {
        if required.matches('.').count() == 0 {
            return inst.major == req.major;
        }
        if required.matches('.').count() == 1 {
            return inst.major == req.major && inst.minor >= req.minor;
        }
        return inst >= req;
    }
    installed.starts_with(required)
}

pub fn normalize_version(raw: &str) -> String {
    let trimmed = raw.trim();
    let trimmed = trimmed.trim_start_matches('v');
    let mut parts = Vec::new();
    for chunk in trimmed.split(|c: char| !c.is_ascii_digit() && c != '.') {
        if !chunk.is_empty() && chunk.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            for p in chunk.split('.') {
                if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) {
                    parts.push(p);
                }
            }
            break;
        }
    }
    if parts.is_empty() {
        return trimmed.to_string();
    }
    while parts.len() < 3 {
        parts.push("0");
    }
    parts.into_iter().take(3).collect::<Vec<_>>().join(".")
}

fn pad_patch(v: &str) -> String {
    let n = v.matches('.').count();
    match n {
        0 => format!("{v}.0.0"),
        1 => format!("{v}.0"),
        _ => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_minor() {
        assert!(version_matches("3.13.2", "3.13"));
        assert!(version_matches("3.14.6", "3.13"));
        assert!(!version_matches("3.12.0", "3.13"));
        assert!(version_matches("3.4.5", "3.4"));
    }
}
