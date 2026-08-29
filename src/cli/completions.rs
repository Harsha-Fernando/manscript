use std::io;

use clap::builder::PossibleValue;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

use super::Cli;
use crate::core::errors::Result;

pub fn execute(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "manscript", &mut io::stdout());
    Ok(())
}

/// First `create` argument: suggest known stacks, still accept any string (in-project names).
#[derive(Clone, Debug)]
pub struct CreateFirstArgParser;

impl clap::builder::TypedValueParser for CreateFirstArgParser {
    type Value = String;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> std::result::Result<Self::Value, clap::Error> {
        Ok(value.to_string_lossy().into_owned())
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        const IDS: &[&str] = &[
            "django", "fastapi", "flask", "rails", "sinatra", "python", "ruby", "c", "cpp", "java",
        ];
        Some(Box::new(IDS.iter().map(|id| PossibleValue::new(*id))))
    }
}
