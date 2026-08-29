use std::path::{Path, PathBuf};

use crate::config::ProjectConfig;
use crate::core::errors::Result;
use crate::utils::filesystem::project_environment_dir;
use crate::utils::platform::env_bin_dir;

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub config: ProjectConfig,
}

impl Project {
    pub fn load(start: &Path) -> Result<Self> {
        let root = crate::config::find_project_root(start)?;
        let config = ProjectConfig::load(&crate::config::config_path(&root))?;
        Ok(Self { root, config })
    }

    pub fn environment_dir(&self) -> PathBuf {
        project_environment_dir(&self.root)
    }

    pub fn environment_bin_dir(&self) -> PathBuf {
        env_bin_dir(&self.environment_dir())
    }

    pub fn language(&self) -> &str {
        &self.config.language.name
    }

    pub fn language_version(&self) -> &str {
        &self.config.language.version
    }

    pub fn framework_name(&self) -> Option<&str> {
        self.config.framework.as_ref().map(|f| f.name.as_str())
    }
}
