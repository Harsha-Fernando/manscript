use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::core::errors::{ManscriptError, Result};

#[derive(Debug, Clone)]
pub struct PreparedCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub extra_env: HashMap<String, String>,
    pub path_prepend: Vec<PathBuf>,
}

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self
    }

    fn command(&self, prepared: &PreparedCommand) -> Result<Command> {
        validate_program(&prepared.program)?;
        let mut cmd = Command::new(&prepared.program);
        cmd.args(&prepared.args);
        cmd.current_dir(&prepared.cwd);
        for (k, v) in &prepared.extra_env {
            cmd.env(k, v);
        }
        if !prepared.path_prepend.is_empty() {
            let mut parts: Vec<String> = prepared
                .path_prepend
                .iter()
                .map(|p| p.display().to_string())
                .collect();
            if let Some(existing) = std::env::var_os("PATH") {
                parts.push(existing.to_string_lossy().into_owned());
            }
            cmd.env("PATH", parts.join(path_separator()));
        }
        Ok(cmd)
    }

    /// Run a user-facing command with inherited stdio (dev servers, tests).
    /// Do not wrap this in an indicatif spinner — it would fight the child's logs.
    pub fn run_inherit(&self, prepared: PreparedCommand) -> Result<i32> {
        let mut cmd = self.command(&prepared)?;
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        let status = cmd.status()?;
        Ok(status.code().unwrap_or(1))
    }

    pub fn run_capture(&self, prepared: PreparedCommand) -> Result<Output> {
        let mut cmd = self.command(&prepared)?;
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        Ok(cmd.output()?)
    }

    pub fn run_status(&self, prepared: PreparedCommand) -> Result<()> {
        let output = self.run_capture(prepared)?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(ManscriptError::Message(format!(
                "A tool ManScript started did not finish successfully (exit {}).\n\n{stdout}{stderr}\nIf this happened during setup, try:\n\n    manscript doctor\n    manscript setup",
                output.status.code().unwrap_or(1)
            )))
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

fn path_separator() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}

pub fn validate_program(program: &Path) -> Result<()> {
    let name = program
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name == "sudo" || name == "sudo.exe" {
        return Err(ManscriptError::SudoRefused);
    }
    Ok(())
}

/// Split a command string into argv without invoking a shell.
pub fn split_command_line(input: &str) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if in_single || in_double {
        return Err(ManscriptError::InvalidCommand(
            "unclosed quote in command".into(),
        ));
    }
    if !current.is_empty() {
        args.push(current);
    }
    if args.is_empty() {
        return Err(ManscriptError::InvalidCommand("empty command".into()));
    }
    for token in &args {
        let program_like =
            token.eq_ignore_ascii_case("sudo") || token.eq_ignore_ascii_case("sudo.exe");
        if program_like {
            return Err(ManscriptError::SudoRefused);
        }
        if token.contains("..") && args.first().map(|p| p.contains("..")).unwrap_or(false) {
            return Err(ManscriptError::InvalidCommand(
                "path traversal is not allowed in commands".into(),
            ));
        }
        if token.contains('|')
            || token.contains(';')
            || token.contains('&')
            || token.contains('>')
            || token.contains('<')
            || token.contains('`')
            || token.contains('$')
        {
            return Err(ManscriptError::InvalidCommand(
                "shell metacharacters are not allowed; ManScript runs argv without a shell".into(),
            ));
        }
    }
    if args[0].contains("..") {
        return Err(ManscriptError::InvalidCommand(
            "path traversal is not allowed in commands".into(),
        ));
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_simple() {
        assert_eq!(
            split_command_line("python manage.py runserver").unwrap(),
            vec!["python", "manage.py", "runserver"]
        );
    }

    #[test]
    fn rejects_sudo() {
        assert!(split_command_line("sudo python").is_err());
    }

    #[test]
    fn rejects_shell_meta() {
        assert!(split_command_line("python && rm -rf /").is_err());
    }
}
