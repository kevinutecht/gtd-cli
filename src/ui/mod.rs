// Ember color palette and terminal drawing primitives

use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Attribute, Color, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType, size},
};
use std::io::{self, Write};

// ── Ember Color Palette ───────────────────────────────────────────────

pub const BG: Color = Color::Rgb { r: 30, g: 33, b: 39 };       // #1E2127
pub const BG_LIGHT: Color = Color::Rgb { r: 43, g: 47, b: 54 }; // #2B2F36
pub const BG_HIGHLIGHT: Color = Color::Rgb { r: 50, g: 55, b: 65 }; // #323741 cursor highlight
pub const BORDER: Color = Color::Rgb { r: 60, g: 65, b: 75 };   // #3C414B
pub const C_DIM: Color = Color::Rgb { r: 123, g: 127, b: 135 }; // #7B7F87
pub const SYS_TEXT: Color = Color::Rgb { r: 155, g: 163, b: 178 }; // #9BA3B2
pub const FG: Color = Color::Rgb { r: 232, g: 227, b: 213 };    // #E8E3D5
pub const FG_BRIGHT: Color = Color::Rgb { r: 243, g: 238, b: 224 }; // #F3EEE0
pub const ACCENT: Color = Color::Rgb { r: 246, g: 196, b: 83 }; // #F6C453
pub const ACCENT_SOFT: Color = Color::Rgb { r: 242, g: 166, b: 90 }; // #F2A65A
pub const CODE: Color = Color::Rgb { r: 240, g: 201, b: 135 };  // #F0C987
pub const QUOTE: Color = Color::Rgb { r: 140, g: 200, b: 255 }; // #8CC8FF
pub const LINK: Color = Color::Rgb { r: 125, g: 211, b: 165 };  // #7DD3A5
pub const ERROR: Color = Color::Rgb { r: 249, g: 112, b: 102 }; // #F97066
pub const PURPLE: Color = Color::Rgb { r: 199, g: 146, b: 234 }; // #C792EA

// ── Terminal Helpers ───────────────────────────────────────────────────

pub fn terminal_size() -> (u16, u16) {
    size().unwrap_or((80, 24))
}

pub fn clear_screen(out: &mut impl Write) -> io::Result<()> {
    queue!(out, Clear(ClearType::All), MoveTo(0, 0))?;
    queue!(out, SetBackgroundColor(BG), Clear(ClearType::All), MoveTo(0, 0))?;
    out.flush()
}

pub fn hide_cursor(out: &mut impl Write) -> io::Result<()> {
    execute!(out, cursor::Hide)
}

#[allow(dead_code)]
pub fn show_cursor(out: &mut impl Write) -> io::Result<()> {
    execute!(out, cursor::Show)
}

pub fn reset_terminal(out: &mut impl Write) -> io::Result<()> {
    execute!(
        out,
        cursor::Show,
        style::ResetColor,
        Clear(ClearType::All),
        MoveTo(0, 0)
    )
}

// ── Text Helpers ───────────────────────────────────────────────────────

pub fn center_text(text: &str, width: u16) -> String {
    let text_len = text.chars().count() as u16;
    if text_len >= width {
        return text.to_string();
    }
    let pad = (width - text_len) / 2;
    format!("{}{}", " ".repeat(pad as usize), text)
}

pub fn word_wrap(text: &str, width: usize) -> Vec<String> {
    if text.len() <= width {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.len() + 1 + word.len() > width {
            lines.push(current);
            current = word.to_string();
        } else if current.is_empty() {
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

// ── Drawing Primitives ─────────────────────────────────────────────────

pub fn draw_top_bar(out: &mut impl Write, title: &str, right: &str) -> io::Result<()> {
    let (cols, _) = terminal_size();
    let cols = cols as usize;
    queue!(out, MoveTo(0, 0))?;
    queue!(out, SetBackgroundColor(BG_LIGHT), SetForegroundColor(ACCENT))?;
    write!(out, "{}", title)?;
    let pad = cols.saturating_sub(title.len() + right.len());
    write!(out, "{}", " ".repeat(pad))?;
    queue!(out, SetForegroundColor(ACCENT_SOFT))?;
    write!(out, "{}", right)?;
    queue!(out, style::ResetColor)?;
    Ok(())
}

pub fn draw_progress_bar(out: &mut impl Write, row: u16, progress: f64) -> io::Result<()> {
    let (cols, _) = terminal_size();
    let bar_width = cols.saturating_sub(4) as usize;
    let filled = (bar_width as f64 * progress) as usize;
    let empty = bar_width.saturating_sub(filled);

    queue!(out, MoveTo(0, row))?;
    queue!(out, SetBackgroundColor(BG), SetForegroundColor(BORDER))?;
    write!(out, "  [")?;
    queue!(out, SetForegroundColor(LINK))?;
    write!(out, "{}", "━".repeat(filled))?;
    queue!(out, SetForegroundColor(BORDER))?;
    write!(out, "{}", "─".repeat(empty))?;
    write!(out, "]")?;
    queue!(out, style::ResetColor)?;
    Ok(())
}

pub fn draw_help_bar(out: &mut impl Write, parts: &[(&str, Color, bool)]) -> io::Result<()> {
    let (cols, rows) = terminal_size();
    let total_len: usize = parts.iter().map(|(t, _, _)| t.len()).sum();
    let left_pad = ((cols as usize).saturating_sub(total_len)) / 2;

    queue!(out, MoveTo(0, rows - 1))?;
    queue!(out, SetBackgroundColor(BG_LIGHT))?;
    write!(out, "{}", " ".repeat(left_pad))?;
    for (text, color, bold) in parts {
        queue!(out, SetForegroundColor(*color))?;
        if *bold {
            queue!(out, SetAttribute(Attribute::Bold))?;
        }
        write!(out, "{}", text)?;
        queue!(out, SetAttribute(Attribute::Reset))?;
        queue!(out, SetBackgroundColor(BG_LIGHT))?;
    }
    let remaining = (cols as usize).saturating_sub(left_pad + total_len);
    write!(out, "{}", " ".repeat(remaining))?;
    queue!(out, style::ResetColor)?;
    Ok(())
}

pub fn draw_divider(out: &mut impl Write, row: u16) -> io::Result<()> {
    let (cols, _) = terminal_size();
    let width = 40.min(cols.saturating_sub(10) as usize);
    let text = "─".repeat(width);
    let centered = center_text(&text, cols);
    queue!(out, MoveTo(0, row))?;
    queue!(out, SetBackgroundColor(BG), SetForegroundColor(BORDER))?;
    write!(out, "{}", centered)?;
    queue!(out, style::ResetColor)?;
    Ok(())
}

// ── Key Input ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Backspace,
    CtrlC,
    CtrlD,
    CtrlU,
}

pub fn read_key() -> io::Result<Key> {
    loop {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            if modifiers.contains(KeyModifiers::CONTROL) {
                match code {
                    KeyCode::Char('c') => return Ok(Key::CtrlC),
                    KeyCode::Char('d') => return Ok(Key::CtrlD),
                    KeyCode::Char('u') => return Ok(Key::CtrlU),
                    _ => continue,
                }
            }
            return Ok(match code {
                KeyCode::Char(c) => Key::Char(c),
                KeyCode::Enter => Key::Enter,
                KeyCode::Esc => Key::Escape,
                KeyCode::Left => Key::Left,
                KeyCode::Right => Key::Right,
                KeyCode::Up => Key::Up,
                KeyCode::Down => Key::Down,
                KeyCode::Backspace => Key::Backspace,
                _ => continue,
            });
        }
    }
}

/// Read a full line of text input. Returns None on Escape/Ctrl-C.
pub fn _read_line(out: &mut impl Write, prompt_col: u16, prompt_row: u16) -> io::Result<Option<String>> {
    queue!(out, cursor::Show)?;
    queue!(out, MoveTo(prompt_col, prompt_row))?;
    out.flush()?;

    let mut chars: Vec<char> = Vec::new();
    loop {
        match read_key()? {
            Key::Enter => {
                queue!(out, cursor::Hide)?;
                return Ok(Some(chars.into_iter().collect()));
            }
            Key::Escape | Key::CtrlC => {
                queue!(out, cursor::Hide)?;
                return Ok(None);
            }
            Key::CtrlD => {
                if chars.is_empty() {
                    queue!(out, cursor::Hide)?;
                    return Ok(None);
                }
            }
            Key::Backspace => {
                if chars.pop().is_some() {
                    queue!(out, MoveTo(prompt_col, prompt_row))?;
                    write!(out, "{}", " ".repeat(chars.len() + 1))?;
                    queue!(out, MoveTo(prompt_col, prompt_row))?;
                    let s: String = chars.iter().collect();
                    write!(out, "{}", s)?;
                    out.flush()?;
                }
            }
            Key::Char(c) => {
                chars.push(c);
                write!(out, "{}", c)?;
                out.flush()?;
            }
            _ => {}
        }
    }
}
