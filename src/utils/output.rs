use std::cell::Cell;
use std::io::{self, IsTerminal, Write};
use std::time::Duration;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use owo_colors::OwoColorize;

const SUBCOMMANDS: &[&str] = &[
    "create",
    "init",
    "setup",
    "install",
    "run",
    "test",
    "build",
    "doctor",
    "env",
    "shell",
    "completions",
    "help",
];

thread_local! {
    static PROGRESS_DEPTH: Cell<usize> = const { Cell::new(0) };
}

fn enter_progress() {
    PROGRESS_DEPTH.with(|c| c.set(c.get() + 1));
}

fn leave_progress() {
    PROGRESS_DEPTH.with(|c| c.set(c.get().saturating_sub(1)));
}

#[derive(Debug, Default)]
pub struct Printer {
    quiet: bool,
}

fn stdout_color() -> bool {
    color_wanted() && io::stdout().is_terminal()
}

fn stderr_color() -> bool {
    color_wanted() && io::stderr().is_terminal()
}

fn color_wanted() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::env::var("TERM").ok().as_deref() != Some("dumb")
}

impl Printer {
    pub fn new() -> Self {
        Self { quiet: false }
    }

    pub fn line(&self, msg: &str) {
        if self.quiet {
            return;
        }
        println!("{msg}");
    }

    pub fn muted(&self, msg: &str) {
        if self.quiet {
            return;
        }
        if stdout_color() {
            println!("{}", msg.dimmed());
        } else {
            println!("{msg}");
        }
    }

    pub fn blank(&self) {
        self.line("");
    }

    pub fn step(&self, index: usize, total: usize, title: &str) {
        let label = format!("[{index}/{total}]");
        if stdout_color() {
            println!("{}  {}", label.cyan().bold(), title.bold());
        } else {
            println!("{label}  {title}");
        }
    }

    pub fn ok(&self, msg: &str) {
        if stdout_color() {
            println!("      {} {msg}", "✓".green().bold());
        } else {
            println!("      ✓ {msg}");
        }
    }

    pub fn fail(&self, msg: &str) {
        if stdout_color() {
            println!("      {} {msg}", "✗".red().bold());
        } else {
            println!("      ✗ {msg}");
        }
    }

    pub fn check_ok(&self, label: &str, detail: &str) {
        if stdout_color() {
            println!("  {} {label}", "✓".green().bold());
        } else {
            println!("  ✓ {label}");
        }
        if !detail.is_empty() {
            self.muted(&format!("    {detail}"));
        }
    }

    pub fn check_fail(&self, label: &str, detail: &str) {
        if stdout_color() {
            println!("  {} {label}", "✗".red().bold());
        } else {
            println!("  ✗ {label}");
        }
        if !detail.is_empty() {
            self.muted(&format!("    {detail}"));
        }
    }

    pub fn heading(&self, title: &str) {
        self.blank();
        if stdout_color() {
            println!("{}", title.bright_magenta().bold());
        } else {
            println!("{title}");
        }
    }

    /// Laravel-style section label (`Usage:`, `Available commands:`).
    pub fn section(&self, title: &str) {
        self.blank();
        if stdout_color() {
            println!("{}", title.yellow().bold());
        } else {
            println!("{title}");
        }
    }

    /// Aligned name + description, like artisan's command list.
    pub fn help_row(&self, name: &str, about: &str, name_width: usize) {
        let name_col = format!("{name:<name_width$}");
        if stdout_color() {
            println!("  {}  {}", name_col.cyan().bold(), about.dimmed());
        } else {
            println!("  {name_col}  {about}");
        }
    }

    pub fn info(&self, msg: &str) {
        if self.quiet {
            return;
        }
        println_tag(" INFO ", TagKind::Info, msg);
    }

    pub fn warn(&self, msg: &str) {
        if self.quiet {
            return;
        }
        println_tag(" WARN ", TagKind::Warn, msg);
    }

    pub fn success(&self, msg: &str) {
        if self.quiet {
            return;
        }
        println_tag("  DONE ", TagKind::Ok, msg);
    }

    pub fn hint_command(&self, line: &str) {
        if self.quiet {
            return;
        }
        if stdout_color() {
            println!("    {}", line.cyan().bold());
        } else {
            println!("    {line}");
        }
    }

    pub fn key_value(&self, label: &str, value: &str) {
        if self.quiet {
            return;
        }
        if stdout_color() {
            println!("  {:<12} {}", label.bold(), value);
        } else {
            println!("  {label:<12} {value}");
        }
    }

    pub fn next_steps(&self, commands: &[&str]) {
        if commands.is_empty() || self.quiet {
            return;
        }
        self.section(if commands.len() == 1 {
            "Next step:"
        } else {
            "Next steps:"
        });
        for command in commands {
            self.hint_command(command);
        }
    }

    pub fn url(&self, url: &str) {
        if stdout_color() {
            println!("{}", url.cyan().underline());
        } else {
            println!("{url}");
        }
    }

    pub fn flush(&self) {
        let _ = io::stdout().flush();
    }

    /// True when we may animate (TTY, colors wanted, not quiet).
    pub fn animates(&self) -> bool {
        !self.quiet && stdout_color()
    }

    /// Spinner for a long, trusted wait. `message` must be a ManScript-authored
    /// label only — never child stdout, URLs, tokens, or env dumps.
    pub fn spinner(&self, message: &str) -> SpinnerGuard {
        if self.quiet {
            return SpinnerGuard::noop();
        }
        if self.animates() {
            let bar = ProgressBar::new_spinner();
            bar.set_draw_target(ProgressDrawTarget::stdout());
            bar.set_style(spinner_style());
            bar.set_message(message.to_string());
            bar.enable_steady_tick(Duration::from_millis(80));
            enter_progress();
            SpinnerGuard {
                bar: Some(bar),
                static_message: None,
                done: false,
                held_depth: true,
            }
        } else {
            println!("  {message}...");
            enter_progress();
            SpinnerGuard {
                bar: None,
                static_message: Some(message.to_string()),
                done: false,
                held_depth: true,
            }
        }
    }

    /// Stepped progress for `create` (and similar). Same message rule as [`Self::spinner`].
    pub fn steps(&self, total: u64) -> StepBar<'_> {
        let animate = self.animates();
        let bar = if animate {
            let bar = ProgressBar::with_draw_target(Some(total), ProgressDrawTarget::stdout());
            bar.set_style(step_style());
            bar.enable_steady_tick(Duration::from_millis(80));
            enter_progress();
            Some(bar)
        } else {
            enter_progress();
            None
        };
        StepBar {
            printer: self,
            bar,
            pos: 0,
            total,
            done: false,
            held_depth: true,
        }
    }
}

/// Clears the bar on drop unless `finish_ok` ran — so `?` cannot leave a stuck spinner.
pub struct SpinnerGuard {
    bar: Option<ProgressBar>,
    static_message: Option<String>,
    done: bool,
    held_depth: bool,
}

impl SpinnerGuard {
    fn noop() -> Self {
        Self {
            bar: None,
            static_message: None,
            done: true,
            held_depth: false,
        }
    }

    fn release_depth(&mut self) {
        if self.held_depth {
            leave_progress();
            self.held_depth = false;
        }
    }

    pub fn finish_ok(mut self, detail: &str) {
        self.done = true;
        self.release_depth();
        if let Some(bar) = self.bar.take() {
            bar.finish_with_message(format!("✓ {detail}"));
        } else if self.static_message.is_some() {
            println!("      ✓ {detail}");
        }
    }
}

impl Drop for SpinnerGuard {
    fn drop(&mut self) {
        self.release_depth();
        if self.done {
            return;
        }
        if let Some(bar) = self.bar.take() {
            bar.finish_and_clear();
        }
    }
}

pub struct StepBar<'a> {
    printer: &'a Printer,
    bar: Option<ProgressBar>,
    pos: u64,
    total: u64,
    done: bool,
    held_depth: bool,
}

impl StepBar<'_> {
    pub fn begin(&mut self, title: &str) {
        self.pos = self.pos.saturating_add(1);
        if let Some(bar) = &self.bar {
            bar.set_position(self.pos.saturating_sub(1));
            bar.set_message(title.to_string());
        } else {
            self.printer
                .step(self.pos as usize, self.total as usize, title);
        }
    }

    pub fn ok(&self, detail: &str) {
        if let Some(bar) = &self.bar {
            bar.println(format!("      ✓ {detail}"));
        } else {
            self.printer.ok(detail);
        }
    }

    pub fn fail(&self, detail: &str) {
        if let Some(bar) = &self.bar {
            bar.println(format!("      ✗ {detail}"));
        } else {
            self.printer.fail(detail);
        }
    }

    pub fn note(&self, detail: &str) {
        if let Some(bar) = &self.bar {
            bar.println(format!("      → {detail}"));
        } else {
            self.printer.muted(&format!("      → {detail}"));
        }
    }

    pub fn finish(mut self) {
        self.done = true;
        if self.held_depth {
            leave_progress();
            self.held_depth = false;
        }
        if let Some(bar) = self.bar.take() {
            bar.finish_and_clear();
        }
    }
}

impl Drop for StepBar<'_> {
    fn drop(&mut self) {
        if self.held_depth {
            leave_progress();
            self.held_depth = false;
        }
        if self.done {
            return;
        }
        if let Some(bar) = self.bar.take() {
            bar.finish_and_clear();
        }
    }
}

/// Spinner for toolchain downloads. Skips if a parent spinner/step bar is already running
/// so we never nest bars or put the download URL in the live UI.
pub fn download_spinner() -> SpinnerGuard {
    if PROGRESS_DEPTH.with(|c| c.get()) > 0 {
        SpinnerGuard::noop()
    } else {
        Printer::new().spinner("Downloading a tool into ~/.manscript")
    }
}

fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.cyan} {msg}")
        .expect("spinner template")
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"])
}

fn step_style() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.cyan} [{pos}/{len}] {msg}")
        .expect("step template")
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"])
}

enum TagKind {
    Ok,
    Info,
    Warn,
    Err,
}

fn println_tag(tag: &str, kind: TagKind, msg: &str) {
    if stdout_color() {
        println!("{} {}", color_tag(tag, kind), msg);
    } else {
        println!("{tag} {msg}");
    }
}

fn eprintln_tag(tag: &str, kind: TagKind, msg: &str, use_stdout: bool) {
    let line = if stderr_color() || (use_stdout && stdout_color()) {
        format!("{} {}", color_tag(tag, kind), msg)
    } else {
        format!("{tag} {msg}")
    };
    if use_stdout {
        println!("{line}");
    } else {
        eprintln!("{line}");
    }
}

fn color_tag(tag: &str, kind: TagKind) -> String {
    match kind {
        TagKind::Ok => tag.on_green().black().bold().to_string(),
        TagKind::Info => tag.on_cyan().black().bold().to_string(),
        TagKind::Warn => tag.on_yellow().black().bold().to_string(),
        TagKind::Err => tag.on_red().white().bold().to_string(),
    }
}

pub fn print_error_block(body: &str, cancelled: bool) {
    eprintln!();
    if cancelled {
        eprintln_tag(
            "  WAIT ",
            TagKind::Warn,
            "Stopped. Nothing was changed.",
            false,
        );
        eprintln!();
        return;
    }
    eprintln_tag(
        " ERROR ",
        TagKind::Err,
        "ManScript could not complete that request.",
        false,
    );
    eprintln!();
    for line in body.lines() {
        eprintln!("  {line}");
    }
    eprintln!();
}

pub fn print_clap_error(err: &clap::Error) {
    let plain = strip_ansi(&err.to_string());
    let body = plain.strip_prefix("error: ").unwrap_or(&plain).trim_end();
    print_error_block(body, false);
}

pub fn print_unknown_command(name: &str) {
    eprintln!();
    eprintln_tag(
        " ERROR ",
        TagKind::Err,
        &format!("`{name}` is not a ManScript command."),
        false,
    );
    eprintln!();
    if let Some(hint) = suggest_command(name) {
        eprintln!("  Did you mean:");
        if stderr_color() {
            eprintln!("    {}", format!("manscript {hint}").cyan().bold());
        } else {
            eprintln!("    manscript {hint}");
        }
        eprintln!();
    }
    eprintln!("  See all available commands:");
    if stderr_color() {
        eprintln!("    {}", "manscript --help".cyan().bold());
    } else {
        eprintln!("    manscript --help");
    }
    eprintln!();
}

pub fn suggest_command(input: &str) -> Option<&'static str> {
    let input = input.to_ascii_lowercase();
    SUBCOMMANDS
        .iter()
        .copied()
        .map(|cmd| (cmd, strsim::levenshtein(&input, cmd)))
        .filter(|(_, d)| *d > 0 && *d <= 3)
        .min_by_key(|(_, d)| *d)
        .map(|(cmd, _)| cmd)
}

pub fn clap_styles() -> clap::builder::styling::Styles {
    use clap::builder::styling::{AnsiColor, Effects, Styles};
    Styles::styled()
        .header(AnsiColor::Magenta.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
        .placeholder(AnsiColor::BrightBlack.on_default())
        .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
        .valid(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
}

/// Best-effort parse of clap's "unrecognized subcommand 'foo'".
pub fn unrecognized_subcommand(err: &clap::Error) -> Option<String> {
    let plain = strip_ansi(&err.to_string());
    for marker in ["unrecognized subcommand '", "unrecognized subcommand `"] {
        if let Some(start) = plain.find(marker) {
            let rest = &plain[start + marker.len()..];
            let end = rest.find(['\'', '`'])?;
            let name = rest[..end].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for x in chars.by_ref() {
                    if x.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggests_doctor() {
        assert_eq!(suggest_command("docter"), Some("doctor"));
        assert_eq!(suggest_command("create"), None);
    }

    #[test]
    fn spinner_guard_finishes_without_panic() {
        let printer = Printer::new();
        let spin = printer.spinner("Preparing runtime");
        spin.finish_ok("Runtime prepared");
        let spin = printer.spinner("Installing dependencies");
        drop(spin);
    }
}
