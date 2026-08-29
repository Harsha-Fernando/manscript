use std::path::{Path, PathBuf};
use std::process::Command;

use crate::adapters::traits::ConfirmPolicy;
use crate::core::errors::{ManscriptError, Result};
use crate::core::runtime::{version_matches, Runtime, RuntimeSource};
use crate::runtime::download::{download_to_file, extract_tar_gz, find_named_file};
use crate::runtime::system::probe_version;
use crate::runtime::RuntimeProvider;
use crate::utils::filesystem::{ensure_dir, manscript_home, tools_dir};
use crate::utils::platform::{exe_name, uv_download_target};

pub struct UvPythonProvider;

impl RuntimeProvider for UvPythonProvider {
    fn id(&self) -> &'static str {
        "uv"
    }

    fn supports(&self, language: &str) -> bool {
        language == "python"
    }

    fn detect(&self, language: &str, version: &str) -> Result<Option<Runtime>> {
        if language != "python" {
            return Ok(None);
        }
        let Some(uv) = find_uv() else {
            return Ok(None);
        };
        find_python_with_uv(&uv, version)
    }

    fn prepare(&self, language: &str, version: &str, confirm: ConfirmPolicy) -> Result<Runtime> {
        if language != "python" {
            return Err(ManscriptError::UnknownLanguage(language.to_string()));
        }
        if let Some(existing) = self.detect(language, version)? {
            return Ok(existing);
        }

        let uv = match find_uv() {
            Some(p) => p,
            None => {
                let ok = confirm.confirm(
                    "uv is not installed. ManScript can download uv into a user-writable directory (~/.manscript/tools) without sudo. Continue?",
                )?;
                if !ok {
                    return Err(ManscriptError::Cancelled);
                }
                bootstrap_uv()?
            }
        };

        if self.detect(language, version)?.is_none() {
            let ok = confirm.confirm(&format!(
                "Python {version} was not found. ManScript can install an isolated CPython via uv into ~/.manscript (no sudo). Continue?"
            ))?;
            if !ok {
                return Err(ManscriptError::Cancelled);
            }
            install_python(&uv, version)?;
        }

        find_python_with_uv(&uv, version)?.ok_or_else(|| ManscriptError::RuntimeNotFound {
            language: "python".into(),
            version: version.to_string(),
        })
    }
}

fn uv_python_dir() -> PathBuf {
    manscript_home().join("runtimes").join("python")
}

pub fn find_uv() -> Option<PathBuf> {
    if let Ok(p) = which::which(exe_name("uv")) {
        return Some(p);
    }
    let local = tools_dir().join("uv").join(exe_name("uv"));
    if local.is_file() {
        return Some(local);
    }
    None
}

fn uv_env(cmd: &mut Command) {
    cmd.env("UV_PYTHON_INSTALL_DIR", uv_python_dir());
    cmd.env("UV_UNMANAGED_INSTALL", tools_dir().join("uv"));
}

fn find_python_with_uv(uv: &Path, version: &str) -> Result<Option<Runtime>> {
    let _ = ensure_dir(&uv_python_dir());
    let mut cmd = Command::new(uv);
    cmd.args(["python", "find", version]);
    uv_env(&mut cmd);
    let output = cmd.output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let path_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path_text.is_empty() {
        return Ok(None);
    }
    let executable = PathBuf::from(path_text);
    if !executable.is_file() {
        return Ok(None);
    }
    let detected = probe_version(&executable, &["-V"])?.unwrap_or_else(|| version.to_string());
    if version_matches(&detected, version) {
        Ok(Some(Runtime {
            language: "python".into(),
            version: detected,
            executable,
            source: RuntimeSource::Provider("uv".into()),
        }))
    } else {
        Ok(None)
    }
}

fn install_python(uv: &Path, version: &str) -> Result<()> {
    ensure_dir(&uv_python_dir())?;
    let mut cmd = Command::new(uv);
    cmd.args(["python", "install", version]);
    uv_env(&mut cmd);
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(ManscriptError::Message(format!(
            "uv python install {version} failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

fn bootstrap_uv() -> Result<PathBuf> {
    let Some((target, bin_name)) = uv_download_target() else {
        return Err(ManscriptError::Message(
            "uv bootstrap is not supported on this platform yet. Install uv manually and re-run."
                .into(),
        ));
    };
    let dest_dir = tools_dir().join("uv");
    ensure_dir(&dest_dir)?;
    let url = format!("https://github.com/astral-sh/uv/releases/latest/download/{target}.tar.gz");
    let archive = dest_dir.join("uv-download.tar.gz");
    download_to_file(&url, &archive)?;
    let extract_dir = dest_dir.join("extract");
    let _ = std::fs::remove_dir_all(&extract_dir);
    extract_tar_gz(&archive, &extract_dir)?;
    let found = find_named_file(&extract_dir, bin_name).ok_or_else(|| {
        ManscriptError::Message("downloaded uv archive did not contain a uv binary".into())
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
