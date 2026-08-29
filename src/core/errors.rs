use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManscriptError {
    #[error("{0}")]
    Message(String),

    #[error("A file or directory already exists at:\n  {0}\n\nManScript left it unchanged. Choose a different project name, or move/remove the existing path before trying again.")]
    ProjectExists(PathBuf),

    #[error("This folder is not inside a ManScript project.\n\nManScript looked in the current folder and its parents but could not find `manscript.toml`.\n\nTo configure an existing project, run:\n\n    manscript init\n    manscript setup\n\nTo create a new project, run:\n\n    manscript create")]
    ProjectNotFound,

    #[error("`{0}` is not a supported framework or language.\n\nSupported frameworks:\n  django, fastapi, flask, rails, sinatra\n\nLanguage-only projects:\n  python, ruby, c, cpp, java, go, rust, php, csharp\n\nExamples:\n\n    manscript create django myproject\n    manscript create python myapp\n    manscript create go hello")]
    UnknownFramework(String),

    #[error(
        "`{0}` is not a supported language.\n\nSupported languages:\n  python, ruby, c, cpp, java, go, rust, php, csharp"
    )]
    UnknownLanguage(String),

    #[error("`{0}` is not a valid project name.\n\nUse only letters, numbers, hyphens, and underscores. Enter a name, not a path; for example:\n\n    my-project")]
    InvalidProjectName(String),

    #[error("ManScript could not safely run that configured command.\n\n{0}\n\nCommands must contain one program followed by arguments. Shell operators such as `|`, `&&`, `$`, redirection, and `sudo` are not supported.")]
    InvalidCommand(String),

    #[error("This project requires {language} {version}, but ManScript could not find a compatible runtime.\n\nPrepare the required runtime and project environment with:\n\n    manscript setup")]
    RuntimeNotFound { language: String, version: String },

    #[error("The project environment is not ready.\n\nExpected it at:\n  {0}\n\nCreate the environment and install dependencies with:\n\n    manscript setup")]
    EnvironmentNotReady(PathBuf),

    #[error("Stopped. Nothing was changed.")]
    Cancelled,

    #[error("ManScript refused to run `sudo` or elevate privileges.\n\nProject commands must run with your current user permissions. Use `manscript setup` to prepare supported tools in user- or project-owned directories.")]
    SudoRefused,

    #[error("ManScript could not access a required file or directory.\n\nSystem detail:\n  {0}\n\nCheck that the path exists, is writable when required, and is accessible to your user account.")]
    Io(String),

    #[error("ManScript could not read `manscript.toml` because its TOML syntax is invalid.\n\nParser detail:\n  {0}\n\nCheck quotes, brackets, and section names such as `[language]` and `[commands]`, then try again.")]
    Toml(String),
}

impl From<io::Error> for ManscriptError {
    fn from(err: io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<toml::de::Error> for ManscriptError {
    fn from(err: toml::de::Error) -> Self {
        Self::Toml(err.to_string())
    }
}

impl From<toml::ser::Error> for ManscriptError {
    fn from(err: toml::ser::Error) -> Self {
        Self::Toml(err.to_string())
    }
}

impl ManscriptError {
    pub fn print(&self) {
        crate::utils::output::print_error_block(&self.to_string(), matches!(self, Self::Cancelled));
    }
}

pub type Result<T> = std::result::Result<T, ManscriptError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_not_found_explains_both_recovery_paths() {
        let message = ManscriptError::ProjectNotFound.to_string();
        assert!(message.contains("manscript.toml"));
        assert!(message.contains("manscript init"));
        assert!(message.contains("manscript setup"));
        assert!(message.contains("manscript create"));
    }

    #[test]
    fn environment_not_ready_names_path_and_next_step() {
        let message = ManscriptError::EnvironmentNotReady(PathBuf::from(".manscript/environment"))
            .to_string();
        assert!(message.contains(".manscript/environment"));
        assert!(message.contains("manscript setup"));
    }
}
