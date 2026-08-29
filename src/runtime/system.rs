use std::process::Command;

use crate::adapters::traits::ConfirmPolicy;
use crate::core::errors::{ManscriptError, Result};
use crate::core::runtime::{version_matches, Runtime, RuntimeSource};
use crate::runtime::RuntimeProvider;
use crate::utils::platform::exe_name;

pub struct SystemRuntimeProvider;

impl RuntimeProvider for SystemRuntimeProvider {
    fn id(&self) -> &'static str {
        "system"
    }

    fn supports(&self, language: &str) -> bool {
        matches!(language, "python" | "ruby" | "c" | "cpp" | "java")
    }

    fn detect(&self, language: &str, version: &str) -> Result<Option<Runtime>> {
        match language {
            "python" => detect_python(version),
            "ruby" => detect_ruby(version),
            "c" => detect_compiler("c", &["cc", "clang", "gcc"]),
            "cpp" => detect_compiler("cpp", &["c++", "clang++", "g++"]),
            "java" => detect_java(version),
            _ => Ok(None),
        }
    }

    fn prepare(&self, language: &str, version: &str, _confirm: ConfirmPolicy) -> Result<Runtime> {
        self.detect(language, version)?
            .ok_or_else(|| ManscriptError::RuntimeNotFound {
                language: language.to_string(),
                version: version.to_string(),
            })
    }
}

fn detect_python(required: &str) -> Result<Option<Runtime>> {
    let names = [
        format!("python{}", major_minor(required)),
        "python3".into(),
        "python".into(),
    ];
    for name in names {
        if let Some(runtime) = probe("python", &name, required, "-V")? {
            return Ok(Some(runtime));
        }
    }
    Ok(None)
}

fn detect_ruby(required: &str) -> Result<Option<Runtime>> {
    probe("ruby", "ruby", required, "--version")
}

fn detect_compiler(language: &str, names: &[&str]) -> Result<Option<Runtime>> {
    for name in names {
        if let Some(runtime) = probe_any(language, name, "--version")? {
            return Ok(Some(runtime));
        }
    }
    Ok(None)
}

fn detect_java(required: &str) -> Result<Option<Runtime>> {
    let Some(runtime) = probe_any("java", "javac", "-version")? else {
        return Ok(None);
    };
    let min_major = required
        .split('.')
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(17);
    let installed_major = runtime
        .version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    if installed_major >= min_major {
        Ok(Some(runtime))
    } else {
        Ok(None)
    }
}

fn probe_any(language: &str, program: &str, version_flag: &str) -> Result<Option<Runtime>> {
    let exe = exe_name(program);
    let path = match which::which(&exe) {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    let output = Command::new(&path).arg(version_flag).output()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let version = extract_version(&text).unwrap_or_else(|| {
        if text.trim().is_empty() {
            "system".into()
        } else {
            text.split_whitespace().next().unwrap_or("system").into()
        }
    });
    Ok(Some(Runtime {
        language: language.to_string(),
        version,
        executable: path,
        source: RuntimeSource::System,
    }))
}

fn probe(
    language: &str,
    program: &str,
    required: &str,
    version_flag: &str,
) -> Result<Option<Runtime>> {
    let exe = exe_name(program);
    let path = match which::which(&exe) {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    let output = Command::new(&path).arg(version_flag).output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = if text.trim().is_empty() {
        stderr.into_owned()
    } else {
        text.into_owned()
    };
    let version = extract_version(&combined).unwrap_or_else(|| combined.trim().to_string());
    if version_matches(&version, required) {
        Ok(Some(Runtime {
            language: language.to_string(),
            version,
            executable: path,
            source: RuntimeSource::System,
        }))
    } else {
        Ok(None)
    }
}

fn extract_version(text: &str) -> Option<String> {
    let mut best = None;
    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
        if cleaned
            .split('.')
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
            && cleaned.contains('.')
        {
            best = Some(cleaned.to_string());
        }
    }
    best
}

fn major_minor(version: &str) -> String {
    let parts: Vec<_> = version.split('.').take(2).collect();
    parts.join(".")
}

pub fn probe_version(executable: &std::path::Path, args: &[&str]) -> Result<Option<String>> {
    let output = Command::new(executable).args(args).output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(extract_version(&text))
}
