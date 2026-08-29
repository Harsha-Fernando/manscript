use std::io::{self, IsTerminal, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    terminal::{self, Clear, ClearType},
};
use owo_colors::OwoColorize;

use crate::core::errors::{ManscriptError, Result};

const BOX_WIDTH: usize = 56;
const HINT: &str = "↑↓ move  ·  enter select  ·  esc cancel";

#[derive(Clone, Copy)]
pub struct Choice<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub hint: &'a str,
}

pub fn select(title: &str, choices: &[Choice<'_>], start: usize) -> Result<String> {
    if choices.is_empty() {
        return Err(ManscriptError::Message(
            "ManScript could not show this selection because no options are available.".into(),
        ));
    }
    if !io::stdin().is_terminal() {
        return Err(ManscriptError::Cancelled);
    }
    let mut idx = start.min(choices.len() - 1);
    let answer = with_raw(|out, last_h| loop {
        let lines = render_select_frame(title, choices, idx, true);
        redraw(out, last_h, &lines)?;
        match read_key()? {
            Key::Up | Key::Char('k') => {
                idx = if idx == 0 { choices.len() - 1 } else { idx - 1 };
            }
            Key::Down | Key::Char('j') => {
                idx = (idx + 1) % choices.len();
            }
            Key::Enter => {
                let chosen = &choices[idx];
                let done = render_select_frame(title, choices, idx, false);
                redraw(out, last_h, &done)?;
                return Ok(chosen.id.to_string());
            }
            Key::Cancel => return Err(ManscriptError::Cancelled),
            Key::Char(_) | Key::Backspace | Key::Other => {}
        }
    })?;
    println!();
    Ok(answer)
}

pub fn text(title: &str, placeholder: &str, hint: &str) -> Result<String> {
    if !io::stdin().is_terminal() {
        return Err(ManscriptError::Cancelled);
    }
    let mut value = String::new();
    let answer = with_raw(|out, last_h| loop {
        let lines = render_text_frame(title, &value, placeholder, hint, true);
        redraw(out, last_h, &lines)?;
        match read_key()? {
            Key::Enter => {
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                let done = render_text_frame(title, &trimmed, placeholder, "", false);
                redraw(out, last_h, &done)?;
                return Ok(trimmed);
            }
            Key::Cancel => return Err(ManscriptError::Cancelled),
            Key::Backspace => {
                value.pop();
            }
            Key::Char(c) => value.push(c),
            Key::Up | Key::Down | Key::Other => {}
        }
    })?;
    println!();
    Ok(answer)
}

pub fn confirm(message: &str, default_yes: bool) -> Result<bool> {
    let start = if default_yes { 0 } else { 1 };
    let id = select(
        message,
        &[
            Choice {
                id: "yes",
                label: "Yes",
                hint: "",
            },
            Choice {
                id: "no",
                label: "No",
                hint: "",
            },
        ],
        start,
    )?;
    Ok(id == "yes")
}

enum Key {
    Up,
    Down,
    Enter,
    Cancel,
    Backspace,
    Char(char),
    Other,
}

fn read_key() -> Result<Key> {
    loop {
        match event::read().map_err(|e| ManscriptError::Io(e.to_string()))? {
            Event::Key(k) if k.kind == KeyEventKind::Press || k.kind == KeyEventKind::Repeat => {
                if k.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(k.code, KeyCode::Char('c') | KeyCode::Char('C'))
                {
                    return Ok(Key::Cancel);
                }
                return Ok(match k.code {
                    KeyCode::Up => Key::Up,
                    KeyCode::Down => Key::Down,
                    KeyCode::Enter => Key::Enter,
                    KeyCode::Esc => Key::Cancel,
                    KeyCode::Backspace => Key::Backspace,
                    KeyCode::Char(c) => Key::Char(c),
                    _ => Key::Other,
                });
            }
            _ => {}
        }
    }
}

struct RawGuard;

impl Drop for RawGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show);
    }
}

fn with_raw<T>(mut body: impl FnMut(&mut io::Stdout, &mut usize) -> Result<T>) -> Result<T> {
    terminal::enable_raw_mode().map_err(|e| ManscriptError::Io(e.to_string()))?;
    let _guard = RawGuard;
    let mut out = io::stdout();
    execute!(out, cursor::Hide).map_err(|e| ManscriptError::Io(e.to_string()))?;
    let mut last_h = 0usize;
    let result = body(&mut out, &mut last_h);
    execute!(out, cursor::Show).ok();
    result
}

fn redraw(out: &mut io::Stdout, last_h: &mut usize, lines: &[String]) -> Result<()> {
    if *last_h > 0 {
        queue!(
            out,
            cursor::MoveToColumn(0),
            cursor::MoveUp(*last_h as u16),
            Clear(ClearType::FromCursorDown)
        )
        .map_err(|e| ManscriptError::Io(e.to_string()))?;
    }
    for (i, line) in lines.iter().enumerate() {
        queue!(out, cursor::MoveToColumn(0), crossterm::style::Print(line))
            .map_err(|e| ManscriptError::Io(e.to_string()))?;
        if i + 1 < lines.len() {
            queue!(out, crossterm::style::Print("\r\n"))
                .map_err(|e| ManscriptError::Io(e.to_string()))?;
        }
    }
    out.flush().map_err(|e| ManscriptError::Io(e.to_string()))?;
    *last_h = lines.len().saturating_sub(1);
    Ok(())
}

fn render_select_frame(title: &str, choices: &[Choice<'_>], idx: usize, live: bool) -> Vec<String> {
    let mut body = Vec::new();
    if live {
        for (i, c) in choices.iter().enumerate() {
            let selected = i == idx;
            let row = if c.hint.is_empty() {
                c.label.to_string()
            } else {
                format!("{}  {}", c.label, c.hint)
            };
            body.push(option_row(&row, selected, live));
        }
    } else {
        let c = &choices[idx];
        body.push(option_row(c.label, true, false));
    }
    let mut lines = paint_box(title, &body, live);
    if live {
        lines.push(dim_line(&format!(" {HINT}")));
    }
    lines
}

fn render_text_frame(
    title: &str,
    value: &str,
    placeholder: &str,
    hint: &str,
    live: bool,
) -> Vec<String> {
    let body = if value.is_empty() && live {
        vec![dim_text(&fit(placeholder, BOX_WIDTH.saturating_sub(4)))]
    } else if live {
        vec![format!("{}█", value.cyan())]
    } else {
        vec![value.cyan().to_string()]
    };
    let mut lines = paint_box(title, &body, live);
    if live && !hint.is_empty() {
        lines.push(dim_line(&format!(" {hint}")));
    }
    lines
}

fn option_row(text: &str, selected: bool, live: bool) -> String {
    let inner_w = BOX_WIDTH.saturating_sub(4);
    let prefix = if selected && live { "› " } else { "  " };
    let room = inner_w.saturating_sub(prefix.chars().count());
    let content = fit(text, room);
    let row = format!("{prefix}{content}");
    if selected {
        row.cyan().to_string()
    } else {
        dim_text(&row)
    }
}

fn paint_box(title: &str, body: &[String], live: bool) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(color_border(&box_top(title), live));
    for row in body {
        let inner = visible_pad(row, BOX_WIDTH.saturating_sub(4));
        lines.push(format!("{} {inner} {}", dim_text(" │"), dim_text("│")));
    }
    lines.push(color_border(&box_bottom(), live));
    lines
}

fn box_top(title: &str) -> String {
    let label = format!(" {title} ");
    let inner = BOX_WIDTH.saturating_sub(2);
    let fill = inner.saturating_sub(label.chars().count());
    format!(" ┌{label}{}┐", "─".repeat(fill))
}

fn box_bottom() -> String {
    format!(" └{}┘", "─".repeat(BOX_WIDTH.saturating_sub(2)))
}

fn color_border(s: &str, _live: bool) -> String {
    dim_text(s)
}

fn visible_pad(colored: &str, width: usize) -> String {
    let vis = plain(colored).chars().count();
    if vis >= width {
        colored.to_string()
    } else {
        format!("{colored}{}", " ".repeat(width - vis))
    }
}

fn fit(s: &str, width: usize) -> String {
    let n = s.chars().count();
    if n <= width {
        format!("{s}{}", " ".repeat(width - n))
    } else if width <= 1 {
        "…".into()
    } else {
        let take = width.saturating_sub(1);
        format!("{}…", s.chars().take(take).collect::<String>())
    }
}

fn plain(s: &str) -> String {
    let mut out = String::new();
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

fn dim_text(s: &str) -> String {
    s.dimmed().to_string()
}

fn dim_line(s: &str) -> String {
    dim_text(s)
}

pub fn pretty_language(id: &str) -> String {
    match id {
        "python" => "Python   — virtual environments and the Python ecosystem".into(),
        "ruby" => "Ruby     — Bundler, gems, and Ruby applications".into(),
        "c" => "C        — native compiled applications".into(),
        "cpp" => "C++      — native C++ applications".into(),
        "java" => "Java     — JDK-based applications".into(),
        other => other.to_string(),
    }
}

pub fn language_choice(id: &str) -> Choice<'static> {
    match id {
        "python" => Choice {
            id: "python",
            label: "Python",
            hint: "virtual environments and the Python ecosystem",
        },
        "ruby" => Choice {
            id: "ruby",
            label: "Ruby",
            hint: "Bundler, gems, and Ruby applications",
        },
        "c" => Choice {
            id: "c",
            label: "C",
            hint: "native compiled applications",
        },
        "cpp" => Choice {
            id: "cpp",
            label: "C++",
            hint: "native C++ applications",
        },
        "java" => Choice {
            id: "java",
            label: "Java",
            hint: "JDK-based applications",
        },
        _ => Choice {
            id: "python",
            label: "Unknown",
            hint: "",
        },
    }
}

pub fn language_picker_rank(id: &str) -> u8 {
    match id {
        "python" => 0,
        "ruby" => 1,
        _ => 10,
    }
}

pub fn language_id_from_choice(choice: &str) -> String {
    choice
        .split_whitespace()
        .next()
        .unwrap_or(choice)
        .to_ascii_lowercase()
}

pub fn pretty_framework(id: &str) -> String {
    match id {
        "django" => "Django    — full-stack Python web framework".into(),
        "fastapi" => "FastAPI   — typed Python APIs with automatic documentation".into(),
        "flask" => "Flask     — lightweight Python web applications".into(),
        "rails" => "Rails     — full-stack Ruby web framework".into(),
        "sinatra" => "Sinatra   — lightweight Ruby web applications".into(),
        "none" => "None      — language-only project".into(),
        other => other.to_string(),
    }
}

pub fn framework_choice(id: &str) -> Choice<'static> {
    match id {
        "django" => Choice {
            id: "django",
            label: "Django",
            hint: "full-stack Python web framework",
        },
        "fastapi" => Choice {
            id: "fastapi",
            label: "FastAPI",
            hint: "typed APIs with automatic documentation",
        },
        "flask" => Choice {
            id: "flask",
            label: "Flask",
            hint: "lightweight Python web applications",
        },
        "rails" => Choice {
            id: "rails",
            label: "Rails",
            hint: "full-stack Ruby web framework",
        },
        "sinatra" => Choice {
            id: "sinatra",
            label: "Sinatra",
            hint: "lightweight Ruby web applications",
        },
        "none" => Choice {
            id: "none",
            label: "None",
            hint: "language-only project",
        },
        _ => Choice {
            id: "none",
            label: "None",
            hint: "",
        },
    }
}

pub fn resolve_none_to_language(framework_choice: &str, language: &str) -> String {
    if framework_choice == "none" {
        language.to_string()
    } else {
        framework_choice.to_string()
    }
}

pub fn framework_id_from_choice(choice: &str) -> String {
    choice
        .split_whitespace()
        .next()
        .unwrap_or(choice)
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_pretty_choices_back_to_ids() {
        assert_eq!(
            language_id_from_choice(&pretty_language("python")),
            "python"
        );
        assert_eq!(
            framework_id_from_choice(&pretty_framework("django")),
            "django"
        );
        assert_eq!(framework_id_from_choice(&pretty_framework("none")), "none");
        assert_eq!(resolve_none_to_language("none", "python"), "python");
        assert_eq!(resolve_none_to_language("django", "python"), "django");
        assert_eq!(language_choice("python").id, "python");
        assert_eq!(framework_choice("none").id, "none");
    }
}
