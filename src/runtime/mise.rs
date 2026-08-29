use std::path::{Path, PathBuf};
use std::process::Command;

use crate::adapters::traits::ConfirmPolicy;
use crate::core::errors::{ManscriptError, Result};
use crate::core::runtime::{version_matches, Runtime, RuntimeSource};
use crate::runtime::download::{download_to_file, extract_tar_gz, find_named_file};
use crate::runtime::system::probe_version;
use crate::runtime::RuntimeProvider;
use crate::utils::filesystem::{ensure_dir, manscript_home, tools_dir};
use crate::utils::platform::{exe_name, mise_platform_suffix};

pub struct MiseProvider;

impl RuntimeProvider for MiseProvider {
    fn id(&self) -> &'static str {
        "mise"
    }

    fn supports(&self, language: &str) -> bool {
        mise_tool(language).is_some()
    }

    fn detect(&self, language: &str, version: &str) -> Result<Option<Runtime>> {
        if !self.supports(language) {
            return Ok(None);
        }
        let Some(mise) = find_mise() else {
            return Ok(None);
        };
        find_with_mise(&mise, language, version)
    }

    fn prepare(&self, language: &str, version: &str, confirm: ConfirmPolicy) -> Result<Runtime> {
        if !self.supports(language) {
            return Err(ManscriptError::UnknownLanguage(language.to_string()));
        }
        if let Some(existing) = self.detect(language, version)? {
            return Ok(existing);
        }

        let mise = match find_mise() {
            Some(p) => p,
            None => {
                let ok = confirm.confirm(
                    "mise is not installed. ManScript can download it into a user-writable directory under ~/.manscript without sudo. Continue?",
                )?;
                if !ok {
                    return Err(ManscriptError::Cancelled);
                }
                bootstrap_mise()?
            }
        };

        if self.detect(language, version)?.is_none() {
            let ok = confirm.confirm(&format!(
                "{} {version} was not found. ManScript can install an isolated runtime via mise under ~/.manscript without sudo. Continue?",
                display_language(language)
            ))?;
            if !ok {
                return Err(ManscriptError::Cancelled);
            }
            install_runtime(&mise, language, version)?;
        }

        find_with_mise(&mise, language, version)?.ok_or_else(|| ManscriptError::RuntimeNotFound {
            language: language.into(),
            version: version.to_string(),
        })
    }
}

fn mise_data_dir() -> PathBuf {
    manscript_home().join("runtimes").join("mise")
}

pub fn find_mise() -> Option<PathBuf> {
    if let Ok(p) = which::which(exe_name("mise")) {
        return Some(p);
    }
    let local = tools_dir().join("mise").join(exe_name("mise"));
    if local.is_file() {
        return Some(local);
    }
    None
}

fn mise_env(cmd: &mut Command) {
    cmd.env("MISE_DATA_DIR", mise_data_dir());
    cmd.env("MISE_YES", "1");
}

fn find_with_mise(mise: &Path, language: &str, version: &str) -> Result<Option<Runtime>> {
    let Some((tool, executable_name)) = mise_tool(language) else {
        return Ok(None);
    };
    let spec = format!("{tool}@{version}");
    let mut cmd = Command::new(mise);
    cmd.args(["where", &spec]);
    mise_env(&mut cmd);
    let output = cmd.output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let dir = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    if dir.as_os_str().is_empty() {
        return Ok(None);
    }
    let executable = [
        dir.join("bin").join(exe_name(executable_name)),
        dir.join(exe_name(executable_name)),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file());
    let Some(executable) = executable else {
        return Ok(None);
    };
    let detected =
        probe_version(&executable, &["--version"])?.unwrap_or_else(|| version.to_string());
    if version_matches(&detected, version) {
        Ok(Some(Runtime {
            language: language.into(),
            version: detected,
            executable,
            source: RuntimeSource::Provider("mise".into()),
        }))
    } else {
        Ok(None)
    }
}

fn install_runtime(mise: &Path, language: &str, version: &str) -> Result<()> {
    let (tool, _) =
        mise_tool(language).ok_or_else(|| ManscriptError::UnknownLanguage(language.to_string()))?;
    ensure_dir(&mise_data_dir())?;
    let spec = format!("{tool}@{version}");
    let mut cmd = Command::new(mise);
    cmd.args(["install", &spec]);
    mise_env(&mut cmd);
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(ManscriptError::Message(format!(
            "`mise` could not install {spec}.\n\nTool output:\n{}{}\nCheck the output above, then run `manscript setup` again.",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

fn mise_tool(language: &str) -> Option<(&'static str, &'static str)> {
    match language {
        "ruby" => Some(("ruby", "ruby")),
        "go" => Some(("go", "go")),
        "rust" => Some(("rust", "rustc")),
        "php" => Some(("php", "php")),
        "csharp" => Some(("dotnet", "dotnet")),
        _ => None,
    }
}

fn display_language(language: &str) -> &'static str {
    match language {
        "ruby" => "Ruby",
        "go" => "Go",
        "rust" => "Rust",
        "php" => "PHP",
        "csharp" => ".NET",
        _ => "Runtime",
    }
}

fn latest_mise_release_tag() -> Result<String> {
    let response = ureq::get("https://api.github.com/repos/jdx/mise/releases/latest")
        .set("User-Agent", "manscript/0.1")
        .call()
        .map_err(|e| {
            ManscriptError::Message(format!(
                "ManScript could not determine the latest `mise` release.\n\nNetwork detail:\n  {e}\n\nCheck your connection and try `manscript setup` again."
            ))
        })?;
    let body = response.into_string().map_err(|e| {
        ManscriptError::Message(format!(
            "ManScript downloaded the latest `mise` release metadata but could not read it.\n\nSystem detail:\n  {e}\n\nTry `manscript setup` again."
        ))
    })?;
    parse_release_tag(&body).ok_or_else(|| {
        ManscriptError::Message(
            "ManScript could not parse the latest `mise` release metadata.\n\nTry `manscript setup` again.".into(),
        )
    })
}

fn parse_release_tag(body: &str) -> Option<String> {
    let marker = "\"tag_name\":";
    let rest = body.split(marker).nth(1)?.trim_start();
    let tag = rest.strip_prefix('"')?;
    let end = tag.find('"')?;
    let tag = tag[..end].trim();
    if tag.is_empty() {
        None
    } else {
        Some(tag.to_string())
    }
}

fn bootstrap_mise() -> Result<PathBuf> {
    let Some((platform_suffix, bin_name)) = mise_platform_suffix() else {
        return Err(ManscriptError::Message(
            "Automatic `mise` installation is not supported on this platform.\n\nInstall `mise` or a compatible runtime manually, then run `manscript setup` again.".into(),
        ));
    };
    let dest_dir = tools_dir().join("mise");
    ensure_dir(&dest_dir)?;
    let tag = latest_mise_release_tag()?;
    let dest_bin = dest_dir.join(bin_name);

    #[cfg(windows)]
    {
        let url = format!(
            "https://github.com/jdx/mise/releases/download/{tag}/mise-{tag}-{platform_suffix}.exe"
        );
        download_to_file(&url, &dest_bin)?;
        return Ok(dest_bin);
    }

    #[cfg(not(windows))]
    {
        let url = format!(
            "https://github.com/jdx/mise/releases/download/{tag}/mise-{tag}-{platform_suffix}.tar.gz"
        );
        let archive = dest_dir.join("mise-download.tar.gz");
        download_to_file(&url, &archive)?;
        let extract_dir = dest_dir.join("extract");
        let _ = std::fs::remove_dir_all(&extract_dir);
        extract_tar_gz(&archive, &extract_dir)?;
        let found = find_named_file(&extract_dir, bin_name).ok_or_else(|| {
            ManscriptError::Message(
                "The downloaded `mise` archive did not contain the expected executable.\n\nRemove the incomplete `mise` directory from the ManScript cache, then run `manscript setup` again."
                    .into(),
            )
        })?;
        std::fs::copy(&found, &dest_bin)?;
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest_bin)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest_bin, perms)?;
        let _ = std::fs::remove_file(&archive);
        let _ = std::fs::remove_dir_all(&extract_dir);
        Ok(dest_bin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_latest_mise_release_tag() {
        let body = r#"{"tag_name":"v2026.8.14","name":"v2026.8.14"}"#;
        assert_eq!(parse_release_tag(body).as_deref(), Some("v2026.8.14"));
    }

    #[test]
    fn maps_supported_languages_to_mise_tools() {
        assert_eq!(mise_tool("ruby"), Some(("ruby", "ruby")));
        assert_eq!(mise_tool("go"), Some(("go", "go")));
        assert_eq!(mise_tool("rust"), Some(("rust", "rustc")));
        assert_eq!(mise_tool("php"), Some(("php", "php")));
        assert_eq!(mise_tool("csharp"), Some(("dotnet", "dotnet")));
        assert_eq!(mise_tool("python"), None);
    }
}
