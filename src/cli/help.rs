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
    print_wordmark(&printer);
    printer.info(&format!("ManScript  v{}", env!("CARGO_PKG_VERSION")));
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
    print_wordmark(&printer);
    printer.info(&format!("ManScript  v{}", env!("CARGO_PKG_VERSION")));
    if let Some(about) = cmd.get_about() {
        printer.blank();
        printer.muted(&format!("  {about}"));
    }

    printer.section("Usage:");
    printer.hint_command("manscript [options] [command]");

    let options: Vec<(&str, String)> = vec![
        ("-y, --yes", "Assume yes for confirmation prompts".into()),
        ("-h, --help", "Display help for a command".into()),
        ("-V, --version", "Display this application version".into()),
    ];
    print_rows(&printer, "Options:", &options);

    let mut commands: Vec<(String, String)> = cmd
        .get_subcommands()
        .filter(|s| !s.is_hide_set())
        .map(|s| {
            (
                s.get_name().to_string(),
                s.get_about().map(|a| a.to_string()).unwrap_or_default(),
            )
        })
        .collect();
    commands.sort_by(|a, b| a.0.cmp(&b.0));
    let command_refs: Vec<(&str, String)> = commands
        .iter()
        .map(|(n, a)| (n.as_str(), a.clone()))
        .collect();
    print_rows(&printer, "Available commands:", &command_refs);

    printer.blank();
    printer.info("Common workflows");
    printer.muted("  Create and run a new project:");
    printer.hint_command("manscript create django myproject");
    printer.hint_command("cd myproject && manscript run");
    printer.blank();
    printer.muted("  Prepare an existing ManScript project:");
    printer.hint_command("manscript setup");
    printer.blank();
    printer.muted("  Add something inside the current framework project:");
    printer.hint_command("manscript create blog");
    printer.blank();
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

fn print_wordmark(printer: &Printer) {
    printer.line("  ███╗   ███╗ █████╗ ███╗   ██╗███████╗ ██████╗██████╗ ██╗██████╗ ████████╗");
    printer.line("  ████╗ ████║██╔══██╗████╗  ██║██╔════╝██╔════╝██╔══██╗██║██╔══██╗╚══██╔══╝");
    printer.line("  ██╔████╔██║███████║██╔██╗ ██║███████╗██║     ██████╔╝██║██████╔╝   ██║   ");
    printer.line("  ██║╚██╔╝██║██╔══██║██║╚██╗██║╚════██║██║     ██╔══██╗██║██╔═══╝    ██║   ");
    printer.line("  ██║ ╚═╝ ██║██║  ██║██║ ╚████║███████║╚██████╗██║  ██║██║██║        ██║   ");
    printer.blank();
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
