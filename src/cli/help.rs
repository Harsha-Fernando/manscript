use clap::{Command, CommandFactory};

use crate::utils::output::{print_unknown_command, Printer};

use super::Cli;

pub fn print_from_args(args: impl IntoIterator<Item = String>) {
    match help_topic(args) {
        None => print_root_help(),
        Some(name) => print_command_help(&name),
    }
}

pub fn print_version() {
    let printer = Printer::new();
    printer.line(&format!("manscript {}", env!("CARGO_PKG_VERSION")));
}

/// `manscript -h` → None; `manscript create -h` / `manscript help run` → Some(cmd).
pub fn help_topic(args: impl IntoIterator<Item = String>) -> Option<String> {
    let mut positional = args
        .into_iter()
        .filter(|a| a != "--" && !a.starts_with('-'));
    match positional.next().as_deref() {
        None => None,
        Some("help") => positional.next(),
        Some(cmd) => Some(cmd.to_string()),
    }
}

fn print_root_help() {
    let printer = Printer::new();
    let cmd = Cli::command();
    printer.info(&format!("ManScript  v{}", env!("CARGO_PKG_VERSION")));
    if let Some(about) = cmd.get_about() {
        printer.blank();
        printer.muted(&format!("  {about}"));
    }

    printer.section("Start here");
    printer.muted("  Choose the one that describes your project:");
    printer.blank();
    printer.muted("  Create a new project:");
    printer.hint_command("manscript create");
    printer.blank();
    printer.muted("  Add ManScript to an existing project:");
    printer.hint_command("cd your-project");
    printer.hint_command("manscript init");
    printer.hint_command("manscript setup");
    printer.blank();
    printer.muted("  Prepare a cloned project that already has manscript.toml:");
    printer.hint_command("cd your-project");
    printer.hint_command("manscript setup");

    print_command_group(
        &printer,
        &cmd,
        "Use your project",
        &["run", "shell", "test", "build"],
    );
    print_command_group(
        &printer,
        &cmd,
        "Set up your project",
        &["create", "init", "setup", "install"],
    );
    print_command_group(
        &printer,
        &cmd,
        "Inspect and configure",
        &["doctor", "env", "completions"],
    );

    printer.section("Help");
    printer.hint_command("manscript <command> --help");
    printer.muted("    Example: manscript create --help");
    printer.blank();
}

fn print_command_group(printer: &Printer, cmd: &Command, title: &str, names: &[&str]) {
    let rows: Vec<(&str, String)> = names
        .iter()
        .filter_map(|name| {
            cmd.find_subcommand(name).map(|sub| {
                (
                    *name,
                    sub.get_about()
                        .map(|about| about.to_string())
                        .unwrap_or_default(),
                )
            })
        })
        .collect();
    print_rows(printer, title, &rows);
}

fn print_command_help(name: &str) {
    let cmd = Cli::command();
    let Some(sub) = cmd.find_subcommand(name) else {
        print_unknown_command(name);
        std::process::exit(2);
    };

    let printer = Printer::new();
    printer.info(name);
    if let Some(about) = sub.get_about() {
        printer.blank();
        printer.muted(&format!("  {about}"));
    }

    printer.section("Usage:");
    printer.hint_command(&usage_line(sub));

    let args: Vec<(String, String)> = sub
        .get_positionals()
        .map(|arg| {
            let id = arg.get_id().as_str().to_string();
            let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();
            (id, help)
        })
        .collect();
    if !args.is_empty() {
        let refs: Vec<(&str, String)> = args.iter().map(|(n, a)| (n.as_str(), a.clone())).collect();
        print_rows(&printer, "Arguments:", &refs);
    }

    let opts: Vec<(String, String)> = sub
        .get_opts()
        .filter(|arg| !arg.is_hide_set())
        .map(|arg| {
            let mut flags = String::new();
            if let Some(c) = arg.get_short() {
                flags.push('-');
                flags.push(c);
            }
            if let Some(long) = arg.get_long() {
                if !flags.is_empty() {
                    flags.push_str(", ");
                }
                flags.push_str("--");
                flags.push_str(long);
            }
            if flags.is_empty() {
                flags = arg.get_id().as_str().to_string();
            }
            let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();
            (flags, help)
        })
        .collect();
    if !opts.is_empty() {
        let refs: Vec<(&str, String)> = opts.iter().map(|(n, a)| (n.as_str(), a.clone())).collect();
        print_rows(&printer, "Options:", &refs);
    }
    printer.blank();
}

fn usage_line(sub: &Command) -> String {
    let mut parts = vec!["manscript".to_string(), sub.get_name().to_string()];
    for arg in sub.get_positionals() {
        let id = arg.get_id().as_str();
        if arg.is_required_set() {
            parts.push(format!("<{id}>"));
        } else {
            parts.push(format!("[{id}]"));
        }
    }
    parts.join(" ")
}

fn print_rows(printer: &Printer, title: &str, rows: &[(&str, String)]) {
    printer.section(title);
    let width = rows.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
    for (name, about) in rows {
        printer.help_row(name, about, width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_help_has_no_topic() {
        assert_eq!(help_topic(["-h".into()]), None);
        assert_eq!(help_topic(["--help".into()]), None);
        assert_eq!(help_topic(["help".into()]), None);
    }

    #[test]
    fn subcommand_help_topic() {
        assert_eq!(
            help_topic(["create".into(), "-h".into()]),
            Some("create".into())
        );
        assert_eq!(
            help_topic(["help".into(), "run".into()]),
            Some("run".into())
        );
    }
}
