use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentKind {
    Venv,
    BundlerPath,
    Toolchain,
    Other,
}

impl EnvironmentKind {
    pub fn from_manager(name: &str) -> Self {
        match name {
            "venv" => Self::Venv,
            "bundler" => Self::BundlerPath,
            "toolchain" => Self::Toolchain,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub root: PathBuf,
    pub bin_dir: PathBuf,
    pub kind: EnvironmentKind,
}

#[derive(Debug, Clone)]
pub struct ShellEnvironment {
    pub path_prepend: Vec<PathBuf>,
    pub extra_env: HashMap<String, String>,
}

impl Environment {
    pub fn python_executable(&self) -> PathBuf {
        self.bin_dir.join(crate::utils::platform::python_bin_name())
    }
}
