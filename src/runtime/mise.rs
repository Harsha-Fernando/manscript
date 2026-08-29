use std::path::{Path, PathBuf};
use std::process::Command;

use crate::adapters::traits::ConfirmPolicy;
use crate::core::errors::{ManscriptError, Result};
use crate::core::runtime::{version_matches, Runtime, RuntimeSource};
use crate::runtime::download::{download_to_file, extract_tar_gz, find_named_file};
use crate::runtime::system::probe_version;
use crate::runtime::RuntimeProvider;
use crate::utils::filesystem::{ensure_dir, manscript_home, tools_dir};
use crate::utils::platform::{exe_name, mise_download_target};

pub struct MiseRubyProvider;

impl RuntimeProvider for MiseRubyProvider {
    fn id(&self) -> &'static str {
        "mise"
    }

    fn supports(&self, language: &str) -> bool {
        language == "ruby"
    }

    fn detect(&self, language: &str, version: &str) -> Result<Option<Runtime>> {
        if language != "ruby" {
            return Ok(None);
        }
        let Some(mise) = find_mise() else {
            return Ok(None);
        };
        find_ruby_with_mise(&mise, version)
    }

    fn prepare(&self, language: &str, version: &str, confirm: ConfirmPolicy) -> Result<Runtime> {
        if language != "ruby" {
            return Err(ManscriptError::UnknownLanguage(language.to_string()));
        }
        if let Some(existing) = self.detect(language, version)? {
            return Ok(existing);
        }

        let mise = match find_mise() {
            Some(p) => p,
            None => {
                let ok = confirm.confirm(
                    "mise is not installed. ManScript can download mise into a user-writable directory (~/.manscript/tools) without sudo. Continue?",
                )?;
                if !ok {
                    return Err(ManscriptError::Cancelled);
                }
                bootstrap_mise()?
            }
        };

        if self.detect(language, version)?.is_none() {
            let ok = confirm.confirm(&format!(
                "Ruby {version} was not found. ManScript can install an isolated Ruby via mise into ~/.manscript (no sudo). Continue?"
            ))?;
            if !ok {
                return Err(ManscriptError::Cancelled);
            }
            install_ruby(&mise, version)?;
        }

        find_ruby_with_mise(&mise, version)?.ok_or_else(|| ManscriptError::RuntimeNotFound {
            language: "ruby".into(),
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

fn find_ruby_with_mise(mise: &Path, version: &str) -> Result<Option<Runtime>> {
    let spec = format!("ruby@{version}");
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
    let executable = dir.join("bin").join(exe_name("ruby"));
    if !executable.is_file() {
        return Ok(None);
    }
    let detected =
        probe_version(&executable, &["--version"])?.unwrap_or_else(|| version.to_string());
    if version_matches(&detected, version) {
        Ok(Some(Runtime {
            language: "ruby".into(),
            version: detected,
            executable,
            source: RuntimeSource::Provider("mise".into()),
        }))
    } else {
        Ok(None)
    }
}

fn install_ruby(mise: &Path, version: &str) -> Result<()> {
    ensure_dir(&mise_data_dir())?;
    let spec = format!("ruby@{version}");
    let mut cmd = Command::new(mise);
    cmd.args(["install", &spec]);
    mise_env(&mut cmd);
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(ManscriptError::Message(format!(
            "mise install {spec} failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

fn bootstrap_mise() -> Result<PathBuf> {
    let Some((target, bin_name)) = mise_download_target() else {
        return Err(ManscriptError::Message(
            "mise bootstrap is not supported on this platform yet. Install mise or Ruby manually and re-run.".into(),
        ));
    };
    let dest_dir = tools_dir().join("mise");
    ensure_dir(&dest_dir)?;
    let url = format!("https://github.com/jdx/mise/releases/latest/download/{target}");
    // GitHub asset names include version; try tarball naming used by mise.
    let tar_url = format!("{url}.tar.gz");
    let archive = dest_dir.join("mise-download.tar.gz");
    match download_to_file(&tar_url, &archive) {
        Ok(()) => {}
        Err(_) => {
            // Fallback: linux/mac binary without archive.
            let bin = dest_dir.join(bin_name);
            download_to_file(&url, &bin)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&bin)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&bin, perms)?;
            }
            return Ok(bin);
        }
    }
    let extract_dir = dest_dir.join("extract");
    let _ = std::fs::remove_dir_all(&extract_dir);
    extract_tar_gz(&archive, &extract_dir)?;
    let found = find_named_file(&extract_dir, bin_name).ok_or_else(|| {
        ManscriptError::Message("downloaded mise archive did not contain a mise binary".into())
    })?;
    let dest_bin = dest_dir.join(bin_name);
    std::fs::copy(&found, &dest_bin)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest_bin)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest_bin, perms)?;
    }
    let _ = std::fs::remove_file(&archive);
    let _ = std::fs::remove_dir_all(&extract_dir);
    Ok(dest_bin)
}
