use clap::{error::ErrorKind, Parser, Subcommand};
use clap_complete::Shell;

use crate::core::errors::Result;
use crate::core::registry::default_registry;
use crate::utils::output::{
    clap_styles, print_clap_error, print_missing_completion_shell, print_unknown_command,
    unrecognized_subcommand,
};

mod build;
mod completions;
mod create;
mod doctor;
mod env;
mod help;
mod init;
mod install;
mod run;
mod setup;
mod shell;
mod test;

#[derive(Parser, Debug)]
#[command(
    name = "manscript",
    version,
    styles = clap_styles(),
    about = "Set up and run isolated development environments with less manual configuration.",
    after_help = "Need help with your environment?\n  manscript doctor\n  manscript --help",
    long_about = "ManScript prepares isolated development environments from project requirements.\n\nYour system tools remain unchanged. ManScript runs project-managed tools directly,\nor opens a temporary development shell, so you do not need to activate an\nenvironment manually.\n\n  manscript create django myproject\n  cd myproject\n  manscript run\n  manscript shell"
)]
pub struct Cli {
    /// Assume yes for confirmation prompts (CI / non-interactive)
    #[arg(short = 'y', long = "yes", global = true)]
    pub yes: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a project, or add an app/module inside the current project
    Create {
        /// django / fastapi / … for a new project, or an app name if you are already in one
        #[arg(value_parser = completions::CreateFirstArgParser)]
        framework: Option<String>,
        /// Project directory name
        name: Option<String>,
    },
    /// Configure the current directory as a ManScript project
    Init,
    /// Prepare the runtime, project environment, and dependencies
    Setup,
    /// Install project dependencies into the managed environment
    Install,
    /// Run the project using the configured command
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run tests using the configured command
    Test {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Build the project if a build command is configured
    Build {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Diagnose the local development environment
    Doctor,
    /// Show the current project's resolved paths and tools
    Env,
    /// Open an interactive shell with the project environment on PATH
    Shell,
    /// Print a tab-completion script for your shell
    Completions {
        /// bash, zsh, fish, powershell, or elvish
        #[arg(value_enum)]
        shell: Shell,
    },
}

pub fn dispatch() -> Result<()> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => match err.kind() {
            ErrorKind::DisplayHelp => {
                help::print_from_args(std::env::args().skip(1));
                std::process::exit(0);
            }
            ErrorKind::DisplayVersion => {
                help::print_version();
                std::process::exit(0);
            }
            ErrorKind::InvalidSubcommand => {
                let name = unrecognized_subcommand(&err)
                    .or_else(|| std::env::args().skip(1).find(|a| !a.starts_with('-')));
                if let Some(name) = name {
                    print_unknown_command(&name);
                    std::process::exit(2);
                }
                let exit_code = err.exit_code();
                print_clap_error(&err);
                std::process::exit(exit_code);
            }
            ErrorKind::MissingRequiredArgument
                if std::env::args()
                    .skip(1)
                    .find(|a| !a.starts_with('-'))
                    .as_deref()
                    == Some("completions") =>
            {
                print_missing_completion_shell();
                std::process::exit(err.exit_code());
            }
            _ => {
                let exit_code = err.exit_code();
                print_clap_error(&err);
                std::process::exit(exit_code);
            }
        },
    };
    let registry = default_registry();
    match cli.command {
        None => {
            help::print_from_args(std::iter::empty::<String>());
            Ok(())
        }
        Some(Commands::Create { framework, name }) => {
            create::execute(&registry, framework, name, cli.yes)
        }
        Some(Commands::Init) => init::execute(&registry, cli.yes),
        Some(Commands::Setup) => setup::execute(&registry, cli.yes),
        Some(Commands::Install) => install::execute(&registry),
        Some(Commands::Run { args }) => run::execute_run(&registry, &args),
        Some(Commands::Test { args }) => test::execute(&registry, &args),
        Some(Commands::Build { args }) => build::execute(&registry, &args),
        Some(Commands::Doctor) => doctor::execute(&registry),
        Some(Commands::Env) => env::execute(&registry),
        Some(Commands::Shell) => shell::execute(&registry),
        Some(Commands::Completions { shell }) => completions::execute(shell),
    }
}
