use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManscriptError {
    #[error("{0}")]
    Message(String),

    #[error("A project already exists at `{0}`.\n\nManScript will not overwrite it unless you confirm. Pick a new folder name, or remove that directory first.")]
    ProjectExists(PathBuf),

    #[error("This folder is not a ManScript project.\n\nI looked here and in parent folders for a file named manscript.toml, and did not find one.\n\nIf this is a new app, run:\n\n    manscript create django myproject\n\nIf the app already exists, cd into it (or run manscript init).")]
    ProjectNotFound,

    #[error("`{0}` is not a framework ManScript knows yet.\n\nFrameworks:\n  django, fastapi, flask, rails, sinatra\n\nLanguage only:\n  python, ruby, c, cpp, java\n\nExamples:\n\n    manscript create django myproject\n    manscript create python myapp\n    manscript create c hello")]
    UnknownFramework(String),

    #[error(
        "`{0}` is not a language ManScript knows yet.\n\nLanguages: python, ruby, c, cpp, java."
    )]
    UnknownLanguage(String),

    #[error("`{0}` is not a valid project name.\n\nUse only letters, numbers, hyphens, and underscores. Do not use spaces or paths like ../myapp.")]
    InvalidProjectName(String),

    #[error("That run command is not allowed.\n\n{0}\n\nManScript runs commands as a simple program plus arguments (not a shell), so things like sudo, |, &&, and $ are blocked.")]
    InvalidCommand(String),

    #[error("{language} {version} is needed for this project, but ManScript could not find it on this machine.\n\nManScript can prepare an isolated {language} runtime for you (no sudo).\n\nRun:\n\n    manscript setup")]
    RuntimeNotFound { language: String, version: String },

    #[error("The project environment is not ready yet.\n\nLooked in:\n  {0}\n\nPrepare it with:\n\n    manscript setup")]
    EnvironmentNotReady(PathBuf),

    #[error("Stopped. Nothing was changed.")]
    Cancelled,

    #[error("ManScript will not run sudo or raise privileges.\n\nInstall or change things only in your user directory, or run setup so ManScript can use an isolated runtime.")]
    SudoRefused,

    #[error("Could not read or write a file.\n\n{0}\n\nCheck that the path exists and that you have permission to use it.")]
    Io(String),

    #[error("manscript.toml is not valid.\n\n{0}\n\nOpen the file and check quotes, brackets, and section names like [language] and [commands].")]
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
