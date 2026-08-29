use std::io::{self, IsTerminal, Write};

use clap::builder::PossibleValue;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

use super::Cli;
use crate::core::errors::Result;
use crate::utils::output::Printer;

pub fn execute(shell: Shell) -> Result<()> {
    if io::stdout().is_terminal() {
        let printer = Printer::new();
        printer.command_intro(
            "Completions",
            "Load generated completion code into the current shell session.",
        );
        printer.section("Enable for this terminal");
        printer.hint_command(activation_command(shell));
        printer.blank();
        return Ok(());
    }

    write_completion(shell, &mut io::stdout())
}

fn activation_command(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => "eval \"$(manscript completions bash)\"",
        Shell::Zsh => "eval \"$(manscript completions zsh)\"",
        Shell::Fish => "manscript completions fish | source",
        Shell::PowerShell => "manscript completions powershell | Out-String | Invoke-Expression",
        Shell::Elvish => "eval (manscript completions elvish | slurp)",
        _ => "manscript completions <shell> > completion-script",
    }
}

fn write_completion(shell: Shell, output: &mut dyn Write) -> Result<()> {
    if matches!(shell, Shell::Zsh) {
        writeln!(
            output,
            "if (( ! $+functions[compdef] )); then\n  autoload -Uz compinit\n  compinit\nfi"
        )?;
    }

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "manscript", output);
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
            "go", "rust", "php", "csharp",
        ];
        Some(Box::new(IDS.iter().map(|id| PossibleValue::new(*id))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zsh_completion_initializes_compdef() {
        let mut output = Vec::new();
        write_completion(Shell::Zsh, &mut output).unwrap();
        let script = String::from_utf8(output).unwrap();

        assert!(script.contains("autoload -Uz compinit"));
        assert!(script.contains("compinit"));
        assert!(script.contains("compdef _manscript manscript"));
    }

    #[test]
    fn zsh_activation_uses_command_substitution() {
        assert_eq!(
            activation_command(Shell::Zsh),
            "eval \"$(manscript completions zsh)\""
        );
    }
}
