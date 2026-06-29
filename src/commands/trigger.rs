// GTD Trigger List — walk through a trigger list one item at a time

use std::io::{self, Write, stdout};

use crossterm::{
    cursor::MoveTo,
    style,
    style::{Attribute, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal, queue,
};

use crate::data;
use crate::ui;

pub fn run() -> io::Result<()> {
    let filepath = data::trigger_list_file();

    let (triggers, sections) = match data::parse_trigger_list(&filepath) {
        Some(result) => result,
        None => {
            eprintln!("Error: No trigger items found in {}", filepath.display());
            eprintln!("Expected markdown with ## headings and - list items");
            std::process::exit(1);
        }
    };

    let mut out = stdout();
    terminal::enable_raw_mode()?;
    ui::hide_cursor(&mut out)?;

    let mut index: usize = 0;
    let mut finished = false;

    let result = run_loop(&mut out, &triggers, &sections, &mut index, &mut finished);

    terminal::disable_raw_mode()?;
    ui::reset_terminal(&mut out)?;
    out.flush()?;

    match result {
        Ok(()) => {
            eprintln!("\u{2713} Reviewed {}/{} triggers.", index + 1, triggers.len());
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}

fn run_loop(
    out: &mut impl Write,
    triggers: &[data::TriggerItem],
    sections: &[(String, Vec<String>)],
    index: &mut usize,
    finished: &mut bool,
) -> io::Result<()> {
    loop {
        if *finished {
            draw_summary(out, triggers, sections)?;
        } else {
            draw_screen(out, triggers, sections, *index)?;
        }

        let key = ui::read_key()?;
        match key {
            ui::Key::Char('q') | ui::Key::CtrlC => break,
            ui::Key::Char('b') if *finished => {
                *finished = false;
                *index = triggers.len() - 1;
            }
            _ if *finished => break,
            ui::Key::Char('l') | ui::Key::Char(' ') | ui::Key::Enter => {
                if *index < triggers.len() - 1 {
                    *index += 1;
                } else {
                    *finished = true;
                }
            }
            ui::Key::Char('j') | ui::Key::Up => {
                if *index > 0 {
                    *index -= 1;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn draw_screen(
    out: &mut impl Write,
    triggers: &[data::TriggerItem],
    _sections: &[(String, Vec<String>)],
    index: usize,
) -> io::Result<()> {
    let (cols, rows) = ui::terminal_size();
    let trigger = &triggers[index];

    ui::clear_screen(out)?;

    // ── Top bar ─────────────────────────────────────────────
    let title = " \u{1f99e} GTD Trigger List ";
    ui::draw_top_bar(out, title, "")?;

    // ── Section name ────────────────────────────────────────
    queue!(out, MoveTo(0, 2))?;
    queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::ACCENT_SOFT), SetAttribute(Attribute::Bold))?;
    write!(out, "{}", ui::center_text(&trigger.section, cols))?;
    queue!(out, style::ResetColor)?;

    // ── Divider ─────────────────────────────────────────────
    ui::draw_divider(out, 3)?;

    // ── Main trigger item ───────────────────────────────────
    let max_text_width = (cols.saturating_sub(8)).min(72) as usize;
    let wrapped = ui::word_wrap(&trigger.text, max_text_width);
    let total_lines = wrapped.len() as u16;

    let available_top = 5u16;
    let available_bottom = rows.saturating_sub(2);
    let available_height = available_bottom.saturating_sub(available_top);
    let start_row = available_top + available_height.saturating_sub(total_lines) / 2;

    for (i, line) in wrapped.iter().enumerate() {
        let r = start_row + i as u16;
        if r >= rows.saturating_sub(1) {
            break;
        }
        queue!(out, MoveTo(0, r))?;
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::CODE))?;
        write!(out, "{}", ui::center_text(line, cols))?;
        queue!(out, style::ResetColor)?;
    }

    // ── Bottom help bar ─────────────────────────────────────
    let help_parts = vec![
        (" SPACE ", ui::ACCENT, true),
        (" next ", ui::C_DIM, false),
        (" b ", ui::ACCENT, true),
        (" back ", ui::C_DIM, false),
        (" q ", ui::ERROR, true),
        (" quit ", ui::C_DIM, false),
    ];
    ui::draw_help_bar(out, &help_parts)?;

    out.flush()
}

fn draw_summary(
    out: &mut impl Write,
    triggers: &[data::TriggerItem],
    sections: &[(String, Vec<String>)],
) -> io::Result<()> {
    let (cols, rows) = ui::terminal_size();

    ui::clear_screen(out)?;

    // Title
    let title = "\u{2713} Trigger List Complete";
    queue!(out, MoveTo(0, 2))?;
    queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::LINK), SetAttribute(Attribute::Bold))?;
    write!(out, "{}", ui::center_text(title, cols))?;
    queue!(out, style::ResetColor)?;

    // Stats
    queue!(out, MoveTo(0, 4))?;
    queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::SYS_TEXT))?;
    write!(out, "{}", ui::center_text(&format!("{} items across {} sections", triggers.len(), sections.len()), cols))?;
    queue!(out, style::ResetColor)?;

    // Section breakdown
    queue!(out, MoveTo(0, 6))?;
    queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::ACCENT_SOFT), SetAttribute(Attribute::Bold))?;
    write!(out, "{}", ui::center_text("Sections", cols))?;
    queue!(out, style::ResetColor)?;

    for (i, (section, items)) in sections.iter().enumerate() {
        let row = 7 + i as u16;
        if row >= rows.saturating_sub(2) {
            break;
        }
        let line = format!("  {}: {} items", section, items.len());
        queue!(out, MoveTo(0, row))?;
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::SYS_TEXT))?;
        write!(out, "{}", ui::center_text(&line, cols))?;
        queue!(out, style::ResetColor)?;
    }

    // Help
    let help_parts = vec![
        (" b ", ui::ACCENT, true),
        (" go back ", ui::C_DIM, false),
        (" q ", ui::ERROR, true),
        (" quit ", ui::C_DIM, false),
    ];
    ui::draw_help_bar(out, &help_parts)?;

    out.flush()
}
