//! System compiler/JDK shims under `.manscript/environment` (not a copied toolchain).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::environment::{Environment, EnvironmentKind};
use crate::core::errors::{ManscriptError, Result};
use crate::core::project::Project;
use crate::core::runtime::Runtime;
use crate::process::{split_command_line, Executor, PreparedCommand};
use crate::utils::filesystem::ensure_dir;
use crate::utils::platform::{env_bin_dir, exe_name};

pub fn create_toolchain_env(project: &Project, links: &[(&str, &Path)]) -> Result<Environment> {
    let root = project.environment_dir();
    let bin = env_bin_dir(&root);
    ensure_dir(&bin)?;
    let mut map = String::new();
    for (name, target) in links {
        let target = target
            .canonicalize()
            .unwrap_or_else(|_| target.to_path_buf());
        let target_text = target.to_string_lossy();
        if target_text.contains(['\n', '\r']) {
            return Err(ManscriptError::InvalidCommand(
                "toolchain path contains an unsupported control character".into(),
            ));
        }
        if name.contains('=') || name.contains('\n') || name.contains('\r') {
            return Err(ManscriptError::InvalidCommand(
                "invalid toolchain name".into(),
            ));
        }
        map.push_str(name);
        map.push('=');
        map.push_str(&target_text);
        map.push('\n');
        #[cfg(unix)]
        {
            let dest = bin.join(exe_name(name));
            if !is_under_root(&project.root, &dest)? {
                return Err(ManscriptError::InvalidCommand(
                    "refusing to write a toolchain shim outside the project".into(),
                ));
            }
            link_tool(&target, &dest)?;
        }
        #[cfg(windows)]
        {
            let dest = bin.join(format!("{name}.cmd"));
            if !is_under_root(&project.root, &dest)? {
                return Err(ManscriptError::InvalidCommand(
                    "refusing to write a toolchain shim outside the project".into(),
                ));
            }
            write_windows_wrapper(&target, &dest)?;
        }
    }
    crate::utils::filesystem::write_file(&tools_map_path(project), &map)?;
    Ok(Environment {
        bin_dir: bin,
        root,
        kind: EnvironmentKind::Toolchain,
    })
}

#[cfg(windows)]
fn write_windows_wrapper(target: &Path, dest: &Path) -> Result<()> {
    let escaped = target.to_string_lossy().replace('%', "%%");
    crate::utils::filesystem::write_file(dest, &format!("@echo off\r\n\"{escaped}\" %*\r\n"))
}

pub fn toolchain_ready(project: &Project, names: &[&str]) -> bool {
    let Ok(map) = read_tools_map(project) else {
        return false;
    };
    names
        .iter()
        .all(|n| map.get(*n).is_some_and(|p| p.is_file() || p.exists()))
}

pub fn no_packages() -> Result<()> {
    Err(ManscriptError::Message(
        "This language does not use a package manager through ManScript.\n\nAdd source files directly, then run:\n\n    manscript build\n    manscript run"
            .into(),
    ))
}

pub fn compile_to_app(project: &Project, compiler_shim: &str, source: &str) -> Result<()> {
    if source.contains('/') || source.contains('\\') || source.contains("..") {
        return Err(ManscriptError::InvalidCommand(
            "source file must be a name in the project root (no paths)".into(),
        ));
    }
    let src = project.root.join(source);
    if !src.is_file() {
        return Err(ManscriptError::Message(format!(
            "ManScript could not build the project because `{source}` is missing from the project root.\n\nAdd the file, then run `manscript build` or `manscript run` again."
        )));
    }
    let compiler = env_tool(project, compiler_shim)?;
    let out = project.environment_bin_dir().join(exe_name("app"));
    if !is_under_root(&project.root, &out)? {
        return Err(ManscriptError::InvalidCommand(
            "build output must stay inside the project".into(),
        ));
    }
    let prepared = PreparedCommand {
        program: compiler,
        args: vec!["-o".into(), out.display().to_string(), source.to_string()],
        cwd: project.root.clone(),
        extra_env: HashMap::new(),
        path_prepend: vec![project.environment_bin_dir()],
    };
    Executor::new().run_status(prepared)
}

pub fn javac_main(project: &Project) -> Result<()> {
    let javac = env_tool(project, "javac")?;
    let src = project.root.join("Main.java");
    if !src.is_file() {
        return Err(ManscriptError::Message(
            "ManScript could not build the Java project because `Main.java` is missing from the project root.\n\nAdd `Main.java`, then run `manscript build` or `manscript run` again."
                .into(),
        ));
    }
    let prepared = PreparedCommand {
        program: javac,
        args: vec!["Main.java".into()],
        cwd: project.root.clone(),
        extra_env: HashMap::new(),
        path_prepend: vec![project.environment_bin_dir()],
    };
    Executor::new().run_status(prepared)
}

pub fn resolve_env_command(
    project: &Project,
    command: &str,
    extra_args: &[String],
) -> Result<PreparedCommand> {
    let mut argv = split_command_line(command)?;
    argv.extend(extra_args.iter().cloned());
    if argv.is_empty() {
        return Err(ManscriptError::InvalidCommand("empty command".into()));
    }
    if !project.environment_bin_dir().is_dir() {
        return Err(ManscriptError::EnvironmentNotReady(
            project.environment_dir(),
        ));
    }
    let program_name = argv.remove(0);
    let program = env_tool(project, &program_name)?;
    Ok(PreparedCommand {
        program,
        args: argv,
        cwd: project.root.clone(),
        extra_env: HashMap::new(),
        path_prepend: vec![project.environment_bin_dir()],
    })
}

pub fn env_tool(project: &Project, name: &str) -> Result<PathBuf> {
    if let Ok(map) = read_tools_map(project) {
        if let Some(path) = map.get(name) {
            if path.exists() {
                return Ok(path.clone());
            }
        }
    }
    let path = project.environment_bin_dir().join(exe_name(name));
    if path.exists() {
        Ok(path)
    } else {
        Err(ManscriptError::InvalidCommand(format!(
            "could not find `{name}` in the project environment; run `manscript setup` and check the configured command"
        )))
    }
}

fn tools_map_path(project: &Project) -> PathBuf {
    project.environment_dir().join("tools")
}

fn read_tools_map(project: &Project) -> Result<HashMap<String, PathBuf>> {
    let text = fs::read_to_string(tools_map_path(project))?;
    let mut map = HashMap::new();
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.is_empty() || v.is_empty() {
            continue;
        }
        map.insert(k.to_string(), PathBuf::from(v));
    }
    Ok(map)
}

pub fn sibling_or_which(tool: &Path, name: &str) -> Result<PathBuf> {
    if let Some(dir) = tool.parent() {
        let sib = dir.join(exe_name(name));
        if sib.is_file() {
            return Ok(sib);
        }
    }
    which::which(exe_name(name)).map_err(|_| ManscriptError::RuntimeNotFound {
        language: name.to_string(),
        version: "system".into(),
    })
}

pub fn find_cargo_for_rustc(rustc: &Path) -> Result<PathBuf> {
    if let Ok(output) = Command::new("rustup").args(["which", "cargo"]).output() {
        if output.status.success() {
            let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
            if path.is_file() {
                return Ok(path);
            }
        }
    }

    if let Some(dir) = rustc.parent() {
        let cargo = dir.join(exe_name("cargo"));
        if cargo.is_file() && !is_rustup_proxy_name(&cargo) {
            return Ok(cargo);
        }
    }

    let cargo = which::which(exe_name("cargo")).map_err(|_| ManscriptError::RuntimeNotFound {
        language: "cargo".into(),
        version: "system".into(),
    })?;
    if is_rustup_proxy_name(&cargo) {
        return Err(ManscriptError::Message(
            "ManScript found a rustup proxy instead of a concrete `cargo` executable.\n\nInstall the Rust toolchain with `rustup` or ensure `cargo` resolves to the toolchain binary, then run `manscript setup` again."
                .into(),
        ));
    }
    Ok(cargo)
}

fn is_rustup_proxy_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains("rustup"))
}

fn is_under_root(root: &Path, path: &Path) -> Result<bool> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let parent = path.parent().unwrap_or(path);
    if parent.exists() {
        let parent = parent.canonicalize()?;
        Ok(parent.starts_with(&root))
    } else {
        Ok(path.starts_with(root))
    }
}

#[cfg(unix)]
fn link_tool(target: &Path, dest: &Path) -> Result<()> {
    let target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
    if dest.exists() || dest.symlink_metadata().is_ok() {
        fs::remove_file(dest)?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, dest)?;
    }
    Ok(())
}

pub fn from_runtime(project: &Project, runtime: &Runtime, shim_name: &str) -> Result<Environment> {
    create_toolchain_env(project, &[(shim_name, runtime.executable.as_path())])
}
