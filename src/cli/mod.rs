use clap::{error::ErrorKind, Parser, Subcommand};

use crate::core::errors::Result;
use crate::core::registry::default_registry;
use crate::utils::output::{clap_styles, print_unknown_command, unrecognized_subcommand};

mod build;
mod create;
mod doctor;
mod env;
mod help;
mod init;
mod install;
mod run;
mod setup;
mod test;

#[derive(Parser, Debug)]
#[command(
    name = "manscript",
    version,
    styles = clap_styles(),
    about = "From zero to a running app, with fewer ritual sacrifices to PATH.",
    after_help = "Tip: `doctor` is a ManScript command, not a shell command.\n  manscript doctor\n  manscript -h",
    long_about = "ManScript prepares isolated development environments from project requirements.\n\nIt does not disable Python, Django, Ruby, or Rails. Those tools still exist.\nManScript just runs the copies inside this project's environment so you do not\nhave to activate a venv or remember the framework's favorite incantation.\n\n  manscript create django myproject\n  cd myproject\n  manscript run"
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
    /// New project, or add an app/module inside the current one
    Create {
        /// django / fastapi / … for a new project, or an app name if you are already in one
        framework: Option<String>,
        /// Project directory name
        name: Option<String>,
    },
    /// Write manscript.toml for an existing directory
    Init,
    /// Prepare runtime, environment, and dependencies
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
    /// Show this project's paths and interpreters
    Env,
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
                err.exit();
            }
            _ => err.exit(),
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
    }
}
