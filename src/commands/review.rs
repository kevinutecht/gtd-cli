// GTD Weekly Review — guided weekly review checklist with embedded viewers
// Step 0: Assessment (weekly board summary)
// Steps 1+: Standard GTD review (Get Clear, Get Current, Get Creative)

use std::collections::HashMap;
use std::io::{self, Write, stdout};

use crossterm::{
    cursor::MoveTo,
    style,
    style::{Attribute, Color, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal, queue,
};

use chrono::Datelike;

use crate::data;
use crate::ui;
use crate::commands::altitudes;
use unicode_width::UnicodeWidthStr;

// ── Viewer states ──────────────────────────────────────────────────────

struct ListState {
    scroll: usize,
    cursor: usize,
    expanded: Vec<bool>,
}

impl ListState {
    fn new(size: usize) -> Self {
        Self {
            scroll: 0,
            cursor: 0,
            expanded: vec![true; size.max(1)],
        }
    }
}

struct TriggerState {
    triggers: Vec<data::TriggerItem>,
    index: usize,
}

impl TriggerState {
    fn new() -> Self {
        let (triggers, _) = data::parse_trigger_list(&data::trigger_list_file())
            .unwrap_or_default();
        Self {
            triggers,
            index: 0,
        }
    }
}

struct CalendarState {
    scroll: usize,
    cursor: usize,
}

impl CalendarState {
    fn new() -> Self {
        let events = data::load_calendar();
        let today = chrono::Local::now();
        let today_num = today.year() * 10000 + today.month() as i32 * 100 + today.day() as i32;
        let cursor = events.iter().position(|e| {
            let event_num = e.year * 10000 + e.month as i32 * 100 + e.day as i32;
            event_num >= today_num
        }).unwrap_or(0);
        let scroll = cursor.saturating_sub(2);
        Self { scroll, cursor }
    }
}

struct ChecklistsState {
    files: Vec<String>,
    scroll: usize,
    cursor: usize,
}

impl ChecklistsState {
    fn new() -> Self {
        Self {
            files: data::load_checklist_files(),
            scroll: 0,
            cursor: 0,
        }
    }

    fn reload(&mut self) {
        self.files = data::load_checklist_files();
        if self.cursor >= self.files.len() && self.cursor > 0 {
            self.cursor = self.files.len() - 1;
        }
    }
}

struct AltitudesState {
    lines: Vec<(String, crossterm::style::Color, bool)>,
    scroll: usize,
}

impl AltitudesState {
    fn new() -> Self {
        Self {
            lines: altitudes::build_altitudes_lines(),
            scroll: 0,
        }
    }
}

struct BrainstormState {
    lines: Vec<String>,
    scroll: usize,
    cursor: usize,
}

impl BrainstormState {
    fn new() -> Self {
        let path = data::brainstorm_path();
        let lines = if path.exists() {
            std::fs::read_to_string(&path)
                .unwrap_or_default()
                .lines()
                .map(|l| l.to_string())
                .collect()
        } else {
            vec!["# Brainstorm".to_string(), String::new(), "No brainstorm file found.".to_string()]
        };
        Self {
            lines,
            scroll: 0,
            cursor: 0,
        }
    }

    fn reload(&mut self) {
        let path = data::brainstorm_path();
        self.lines = if path.exists() {
            std::fs::read_to_string(&path)
                .unwrap_or_default()
                .lines()
                .map(|l| l.to_string())
                .collect()
        } else {
            vec!["# Brainstorm".to_string(), String::new(), "No brainstorm file found.".to_string()]
        };
        self.cursor = self.cursor.min(self.lines.len().saturating_sub(1));
    }
}

// ── Summary state ──────────────────────────────────────────────────────

struct SummaryState {
    scroll: u16,
}

impl SummaryState {
    fn new() -> Self {
        Self { scroll: 0 }
    }
}

// ── Assessment state ───────────────────────────────────────────────────

struct AssessmentState {
    all_dates: Vec<String>,
    date_index: Option<usize>,
    scroll: u16,
}

impl AssessmentState {
    fn new() -> Self {
        let all_dates = data::list_weekly_board_dates();
        let date_index = if all_dates.is_empty() { None } else { Some(all_dates.len() - 1) };
        Self {
            all_dates,
            date_index,
            scroll: 0,
        }
    }

    fn reload_dates(&mut self) {
        self.all_dates = data::list_weekly_board_dates();
        // Keep same index if valid, otherwise go to last
        if let Some(idx) = self.date_index {
            if idx >= self.all_dates.len() {
                self.date_index = if self.all_dates.is_empty() { None } else { Some(self.all_dates.len() - 1) };
            }
        }
    }

    fn current_date(&self) -> Option<&str> {
        self.date_index.map(|i| self.all_dates[i].as_str())
    }

    fn board(&self) -> Option<data::WeeklyBoard> {
        self.current_date().map(|d| data::load_weekly_board(d))
    }

    fn prev_board(&self) -> Option<(String, data::WeeklyBoard)> {
        self.date_index.and_then(|idx| {
            if idx > 0 {
                let d = &self.all_dates[idx - 1];
                Some((d.clone(), data::load_weekly_board(d)))
            } else {
                None
            }
        })
    }
}

// ── Main entry ─────────────────────────────────────────────────────────

pub fn run() -> io::Result<()> {
    let steps = data::review_steps();
    let mut notes: HashMap<usize, String> = HashMap::new();
    let mut step_index: usize = 0;
    let mut finished = false;
    let mut showing_summary = true;

    // Summary state (startup screen)
    let mut summary_state: Option<SummaryState> = Some(SummaryState::new());

    // Assessment state (step 0)
    let mut assessment: Option<AssessmentState> = Some(AssessmentState::new());

    // Viewer states for steps 1+
    let mut list_state: Option<ListState> = None;
    let mut trigger_state: Option<TriggerState> = None;
    let mut calendar_state: Option<CalendarState> = None;
    let mut checklists_state: Option<ChecklistsState> = None;
    let mut altitudes_state: Option<AltitudesState> = None;
    let mut brainstorm_state: Option<BrainstormState> = None;

    let mut out = stdout();
    terminal::enable_raw_mode()?;
    ui::hide_cursor(&mut out)?;

    let result = run_loop(
        &mut out,
        &steps,
        &mut notes,
        &mut step_index,
        &mut finished,
        &mut showing_summary,
        &mut summary_state,
        &mut assessment,
        &mut list_state,
        &mut trigger_state,
        &mut calendar_state,
        &mut checklists_state,
        &mut altitudes_state,
        &mut brainstorm_state,
    );

    terminal::disable_raw_mode()?;
    ui::reset_terminal(&mut out)?;
    out.flush()?;

    result
}

// ── Main loop ──────────────────────────────────────────────────────────

fn run_loop(
    out: &mut impl Write,
    steps: &[data::ReviewStep],
    notes: &mut HashMap<usize, String>,
    step_index: &mut usize,
    finished: &mut bool,
    showing_summary: &mut bool,
    summary_state: &mut Option<SummaryState>,
    assessment: &mut Option<AssessmentState>,
    list_state: &mut Option<ListState>,
    trigger_state: &mut Option<TriggerState>,
    calendar_state: &mut Option<CalendarState>,
    checklists_state: &mut Option<ChecklistsState>,
    altitudes_state: &mut Option<AltitudesState>,
    brainstorm_state: &mut Option<BrainstormState>,
) -> io::Result<()> {
    loop {
        if *finished {
            draw_summary(out, steps, notes)?;
        } else if *showing_summary {
            // Summary dashboard screen
            draw_summary_screen(out, summary_state.as_ref().unwrap())?;
        } else if *step_index == 0 {
            // Assessment screen
            draw_assessment(out, assessment.as_mut().unwrap())?;
        } else {
            draw_step(
                out, steps, notes, *step_index, list_state,
                trigger_state, calendar_state, checklists_state, altitudes_state,
                brainstorm_state,
            )?;
        }

        let key = ui::read_key()?;
        match key {
            ui::Key::Char('q') | ui::Key::CtrlC => break,
            _ if *finished => break,

            // ── Quick capture (works anywhere) ──────────────────
            ui::Key::Char('c') => {
                let (cols, rows) = ui::terminal_size();
                queue!(out, MoveTo(0, rows - 1))?;
                queue!(out, SetBackgroundColor(ui::BG_LIGHT), SetForegroundColor(ui::ACCENT))?;
                write!(out, "{}", " ".repeat(cols as usize))?;
                queue!(out, MoveTo(0, rows - 1))?;
                write!(out, " Capture: ")?;
                out.flush()?;

                if let Some(text) = ui::_read_line(out, 10, rows - 1)? {
                    if !text.trim().is_empty() {
                        data::save_quick_capture(text.trim());
                    }
                }
            }

            // ── Summary screen keys ─────────────────────────────
            _ if *showing_summary => {
                let summ = summary_state.as_mut().unwrap();
                match key {
                    ui::Key::Char(' ') => {
                        // Advance to Assessment screen
                        *showing_summary = false;
                        *step_index = 0;
                    }
                    ui::Key::Down | ui::Key::Char('j') => {
                        let (_, rows) = ui::terminal_size();
                        let lines = build_summary_lines();
                        let max_scroll = (lines.len() as u16).saturating_sub(rows.saturating_sub(3));
                        if summ.scroll < max_scroll {
                            summ.scroll = (summ.scroll + 3).min(max_scroll);
                        }
                    }
                    ui::Key::Up | ui::Key::Char('k') => {
                        summ.scroll = summ.scroll.saturating_sub(3);
                    }
                    _ => {}
                }
            }

            // ── Assessment screen keys ──────────────────────────
            _ if *step_index == 0 => {
                let asm = assessment.as_mut().unwrap();
                match key {
                    ui::Key::Char('h') => {
                        if let Some(idx) = asm.date_index {
                            if idx > 0 {
                                asm.date_index = Some(idx - 1);
                                asm.scroll = 0;
                            }
                        }
                    }
                    ui::Key::Char('l') => {
                        if let Some(idx) = asm.date_index {
                            if idx + 1 < asm.all_dates.len() {
                                asm.date_index = Some(idx + 1);
                                asm.scroll = 0;
                            }
                        }
                    }
                    ui::Key::Down | ui::Key::Char('j') => {
                        let (_, rows) = ui::terminal_size();
                        let board = asm.board().unwrap_or_default();
                        let profile = data::load_profile();
                        let total_lines = build_assessment_lines(&profile, &board, rows).len() as u16;
                        let max_scroll = total_lines.saturating_sub(rows.saturating_sub(3));
                        if asm.scroll < max_scroll {
                            asm.scroll = (asm.scroll + 3).min(max_scroll);
                        }
                    }
                    ui::Key::Up | ui::Key::Char('k') => {
                        asm.scroll = asm.scroll.saturating_sub(3);
                    }
                    ui::Key::Char('e') => {
                        if let Some(date) = asm.current_date() {
                            terminal::disable_raw_mode()?;
                            ui::reset_terminal(out)?;
                            open_board_editor(date)?;
                            terminal::enable_raw_mode()?;
                            ui::hide_cursor(out)?;
                        }
                    }
                    ui::Key::Char('p') => {
                        // Launch accountability partner skill via mimo
                        terminal::disable_raw_mode()?;
                        ui::reset_terminal(out)?;
                        println!("\n  Running skill <accountability-partner>... please wait.\n");
                        out.flush()?;
                        let home = std::env::var("HOME").unwrap();
                        let gtd_data = format!("{}/data/gtd", home);
                        let status = std::process::Command::new("mimo")
                            .args(["run", "--dir", &gtd_data, "Load the accountability-partner skill and review my weekly performance"])
                            .current_dir(&gtd_data)
                            .stdin(std::process::Stdio::inherit())
                            .stdout(std::process::Stdio::inherit())
                            .stderr(std::process::Stdio::inherit())
                            .status();
                        if let Err(e) = status {
                            eprintln!("Failed to launch mimo: {}", e);
                            eprintln!("Press Enter to continue...");
                            let mut buf = String::new();
                            std::io::stdin().read_line(&mut buf).ok();
                        }
                        terminal::enable_raw_mode()?;
                        ui::hide_cursor(out)?;
                        asm.reload_dates();
                    }
                    ui::Key::Char(' ') => {
                        // Continue to step 1 (first GTD review step)
                        *step_index = 1;
                        init_viewer_state(
                            &steps[*step_index], list_state,
                            trigger_state, calendar_state, checklists_state, altitudes_state,
                            brainstorm_state,
                        );
                    }
                    ui::Key::Char('b') => {
                        // Go back to summary screen
                        *showing_summary = true;
                        *step_index = 0;
                    }
                    _ => {}
                }
            }

            // ── List viewer keys ─────────────────────────────────
            ui::Key::Char('j') if list_state.is_some() => {
                let list = list_state.as_mut().unwrap();
                let step = &steps[*step_index];
                let total = list_total_lines(step, &list.expanded);
                if list.cursor + 1 < total { list.cursor += 1; }
                auto_scroll_list(list);
            }
            ui::Key::Char('k') if list_state.is_some() => {
                let list = list_state.as_mut().unwrap();
                list.cursor = list.cursor.saturating_sub(1);
                auto_scroll_list(list);
            }
            ui::Key::Char('x') | ui::Key::Enter if list_state.is_some() => {
                let list = list_state.as_mut().unwrap();
                let step = &steps[*step_index];
                if let Some(idx) = list_item_at_line(step, &list.expanded, list.cursor) {
                    list.expanded[idx] = !list.expanded[idx];
                }
            }
            ui::Key::Char('g') if list_state.is_some() => {
                let list = list_state.as_mut().unwrap();
                list.cursor = 0;
                list.scroll = 0;
            }
            ui::Key::Char('G') if list_state.is_some() => {
                let list = list_state.as_mut().unwrap();
                let step = &steps[*step_index];
                let total = list_total_lines(step, &list.expanded);
                if total > 0 {
                    list.cursor = total - 1;
                    auto_scroll_list(list);
                }
            }

            // ── Calendar viewer keys ─────────────────────────────
            ui::Key::Char('j') | ui::Key::Down if calendar_state.is_some() => {
                let cal = calendar_state.as_mut().unwrap();
                let events = data::load_calendar();
                if cal.cursor + 1 < events.len() { cal.cursor += 1; }
                auto_scroll_calendar(cal);
            }
            ui::Key::Char('k') | ui::Key::Up if calendar_state.is_some() => {
                let cal = calendar_state.as_mut().unwrap();
                cal.cursor = cal.cursor.saturating_sub(1);
                auto_scroll_calendar(cal);
            }
            ui::Key::Char('g') if calendar_state.is_some() => {
                let cal = calendar_state.as_mut().unwrap();
                cal.cursor = 0;
                cal.scroll = 0;
            }
            ui::Key::Char('G') if calendar_state.is_some() => {
                let cal = calendar_state.as_mut().unwrap();
                let events = data::load_calendar();
                if !events.is_empty() {
                    cal.cursor = events.len() - 1;
                    auto_scroll_calendar(cal);
                }
            }
            ui::Key::Char('u') if calendar_state.is_some() => {
                let cal = calendar_state.as_mut().unwrap();
                cal.cursor = cal.cursor.saturating_sub(10);
                auto_scroll_calendar(cal);
            }

            // ── Checklists viewer keys ──────────────────────────
            ui::Key::Char('j') if checklists_state.is_some() => {
                let checklists = checklists_state.as_mut().unwrap();
                if checklists.cursor + 1 < checklists.files.len() { checklists.cursor += 1; }
                auto_scroll_checklists(checklists);
            }
            ui::Key::Char('k') if checklists_state.is_some() => {
                let checklists = checklists_state.as_mut().unwrap();
                checklists.cursor = checklists.cursor.saturating_sub(1);
                auto_scroll_checklists(checklists);
            }
            ui::Key::Char('g') if checklists_state.is_some() => {
                let checklists = checklists_state.as_mut().unwrap();
                checklists.cursor = 0;
                checklists.scroll = 0;
            }
            ui::Key::Char('G') if checklists_state.is_some() => {
                let checklists = checklists_state.as_mut().unwrap();
                if !checklists.files.is_empty() {
                    checklists.cursor = checklists.files.len() - 1;
                    auto_scroll_checklists(checklists);
                }
            }
            ui::Key::Char('u') if checklists_state.is_some() => {
                let checklists = checklists_state.as_mut().unwrap();
                checklists.cursor = checklists.cursor.saturating_sub(10);
                auto_scroll_checklists(checklists);
            }

            // ── Altitudes viewer keys ───────────────────────────
            ui::Key::Char('j') | ui::Key::Down if altitudes_state.is_some() => {
                let alt = altitudes_state.as_mut().unwrap();
                let (_, rows) = ui::terminal_size();
                let viewer_height = rows.saturating_sub(16) as usize;
                let max_scroll = alt.lines.len().saturating_sub(viewer_height);
                if alt.scroll < max_scroll {
                    alt.scroll = (alt.scroll + 3).min(max_scroll);
                }
            }
            ui::Key::Char('k') | ui::Key::Up if altitudes_state.is_some() => {
                let alt = altitudes_state.as_mut().unwrap();
                alt.scroll = alt.scroll.saturating_sub(3);
            }
            ui::Key::Char('g') if altitudes_state.is_some() => {
                let alt = altitudes_state.as_mut().unwrap();
                alt.scroll = 0;
            }
            ui::Key::Char('G') if altitudes_state.is_some() => {
                let alt = altitudes_state.as_mut().unwrap();
                let (_, rows) = ui::terminal_size();
                let viewer_height = rows.saturating_sub(16) as usize;
                alt.scroll = alt.lines.len().saturating_sub(viewer_height);
            }
            ui::Key::Char('u') if altitudes_state.is_some() => {
                let alt = altitudes_state.as_mut().unwrap();
                alt.scroll = alt.scroll.saturating_sub(10);
            }

            // ── Trigger viewer keys ─────────────────────────────
            ui::Key::Char('l') if trigger_state.is_some() => {
                let trigger = trigger_state.as_mut().unwrap();
                if trigger.index < trigger.triggers.len() - 1 { trigger.index += 1; }
            }
            ui::Key::Char('h') if trigger_state.is_some() => {
                let trigger = trigger_state.as_mut().unwrap();
                if trigger.index > 0 { trigger.index -= 1; }
            }
            ui::Key::Char('g') if trigger_state.is_some() => {
                let trigger = trigger_state.as_mut().unwrap();
                trigger.index = 0;
            }
            ui::Key::Char('G') if trigger_state.is_some() => {
                let trigger = trigger_state.as_mut().unwrap();
                if !trigger.triggers.is_empty() { trigger.index = trigger.triggers.len() - 1; }
            }

            // ── Brainstorm viewer keys ──────────────────────────
            ui::Key::Char('j') if brainstorm_state.is_some() => {
                let bs = brainstorm_state.as_mut().unwrap();
                if bs.cursor + 1 < bs.lines.len() { bs.cursor += 1; }
                auto_scroll_brainstorm(bs);
            }
            ui::Key::Char('k') if brainstorm_state.is_some() => {
                let bs = brainstorm_state.as_mut().unwrap();
                bs.cursor = bs.cursor.saturating_sub(1);
                auto_scroll_brainstorm(bs);
            }
            ui::Key::Char('G') if brainstorm_state.is_some() => {
                let bs = brainstorm_state.as_mut().unwrap();
                if !bs.lines.is_empty() { bs.cursor = bs.lines.len() - 1; }
                auto_scroll_brainstorm(bs);
            }
            ui::Key::Char('g') if brainstorm_state.is_some() => {
                // Launch brainstorm skill via mimo
                terminal::disable_raw_mode()?;
                ui::reset_terminal(out)?;
                println!("\n  Running skill <gtd-brainstorm>... please wait.\n");
                out.flush()?;
                let home = std::env::var("HOME").unwrap();
                let gtd_data = format!("{}/data/gtd", home);
                let status = std::process::Command::new("mimo")
                    .args(["run", "--dir", &gtd_data, "Load the gtd-brainstorm skill and brainstorm 30 new ideas for my GTD system"])
                    .current_dir(&gtd_data)
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .status();
                if let Err(e) = status {
                    eprintln!("Failed to launch mimo: {}", e);
                    eprintln!("Press Enter to continue...");
                    let mut buf = String::new();
                    std::io::stdin().read_line(&mut buf).ok();
                }
                terminal::enable_raw_mode()?;
                ui::hide_cursor(out)?;
                if let Some(bs) = brainstorm_state.as_mut() { bs.reload(); }
            }

            // ── u for paging back (list viewer) ────────────────
            ui::Key::Char('u') if list_state.is_some() => {
                let list = list_state.as_mut().unwrap();
                list.cursor = list.cursor.saturating_sub(10);
                auto_scroll_list(list);
            }
            // ── Editor (all viewers) ─────────────────────────────
            ui::Key::Char('e') if checklists_state.is_some() => {
                let checklists = checklists_state.as_ref().unwrap();
                if let Some(name) = checklists.files.get(checklists.cursor) {
                    let path = data::checklist_path(name);
                    terminal::disable_raw_mode()?;
                    ui::reset_terminal(out)?;
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
                    std::process::Command::new(&editor).arg(&path).status()?;
                    terminal::enable_raw_mode()?;
                    ui::hide_cursor(out)?;
                    if let Some(state) = checklists_state.as_mut() { state.reload(); }
                }
            }
            ui::Key::Char('e') if list_state.is_some() || steps[*step_index].viewer.is_some() => {
                let step = &steps[*step_index];
                if let Some(ref viewer) = step.viewer {
                    terminal::disable_raw_mode()?;
                    ui::reset_terminal(out)?;
                    open_viewer_editor(viewer)?;
                    terminal::enable_raw_mode()?;
                    ui::hide_cursor(out)?;
                    init_viewer_state(step, list_state, trigger_state, calendar_state, checklists_state, altitudes_state, brainstorm_state);
                }
            }

            // ── Step navigation (Space/b when viewer active) ──
            ui::Key::Char(' ') if list_state.is_some() || trigger_state.is_some() || calendar_state.is_some() || checklists_state.is_some() || altitudes_state.is_some() || brainstorm_state.is_some() => {
                if *step_index < steps.len() - 1 {
                    *step_index += 1;
                    init_viewer_state(&steps[*step_index], list_state, trigger_state, calendar_state, checklists_state, altitudes_state, brainstorm_state);
                }
            }
            ui::Key::Char('b') if list_state.is_some() || trigger_state.is_some() || calendar_state.is_some() || checklists_state.is_some() || altitudes_state.is_some() || brainstorm_state.is_some() => {
                if *step_index > 1 {
                    *step_index -= 1;
                    init_viewer_state(&steps[*step_index], list_state, trigger_state, calendar_state, checklists_state, altitudes_state, brainstorm_state);
                } else if *step_index == 1 {
                    // Go back to assessment
                    *step_index = 0;
                }
            }

            // ── Step completion (when viewer active, use 'D' for next) ──
            ui::Key::Char('D') if list_state.is_some() || trigger_state.is_some() || calendar_state.is_some() || checklists_state.is_some() || altitudes_state.is_some() || brainstorm_state.is_some() => {
                if *step_index < steps.len() - 1 {
                    *step_index += 1;
                    init_viewer_state(&steps[*step_index], list_state, trigger_state, calendar_state, checklists_state, altitudes_state, brainstorm_state);
                } else {
                    *finished = true;
                }
            }

            // ── Notes (works with or without viewer) ─────────────
            ui::Key::Char('n') if list_state.is_some() => {
                let step = &steps[*step_index];
                if let Some(data::ReviewViewer::Projects) = &step.viewer {
                    let list = list_state.as_mut().unwrap();
                    let mut projects = data::load_projects();
                    if let Some((proj_idx, item_idx)) = list_item_at_line_with_item(&list.expanded, list.cursor, &projects) {
                        for item in &mut projects[proj_idx].items {
                            if let Some(rest) = item.strip_prefix("[>] ") {
                                *item = rest.to_string();
                            }
                        }
                        let item = &mut projects[proj_idx].items[item_idx];
                        if !item.starts_with("[>] ") {
                            *item = format!("[>] {}", item);
                        }
                        data::save_projects(&projects);
                    }
                }
            }

            // ── No viewer: original step controls ────────────────
            ui::Key::Char(' ') | ui::Key::Enter if list_state.is_none() && trigger_state.is_none() && calendar_state.is_none() && checklists_state.is_none() && brainstorm_state.is_none() => {
                if *step_index < steps.len() - 1 {
                    *step_index += 1;
                    init_viewer_state(&steps[*step_index], list_state, trigger_state, calendar_state, checklists_state, altitudes_state, brainstorm_state);
                } else {
                    *finished = true;
                }
            }
            ui::Key::Char(' ') | ui::Key::Down if list_state.is_none() && trigger_state.is_none() && calendar_state.is_none() && checklists_state.is_none() && altitudes_state.is_none() && brainstorm_state.is_none() => {
                if *step_index < steps.len() - 1 {
                    *step_index += 1;
                    init_viewer_state(&steps[*step_index], list_state, trigger_state, calendar_state, checklists_state, altitudes_state, brainstorm_state);
                }
            }
            ui::Key::Char('b') | ui::Key::Up if list_state.is_none() && trigger_state.is_none() && calendar_state.is_none() && checklists_state.is_none() && altitudes_state.is_none() && brainstorm_state.is_none() => {
                if *step_index > 1 {
                    *step_index -= 1;
                    init_viewer_state(&steps[*step_index], list_state, trigger_state, calendar_state, checklists_state, altitudes_state, brainstorm_state);
                } else if *step_index == 1 {
                    // Go back to Assessment screen
                    *step_index = 0;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

// ── Summary dashboard screen ──────────────────────────────────────────

fn divider_line(label: &str) -> String {
    let dash = "\u{2500}";
    let prefix = format!("{}{} ", dash, dash);
    let suffix_len = 50usize.saturating_sub(prefix.len() + label.len() + 2);
    format!("{}{} {}", prefix, label, dash.repeat(suffix_len))
}

fn build_summary_lines() -> Vec<(String, Color, bool)> {
    let mut lines: Vec<(String, Color, bool)> = Vec::new();

    let profile = data::load_profile();
    let goals = data::load_goals();
    let areas = data::load_areas();
    let vision = data::load_vision();
    let projects = data::load_projects();
    let next_actions = data::load_next_actions();
    let waiting = data::load_waiting_for();
    let agendas = data::load_agendas();
    let someday = data::load_someday_maybe();
    let events = data::load_calendar();
    let dates = data::list_weekly_board_dates();

    // Purpose
    if !profile.purpose.is_empty() {
        lines.push(("Purpose".to_string(), ui::ACCENT, true));
        lines.push((format!("  {}", profile.purpose), ui::FG, false));
        lines.push((String::new(), ui::FG, false));
    }

    // ── Horizons ───────────────────────────────────────────────────────
    lines.push((divider_line("Horizons"), ui::ACCENT_SOFT, false));

    // H5 Purpose
    if profile.purpose.is_empty() {
        lines.push(("  H5 Purpose          \u{2717} not set".to_string(), ui::C_DIM, false));
    } else {
        lines.push(("  H5 Purpose          \u{2713} set".to_string(), ui::LINK, false));
    }

    // H4 Vision
    let vision_count = vision.areas.len();
    if vision_count == 0 {
        lines.push(("  H4 Vision           \u{2717} not set".to_string(), ui::C_DIM, false));
    } else {
        lines.push((format!("  H4 Vision           {} area{}", vision_count, if vision_count == 1 { "" } else { "s" }), ui::LINK, false));
    }

    // H3 Goals
    let active_goals: Vec<_> = goals.iter().filter(|g| g.status == data::GoalStatus::Active).collect();
    if goals.is_empty() {
        lines.push(("  H3 Goals            no goals".to_string(), ui::C_DIM, false));
    } else {
        lines.push((format!("  H3 Goals            {} active", active_goals.len()), ui::LINK, false));
    }

    // H2 Areas
    if areas.is_empty() {
        lines.push(("  H2 Areas            no areas".to_string(), ui::C_DIM, false));
    } else {
        lines.push((format!("  H2 Areas            {} area{}", areas.len(), if areas.len() == 1 { "" } else { "s" }), ui::LINK, false));
    }

    // H1 Projects
    let active_projects: Vec<_> = projects.iter().filter(|p| !p.items.is_empty()).collect();
    if projects.is_empty() {
        lines.push(("  H1 Projects         no projects".to_string(), ui::C_DIM, false));
    } else {
        lines.push((format!("  H1 Projects         {} active", active_projects.len()), ui::LINK, false));
    }

    lines.push((String::new(), ui::FG, false));

    // ── Actions & Commitments ──────────────────────────────────────────
    lines.push((divider_line("Actions & Commitments"), ui::ACCENT_SOFT, false));

    lines.push((format!("  Next Actions        {} item{}", next_actions.len(), if next_actions.len() == 1 { "" } else { "s" }), ui::FG, false));
    lines.push((format!("  Waiting For         {} item{}", waiting.len(), if waiting.len() == 1 { "" } else { "s" }), ui::FG, false));
    lines.push((format!("  Agendas             {} people", agendas.len()), ui::FG, false));
    lines.push((format!("  Someday/Maybe       {} item{}", someday.len(), if someday.len() == 1 { "" } else { "s" }), ui::FG, false));

    lines.push((String::new(), ui::FG, false));

    // ── Calendar ───────────────────────────────────────────────────────
    lines.push((divider_line("Calendar"), ui::ACCENT_SOFT, false));

    let today = chrono::Local::now();
    let today_num = today.year() * 10000 + today.month() as i32 * 100 + today.day() as i32;
    let this_week: Vec<_> = events.iter().filter(|e| {
        let event_num = e.year * 10000 + e.month as i32 * 100 + e.day as i32;
        event_num >= today_num && event_num <= today_num + 7
    }).collect();
    lines.push((format!("  This week           {} event{}", this_week.len(), if this_week.len() == 1 { "" } else { "s" }), ui::FG, false));

    lines.push((String::new(), ui::FG, false));

    // ── Recent Performance ─────────────────────────────────────────────
    lines.push((divider_line("Recent Performance"), ui::ACCENT_SOFT, false));

    if let Some(latest_date) = dates.last() {
        let board = data::load_weekly_board(latest_date);
        if let Some(score) = board.score {
            let note = board.score_note.as_deref().unwrap_or("");
            let truncated = if note.len() > 50 { format!("{}...", &note[..47]) } else { note.to_string() };
            lines.push((format!("  Latest Score        {}/10 \u{2014} {}", score, truncated), ui::ACCENT, false));
        } else {
            lines.push(("  Latest Score        no score".to_string(), ui::C_DIM, false));
        }

        // Review streak
        let (total, streak) = compute_review_streak(&dates);
        if total > 0 {
            lines.push((format!("  Reviews Completed   {} total ({} week streak)", total, streak), ui::FG, false));
        }
    } else {
        lines.push(("  Latest Score        no boards yet".to_string(), ui::C_DIM, false));
    }

    lines
}

fn compute_review_streak(dates: &[String]) -> (usize, usize) {
    if dates.is_empty() {
        return (0, 0);
    }

    let total = dates.len();
    let mut streak = 1usize;

    // Walk backward from the most recent, counting consecutive weeks
    for i in (0..dates.len() - 1).rev() {
        if let (Some(curr), Some(prev)) = (dates.get(i + 1), dates.get(i)) {
            // Parse dates and check if they're ~7 days apart
            if let (Ok(c), Ok(p)) = (
                chrono::NaiveDate::parse_from_str(curr, "%Y-%m-%d"),
                chrono::NaiveDate::parse_from_str(prev, "%Y-%m-%d"),
            ) {
                let diff = (c - p).num_days();
                if diff >= 5 && diff <= 9 { // Allow 5-9 days for weekly cadence
                    streak += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    (total, streak)
}

fn draw_summary_screen(out: &mut impl Write, summ: &SummaryState) -> io::Result<()> {
    let (cols, rows) = ui::terminal_size();
    ui::clear_screen(out)?;

    ui::draw_top_bar(out, " \u{1f4cb} GTD Summary ", " weekly review ")?;

    let lines = build_summary_lines();

    // Render visible lines
    let visible_start = summ.scroll as usize;
    let visible_end = (visible_start + rows.saturating_sub(3) as usize).min(lines.len());
    let visible: Vec<&(String, Color, bool)> = lines
        .iter()
        .skip(visible_start)
        .take(visible_end.saturating_sub(visible_start))
        .collect();

    for (i, (text, color, bold)) in visible.iter().enumerate() {
        let row = 2 + i as u16;
        if row >= rows.saturating_sub(1) { break; }
        queue!(out, MoveTo(0, row))?;
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(*color))?;
        if *bold {
            queue!(out, SetAttribute(Attribute::Bold))?;
        }
        let visible_text = truncate_display(text, cols);
        write!(out, "{}", visible_text)?;
        let shown_width = UnicodeWidthStr::width(visible_text.as_str()) as u16;
        let pad = cols.saturating_sub(shown_width);
        if pad > 0 {
            write!(out, "{}", " ".repeat(pad as usize))?;
        }
        queue!(out, style::ResetColor)?;
    }

    // Fill remaining rows
    let content_end_row = 2 + visible.len() as u16;
    let help_row = rows.saturating_sub(1);
    for row in content_end_row..help_row {
        queue!(out, MoveTo(0, row))?;
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::FG))?;
        write!(out, "{}", " ".repeat(cols as usize))?;
    }
    queue!(out, style::ResetColor)?;

    // Help bar
    let help_parts = vec![
        (" SPACE ", ui::ACCENT, true),
        (" start review ", ui::C_DIM, false),
        (" \u{2191}\u{2193}/jk ", ui::ACCENT, true),
        (" scroll ", ui::C_DIM, false),
        (" c ", ui::ACCENT, true),
        (" capture ", ui::C_DIM, false),
        (" q ", ui::ERROR, true),
        (" quit ", ui::C_DIM, false),
    ];
    ui::draw_help_bar(out, &help_parts)?;

    out.flush()
}

// ── Assessment screen ─────────────────────────────────────────────────

fn build_assessment_lines(
    profile: &data::Profile,
    board: &data::WeeklyBoard,
    _cols: u16,
) -> Vec<(String, Color, bool)> {
    let mut lines: Vec<(String, Color, bool)> = Vec::new();

    // Profile section
    if !profile.name.is_empty() || !profile.purpose.is_empty() {
        if !profile.name.is_empty() {
            lines.push((profile.name.clone(), ui::FG_BRIGHT, true));
            lines.push((String::new(), ui::FG, false));
        }
        if !profile.purpose.is_empty() {
            lines.push((format!("Purpose: {}", profile.purpose), ui::CODE, false));
        }
    }

    // Accountability partner notes
    if let Some(ref notes) = board.partner_notes {
        lines.push((String::new(), ui::FG, false));
        let score_str = board.score
            .map(|s| format!("Score: {}/10", s))
            .unwrap_or_else(|| "Partner Notes".to_string());
        lines.push((format!("\u{1f465} {}", score_str), ui::ACCENT, true));
        if let Some(ref note) = board.score_note {
            lines.push((note.clone(), ui::FG, false));
        }
        for line in notes.lines() {
            push_styled_line(&mut lines, line);
        }
    }

    // Last week
    if !board.accomplishments.is_empty() || !board.struggles.is_empty() {
        lines.push((String::new(), ui::FG, false));
        lines.push(("\u{1f3c3} Last Week".to_string(), ui::ACCENT_SOFT, true));
    }

    if !board.accomplishments.is_empty() {
        lines.push(("Accomplishments".to_string(), ui::LINK, true));
        for item in &board.accomplishments {
            lines.push((format!("  \u{2022} {}", item), ui::FG, false));
        }
    }

    if !board.struggles.is_empty() {
        lines.push(("Struggles".to_string(), ui::ERROR, true));
        for item in &board.struggles {
            lines.push((format!("  \u{2022} {}", item), ui::FG, false));
        }
    }

    lines
}

fn draw_assessment(out: &mut impl Write, asm: &AssessmentState) -> io::Result<()> {
    let (cols, rows) = ui::terminal_size();
    ui::clear_screen(out)?;

    let profile = data::load_profile();

    // Get current board
    let (date_str, board, prev) = if let Some(date) = asm.current_date() {
        let b = data::load_weekly_board(date);
        let p = asm.prev_board();
        (date.to_string(), b, p)
    } else {
        // No boards exist
        ui::draw_top_bar(out, " \u{1f99e} Weekly Assessment ", " no boards found ")?;
        let help_parts = vec![
            (" p ", ui::ACCENT, true),
            (" partner ", ui::C_DIM, false),
            (" q ", ui::ERROR, true),
            (" quit ", ui::C_DIM, false),
        ];
        ui::draw_help_bar(out, &help_parts)?;
        out.flush()?;
        return Ok(());
    };

    // Top bar with week navigation
    let nav = if asm.all_dates.len() > 1 {
        format!(" [{}/{}] ", asm.date_index.map(|i| i + 1).unwrap_or(0), asm.all_dates.len())
    } else {
        String::new()
    };
    let title = format!(" \u{1f99e} Weekly Assessment \u{2014} {}{} ", date_str, nav);
    let right = prev.as_ref().map(|(d, _)| format!(" prev: {} ", d)).unwrap_or_default();
    ui::draw_top_bar(out, &title, &right)?;

    let lines = build_assessment_lines(&profile, &board, cols);

    // Render visible lines
    let visible_start = asm.scroll as usize;
    let visible_end = (visible_start + rows.saturating_sub(3) as usize).min(lines.len());
    let visible: Vec<&(String, Color, bool)> = lines
        .iter()
        .skip(visible_start)
        .take(visible_end.saturating_sub(visible_start))
        .collect();

    for (i, (text, color, bold)) in visible.iter().enumerate() {
        let row = 2 + i as u16;
        if row >= rows.saturating_sub(1) { break; }
        queue!(out, MoveTo(0, row))?;
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(*color))?;
        if *bold {
            queue!(out, SetAttribute(Attribute::Bold))?;
        }
        let visible_text = truncate_display(text, cols);
        write!(out, "{}", visible_text)?;
        let shown_width = UnicodeWidthStr::width(visible_text.as_str()) as u16;
        let pad = cols.saturating_sub(shown_width);
        if pad > 0 {
            write!(out, "{}", " ".repeat(pad as usize))?;
        }
        queue!(out, style::ResetColor)?;
    }

    // Fill remaining rows
    let content_end_row = 2 + visible.len() as u16;
    let help_row = rows.saturating_sub(1);
    for row in content_end_row..help_row {
        queue!(out, MoveTo(0, row))?;
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::FG))?;
        write!(out, "{}", " ".repeat(cols as usize))?;
    }
    queue!(out, style::ResetColor)?;

    // Help bar
    let help_parts = vec![
        (" h/l ", ui::ACCENT, true),
        (" weeks ", ui::C_DIM, false),
        (" \u{2191}\u{2193}/jk ", ui::ACCENT, true),
        (" scroll ", ui::C_DIM, false),
        (" e ", ui::ACCENT, true),
        (" edit ", ui::C_DIM, false),
        (" c ", ui::ACCENT, true),
        (" capture ", ui::C_DIM, false),
        (" p ", ui::ACCENT, true),
        (" partner ", ui::C_DIM, false),
        (" SPACE ", ui::ACCENT, true),
        (" review ", ui::C_DIM, false),
        (" b ", ui::ACCENT, true),
        (" summary ", ui::C_DIM, false),
        (" q ", ui::ERROR, true),
        (" quit ", ui::C_DIM, false),
    ];
    ui::draw_help_bar(out, &help_parts)?;

    out.flush()
}

fn open_board_editor(date_str: &str) -> io::Result<()> {
    let path = data::weekly_board_path(date_str);
    if !path.exists() {
        data::save_weekly_board(&data::WeeklyBoard::default(), date_str);
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
    std::process::Command::new(&editor)
        .arg(&path)
        .status()?;
    Ok(())
}

// ── Step drawing (steps 1+) ───────────────────────────────────────────

fn init_viewer_state(
    step: &data::ReviewStep,
    list_state: &mut Option<ListState>,
    trigger_state: &mut Option<TriggerState>,
    calendar_state: &mut Option<CalendarState>,
    checklists_state: &mut Option<ChecklistsState>,
    altitudes_state: &mut Option<AltitudesState>,
    brainstorm_state: &mut Option<BrainstormState>,
) {
    match &step.viewer {
        Some(data::ReviewViewer::Calendar) => {
            *list_state = None;
            *trigger_state = None;
            *calendar_state = Some(CalendarState::new());
            *checklists_state = None;
            *altitudes_state = None;
            *brainstorm_state = None;
        }
        Some(data::ReviewViewer::Projects) => {
            *list_state = Some(ListState::new(data::load_projects().len()));
            *trigger_state = None;
            *calendar_state = None;
            *checklists_state = None;
            *altitudes_state = None;
            *brainstorm_state = None;
        }
        Some(data::ReviewViewer::Waiting) => {
            *list_state = Some(ListState::new(data::load_waiting_for().len()));
            *trigger_state = None;
            *calendar_state = None;
            *checklists_state = None;
            *altitudes_state = None;
            *brainstorm_state = None;
        }
        Some(data::ReviewViewer::Someday) => {
            *list_state = Some(ListState::new(data::load_someday_maybe().len()));
            *trigger_state = None;
            *calendar_state = None;
            *checklists_state = None;
            *altitudes_state = None;
            *brainstorm_state = None;
        }
        Some(data::ReviewViewer::Trigger) => {
            *list_state = None;
            *trigger_state = Some(TriggerState::new());
            *calendar_state = None;
            *checklists_state = None;
            *altitudes_state = None;
            *brainstorm_state = None;
        }
        Some(data::ReviewViewer::Checklists) => {
            *list_state = None;
            *trigger_state = None;
            *calendar_state = None;
            *checklists_state = Some(ChecklistsState::new());
            *altitudes_state = None;
            *brainstorm_state = None;
        }
        Some(data::ReviewViewer::Altitudes) => {
            *list_state = None;
            *trigger_state = None;
            *calendar_state = None;
            *checklists_state = None;
            *altitudes_state = Some(AltitudesState::new());
            *brainstorm_state = None;
        }
        Some(data::ReviewViewer::Brainstorm) => {
            *list_state = None;
            *trigger_state = None;
            *calendar_state = None;
            *checklists_state = None;
            *altitudes_state = None;
            *brainstorm_state = Some(BrainstormState::new());
        }
        None => {
            *list_state = None;
            *trigger_state = None;
            *calendar_state = None;
            *checklists_state = None;
            *altitudes_state = None;
            *brainstorm_state = None;
        }
    }
}

fn draw_step(
    out: &mut impl Write,
    steps: &[data::ReviewStep],
    notes: &mut HashMap<usize, String>,
    step_index: usize,
    list_state: &Option<ListState>,
    trigger_state: &Option<TriggerState>,
    calendar_state: &Option<CalendarState>,
    checklists_state: &Option<ChecklistsState>,
    altitudes_state: &Option<AltitudesState>,
    brainstorm_state: &Option<BrainstormState>,
) -> io::Result<()> {
    let (cols, rows) = ui::terminal_size();
    let total = steps.len();
    let step = &steps[step_index];
    let progress = (step_index + 1) as f64 / total as f64;

    ui::clear_screen(out)?;

    let bar_title = " \u{1f99e} Weekly Review ";
    let counter = format!(" {} / {} ", step_index + 1, total);
    ui::draw_top_bar(out, bar_title, &counter)?;
    ui::draw_progress_bar(out, 1, progress)?;

    let phase_color = match step.phase_color {
        "link" => ui::LINK,
        "accent" => ui::ACCENT,
        "purple" => ui::PURPLE,
        _ => ui::ACCENT,
    };
    queue!(out, MoveTo(0, 3))?;
    queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(phase_color), SetAttribute(Attribute::Bold))?;
    write!(out, "{}", ui::center_text(&format!("\u{25c6} {}", step.phase), cols))?;
    queue!(out, style::ResetColor)?;

    queue!(out, MoveTo(0, 5))?;
    queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::FG_BRIGHT), SetAttribute(Attribute::Bold))?;
    write!(out, "{}", ui::center_text(step.title, cols))?;
    queue!(out, style::ResetColor)?;

    ui::draw_divider(out, 6)?;

    let desc_lines: Vec<&str> = step.desc.split('\n').collect();
    let desc_start = 8u16;
    let max_line_len = desc_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    let block_left = cols.saturating_sub(max_line_len) / 2;
    for (i, line) in desc_lines.iter().enumerate() {
        let r = desc_start + i as u16;
        if r >= 12 { break; }
        queue!(out, MoveTo(block_left, r))?;
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::ACCENT))?;
        write!(out, "{}", line)?;
        queue!(out, style::ResetColor)?;
    }

    if let Some(note) = notes.get(&step_index) {
        let note_row = desc_start + desc_lines.len() as u16;
        if note_row < 13 {
            queue!(out, MoveTo(block_left, note_row))?;
            queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::ACCENT), SetAttribute(Attribute::Italic))?;
            write!(out, "{}", &format!("\u{1f4dd} {}", note))?;
            queue!(out, style::ResetColor)?;
        }
    }

    if step.viewer.is_some() {
        let viewer_start = 13u16;
        let viewer_end = rows.saturating_sub(2);
        let viewer_height = (viewer_end - viewer_start) as usize;

        if viewer_height > 3 {
            queue!(out, MoveTo(0, viewer_start))?;
            queue!(out, SetBackgroundColor(ui::BG_LIGHT), SetForegroundColor(ui::BORDER))?;
            write!(out, "{}", "\u{2500}".repeat(cols as usize))?;
            queue!(out, style::ResetColor)?;

            match &step.viewer {
                Some(data::ReviewViewer::Calendar) => {
                    let events = data::load_calendar();
                    if let Some(cal) = calendar_state {
                        draw_calendar_viewer(out, &events, viewer_start + 1, viewer_height as u16 - 1, cal.scroll, cal.cursor);
                    }
                }
                Some(data::ReviewViewer::Projects) => {
                    let projects = data::load_projects();
                    if let Some(list) = list_state {
                        draw_projects_viewer(out, &projects, &list.expanded, viewer_start + 1, viewer_height as u16 - 1, list.scroll, list.cursor, ui::ACCENT);
                    }
                }
                Some(data::ReviewViewer::Waiting) => {
                    let items = data::load_waiting_for();
                    if let Some(list) = list_state {
                        draw_projects_viewer(out, &items, &list.expanded, viewer_start + 1, viewer_height as u16 - 1, list.scroll, list.cursor, ui::QUOTE);
                    }
                }
                Some(data::ReviewViewer::Someday) => {
                    let items = data::load_someday_maybe();
                    if let Some(list) = list_state {
                        draw_projects_viewer(out, &items, &list.expanded, viewer_start + 1, viewer_height as u16 - 1, list.scroll, list.cursor, ui::LINK);
                    }
                }
                Some(data::ReviewViewer::Trigger) => {
                    if let Some(trigger) = trigger_state {
                        draw_trigger_onetime(out, &trigger.triggers, trigger.index, viewer_start + 1, viewer_height as u16 - 1);
                    }
                }
                Some(data::ReviewViewer::Checklists) => {
                    if let Some(checklists) = checklists_state {
                        draw_checklists_viewer(out, &checklists.files, viewer_start + 1, viewer_height as u16 - 1, checklists.scroll, checklists.cursor);
                    }
                }
                Some(data::ReviewViewer::Altitudes) => {
                    if let Some(altitudes) = altitudes_state {
                        draw_altitudes_viewer(out, &altitudes.lines, viewer_start + 1, viewer_height as u16 - 1, altitudes.scroll);
                    }
                }
                Some(data::ReviewViewer::Brainstorm) => {
                    if let Some(brainstorm) = brainstorm_state {
                        draw_brainstorm_viewer(out, &brainstorm.lines, viewer_start + 1, viewer_height as u16 - 1, brainstorm.scroll, brainstorm.cursor);
                    }
                }
                None => {}
            }
        }
    }

    let help_parts = if list_state.is_some() {
        let step = &steps[step_index];
        let show_next = matches!(&step.viewer, Some(data::ReviewViewer::Projects));
        let mut parts = vec![
            (" x ", ui::ACCENT, true), (" toggle ", ui::C_DIM, false),
            (" j/k ", ui::ACCENT, true), (" scroll ", ui::C_DIM, false),
        ];
        if show_next {
            parts.push((" n ", ui::ACCENT, true));
            parts.push((" next ", ui::C_DIM, false));
        }
        parts.push((" e ", ui::ACCENT, true));
        parts.push((" edit ", ui::C_DIM, false));
        parts.push((" c ", ui::ACCENT, true));
        parts.push((" capture ", ui::C_DIM, false));
        parts.push((" SPACE/b ", ui::ACCENT, true));
        parts.push((" step ", ui::C_DIM, false));
        parts.push((" q ", ui::ERROR, true));
        parts.push((" quit ", ui::C_DIM, false));
        parts
    } else if trigger_state.is_some() {
        vec![
            (" l ", ui::ACCENT, true), (" next ", ui::C_DIM, false),
            (" h ", ui::ACCENT, true), (" back ", ui::C_DIM, false),
            (" e ", ui::ACCENT, true), (" edit ", ui::C_DIM, false),
            (" c ", ui::ACCENT, true), (" capture ", ui::C_DIM, false),
            (" SPACE/b ", ui::ACCENT, true), (" step ", ui::C_DIM, false),
            (" q ", ui::ERROR, true), (" quit ", ui::C_DIM, false),
        ]
    } else if calendar_state.is_some() {
        vec![
            (" j/k ", ui::ACCENT, true), (" scroll ", ui::C_DIM, false),
            (" e ", ui::ACCENT, true), (" edit ", ui::C_DIM, false),
            (" c ", ui::ACCENT, true), (" capture ", ui::C_DIM, false),
            (" SPACE/b ", ui::ACCENT, true), (" step ", ui::C_DIM, false),
            (" q ", ui::ERROR, true), (" quit ", ui::C_DIM, false),
        ]
    } else if checklists_state.is_some() {
        vec![
            (" j/k ", ui::ACCENT, true), (" scroll ", ui::C_DIM, false),
            (" e ", ui::ACCENT, true), (" edit ", ui::C_DIM, false),
            (" c ", ui::ACCENT, true), (" capture ", ui::C_DIM, false),
            (" SPACE/b ", ui::ACCENT, true), (" step ", ui::C_DIM, false),
            (" q ", ui::ERROR, true), (" quit ", ui::C_DIM, false),
        ]
    } else if altitudes_state.is_some() {
        vec![
            (" j/k ", ui::ACCENT, true), (" scroll ", ui::C_DIM, false),
            (" c ", ui::ACCENT, true), (" capture ", ui::C_DIM, false),
            (" SPACE/b ", ui::ACCENT, true), (" step ", ui::C_DIM, false),
            (" q ", ui::ERROR, true), (" quit ", ui::C_DIM, false),
        ]
    } else if brainstorm_state.is_some() {
        vec![
            (" j/k ", ui::ACCENT, true), (" scroll ", ui::C_DIM, false),
            (" e ", ui::ACCENT, true), (" edit ", ui::C_DIM, false),
            (" c ", ui::ACCENT, true), (" capture ", ui::C_DIM, false),
            (" g ", ui::ACCENT, true), (" generate ", ui::C_DIM, false),
            (" b ", ui::ACCENT, true), (" back ", ui::C_DIM, false),
            (" SPACE ", ui::ACCENT, true), (" next ", ui::C_DIM, false),
            (" q ", ui::ERROR, true), (" quit ", ui::C_DIM, false),
        ]
    } else {
        vec![
            (" SPACE/b ", ui::ACCENT, true), (" nav ", ui::C_DIM, false),
            (" q ", ui::ERROR, true), (" quit ", ui::C_DIM, false),
        ]
    };
    ui::draw_help_bar(out, &help_parts)?;
    out.flush()
}

fn draw_summary(out: &mut impl Write, steps: &[data::ReviewStep], notes: &HashMap<usize, String>) -> io::Result<()> {
    let (cols, rows) = ui::terminal_size();
    ui::clear_screen(out)?;
    ui::draw_top_bar(out, " \u{1f99e} Weekly Review ", " complete ")?;

    queue!(out, MoveTo(0, 3))?;
    queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::ACCENT), SetAttribute(Attribute::Bold))?;
    write!(out, "{}", ui::center_text("\u{2714} Review Complete", cols))?;
    queue!(out, style::ResetColor)?;

    let mut row = 5u16;
    for (i, step) in steps.iter().enumerate() {
        if let Some(note) = notes.get(&i) {
            if row >= rows.saturating_sub(1) { break; }
            queue!(out, MoveTo(0, row))?;
            queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::SYS_TEXT))?;
            write!(out, "  \u{2022} {}: {}", step.title, note)?;
            queue!(out, style::ResetColor)?;
            row += 1;
        }
    }

    let help_parts = vec![
        (" q ", ui::ERROR, true), (" quit ", ui::C_DIM, false),
    ];
    ui::draw_help_bar(out, &help_parts)?;
    out.flush()
}

// ── Viewer drawing functions ──────────────────────────────────────────

fn draw_calendar_viewer(
    out: &mut impl Write,
    events: &[data::CalendarEvent],
    start_row: u16,
    height: u16,
    scroll: usize,
    cursor: usize,
) {
    let (cols, _) = ui::terminal_size();
    if events.is_empty() {
        queue!(out, MoveTo(0, start_row + 1)).ok();
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::C_DIM)).ok();
        write!(out, "{}", ui::center_text("No calendar events", cols)).ok();
        queue!(out, style::ResetColor).ok();
        return;
    }
    let today = chrono::Local::now();
    let today_num = today.year() * 10000 + today.month() as i32 * 100 + today.day() as i32;
    let mut shown = 0;
    for (i, event) in events.iter().enumerate() {
        if i < scroll { continue; }
        if shown >= height as usize { break; }
        let event_num = event.year * 10000 + event.month as i32 * 100 + event.day as i32;
        let is_past = event_num < today_num;
        let is_today = event_num == today_num;
        let is_cursor = i == cursor;
        let row = start_row + shown as u16;
        if is_cursor {
            queue!(out, MoveTo(0, row)).ok();
            queue!(out, SetBackgroundColor(ui::BG_HIGHLIGHT)).ok();
        } else {
            queue!(out, MoveTo(0, row)).ok();
            queue!(out, SetBackgroundColor(ui::BG)).ok();
        }
        if is_cursor {
            queue!(out, SetForegroundColor(ui::ACCENT), SetAttribute(Attribute::Bold)).ok();
            write!(out, " \u{25b6} ").ok();
        } else {
            write!(out, "   ").ok();
        }
        let date_str = format!("{:02}/{:02}", event.month, event.day);
        if is_today {
            queue!(out, SetForegroundColor(ui::ACCENT), SetAttribute(Attribute::Bold)).ok();
            write!(out, "{} ", date_str).ok();
            queue!(out, SetForegroundColor(ui::FG_BRIGHT)).ok();
        } else if is_past {
            queue!(out, SetForegroundColor(ui::C_DIM)).ok();
            write!(out, "{} ", date_str).ok();
        } else {
            queue!(out, SetForegroundColor(ui::LINK)).ok();
            write!(out, "{} ", date_str).ok();
            queue!(out, SetForegroundColor(ui::FG)).ok();
        }
        let max_len = (cols as usize).saturating_sub(15);
        if event.description.len() <= max_len {
            write!(out, "{}", event.description).ok();
        } else {
            write!(out, "{}...", &event.description[..max_len.saturating_sub(3)]).ok();
        }
        queue!(out, style::ResetColor).ok();
        shown += 1;
    }
}

fn draw_projects_viewer(
    out: &mut impl Write,
    projects: &[data::Project],
    expanded: &[bool],
    start_row: u16,
    height: u16,
    scroll: usize,
    cursor: usize,
    accent_color: crossterm::style::Color,
) {
    let (cols, _) = ui::terminal_size();
    if projects.is_empty() {
        queue!(out, MoveTo(0, start_row + 1)).ok();
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::C_DIM)).ok();
        write!(out, "{}", ui::center_text("No items", cols)).ok();
        queue!(out, style::ResetColor).ok();
        return;
    }
    let mut line_num = 0usize;
    let mut drawn = 0u16;
    for (i, proj) in projects.iter().enumerate() {
        if line_num >= scroll && drawn < height {
            let row = start_row + drawn;
            if row < start_row + height {
                let is_cursor = line_num == cursor;
                queue!(out, MoveTo(0, row)).ok();
                if is_cursor { queue!(out, SetBackgroundColor(ui::BG_HIGHLIGHT)).ok(); }
                else { queue!(out, SetBackgroundColor(ui::BG)).ok(); }
                if proj.items.is_empty() {
                    queue!(out, SetForegroundColor(ui::C_DIM)).ok();
                    write!(out, "   ").ok();
                } else if expanded.get(i).copied().unwrap_or(true) {
                    queue!(out, SetForegroundColor(accent_color)).ok();
                    write!(out, " \u{25bc} ").ok();
                } else {
                    queue!(out, SetForegroundColor(accent_color)).ok();
                    write!(out, " \u{25b6} ").ok();
                }
                if is_cursor {
                    queue!(out, SetForegroundColor(ui::FG_BRIGHT), SetAttribute(Attribute::Bold)).ok();
                } else {
                    queue!(out, SetForegroundColor(accent_color), SetAttribute(Attribute::Bold)).ok();
                }
                write!(out, "{}", proj.name).ok();
                if !proj.items.is_empty() {
                    queue!(out, SetForegroundColor(ui::C_DIM)).ok();
                    write!(out, " ({})", proj.items.len()).ok();
                }
                queue!(out, style::ResetColor).ok();
                drawn += 1;
            }
        }
        line_num += 1;
        if expanded.get(i).copied().unwrap_or(true) {
            for item in &proj.items {
                if line_num >= scroll && drawn < height {
                    let row = start_row + drawn;
                    if row < start_row + height {
                        let is_cursor = line_num == cursor;
                        let is_next = item.starts_with("[>] ");
                        let display_text = if is_next { &item[4..] } else { item };
                        queue!(out, MoveTo(0, row)).ok();
                        if is_cursor { queue!(out, SetBackgroundColor(ui::BG_HIGHLIGHT)).ok(); }
                        else { queue!(out, SetBackgroundColor(ui::BG)).ok(); }
                        queue!(out, SetForegroundColor(ui::C_DIM)).ok();
                        if is_next {
                            write!(out, "     ").ok();
                            queue!(out, SetForegroundColor(ui::ACCENT)).ok();
                            write!(out, "\u{25b6} ").ok();
                        } else {
                            write!(out, "     \u{2022} ").ok();
                        }
                        if is_cursor {
                            queue!(out, SetForegroundColor(ui::FG_BRIGHT)).ok();
                        } else if is_next {
                            queue!(out, SetForegroundColor(ui::ACCENT)).ok();
                        } else {
                            queue!(out, SetForegroundColor(ui::FG)).ok();
                        }
                        let max_len = (cols as usize).saturating_sub(10);
                        if display_text.len() <= max_len {
                            write!(out, "{}", display_text).ok();
                        } else {
                            write!(out, "{}...", &display_text[..max_len.saturating_sub(3)]).ok();
                        }
                        queue!(out, style::ResetColor).ok();
                        drawn += 1;
                    }
                }
                line_num += 1;
            }
        }
    }
}

fn draw_trigger_onetime(
    out: &mut impl Write,
    triggers: &[data::TriggerItem],
    index: usize,
    start_row: u16,
    height: u16,
) {
    let (cols, _) = ui::terminal_size();
    if triggers.is_empty() {
        queue!(out, MoveTo(0, start_row + 1)).ok();
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::C_DIM)).ok();
        write!(out, "{}", ui::center_text("No trigger list found", cols)).ok();
        queue!(out, style::ResetColor).ok();
        return;
    }
    let idx = index.min(triggers.len() - 1);
    let trigger = &triggers[idx];
    let counter = format!(" {} / {} ", idx + 1, triggers.len());

    queue!(out, MoveTo(0, start_row)).ok();
    queue!(out, SetBackgroundColor(ui::BG_LIGHT), SetForegroundColor(ui::BORDER)).ok();
    write!(out, "{}", "\u{2500}".repeat(cols as usize)).ok();
    queue!(out, style::ResetColor).ok();

    queue!(out, MoveTo(0, start_row + 1)).ok();
    queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::ACCENT_SOFT), SetAttribute(Attribute::Bold)).ok();
    write!(out, "{}", ui::center_text(&trigger.section, cols)).ok();
    queue!(out, style::ResetColor).ok();

    let text_start = start_row + 3;
    let text_lines: Vec<&str> = trigger.text.split('\n').collect();
    for (i, line) in text_lines.iter().enumerate() {
        let row = text_start + i as u16;
        if row >= start_row + height - 1 { break; }
        queue!(out, MoveTo(0, row)).ok();
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::FG)).ok();
        write!(out, "{}", ui::center_text(line, cols)).ok();
        queue!(out, style::ResetColor).ok();
    }

    queue!(out, MoveTo(0, start_row + height - 1)).ok();
    queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::C_DIM)).ok();
    write!(out, "{}", ui::center_text(&counter, cols)).ok();
    queue!(out, style::ResetColor).ok();
}

fn draw_checklists_viewer(
    out: &mut impl Write,
    files: &[String],
    start_row: u16,
    height: u16,
    scroll: usize,
    cursor: usize,
) {
    let (cols, _) = ui::terminal_size();
    if files.is_empty() {
        queue!(out, MoveTo(0, start_row + 1)).ok();
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::C_DIM)).ok();
        write!(out, "{}", ui::center_text("No checklists found", cols)).ok();
        queue!(out, style::ResetColor).ok();
        return;
    }
    for i in 0..height as usize {
        let idx = i + scroll;
        if idx >= files.len() { break; }
        let row = start_row + i as u16;
        let is_cursor = idx == cursor;
        queue!(out, MoveTo(0, row)).ok();
        if is_cursor { queue!(out, SetBackgroundColor(ui::BG_HIGHLIGHT)).ok(); }
        else { queue!(out, SetBackgroundColor(ui::BG)).ok(); }
        if is_cursor {
            queue!(out, SetForegroundColor(ui::ACCENT), SetAttribute(Attribute::Bold)).ok();
            write!(out, " \u{25b6} ").ok();
        } else {
            write!(out, "   ").ok();
        }
        queue!(out, SetForegroundColor(ui::FG)).ok();
        write!(out, "{}", files[idx]).ok();
        queue!(out, style::ResetColor).ok();
    }
}

fn draw_altitudes_viewer(
    out: &mut impl Write,
    lines: &[(String, crossterm::style::Color, bool)],
    start_row: u16,
    height: u16,
    scroll: usize,
) {
    let (cols, _) = ui::terminal_size();
    for i in 0..height as usize {
        let idx = i + scroll;
        if idx >= lines.len() { break; }
        let (text, color, bold) = &lines[idx];
        let row = start_row + i as u16;
        queue!(out, MoveTo(0, row)).ok();
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(*color)).ok();
        if *bold { queue!(out, SetAttribute(Attribute::Bold)).ok(); }
        let max_len = cols as usize;
        if text.len() <= max_len {
            write!(out, "{}", text).ok();
        } else {
            write!(out, "{}", &text[..max_len]).ok();
        }
        queue!(out, style::ResetColor).ok();
    }
}

fn draw_brainstorm_viewer(
    out: &mut impl Write,
    lines: &[String],
    start_row: u16,
    height: u16,
    scroll: usize,
    cursor: usize,
) {
    let (cols, _) = ui::terminal_size();
    if lines.is_empty() {
        queue!(out, MoveTo(0, start_row + 1)).ok();
        queue!(out, SetBackgroundColor(ui::BG), SetForegroundColor(ui::C_DIM)).ok();
        write!(out, "{}", ui::center_text("No brainstorm content", cols)).ok();
        queue!(out, style::ResetColor).ok();
        return;
    }
    for i in 0..height as usize {
        let idx = i + scroll;
        if idx >= lines.len() { break; }
        let text = &lines[idx];
        let row = start_row + i as u16;
        let is_cursor = idx == cursor;
        queue!(out, MoveTo(0, row)).ok();
        if is_cursor {
            queue!(out, SetBackgroundColor(ui::BG_HIGHLIGHT)).ok();
        } else {
            queue!(out, SetBackgroundColor(ui::BG)).ok();
        }
        let is_heading = text.starts_with("# ");
        let is_subheading = text.starts_with("## ");
        let is_item = text.starts_with("- ") || text.starts_with("  - ");
        if is_heading {
            queue!(out, SetForegroundColor(ui::ACCENT), SetAttribute(Attribute::Bold)).ok();
        } else if is_subheading {
            queue!(out, SetForegroundColor(ui::ACCENT_SOFT), SetAttribute(Attribute::Bold)).ok();
        } else if is_item {
            queue!(out, SetForegroundColor(ui::FG)).ok();
        } else {
            queue!(out, SetForegroundColor(ui::FG)).ok();
        }
        let max_len = (cols as usize).saturating_sub(2);
        if text.len() <= max_len {
            write!(out, " {}", text).ok();
        } else {
            write!(out, " {}...", &text[..max_len.saturating_sub(3)]).ok();
        }
        queue!(out, style::ResetColor).ok();
    }
}

// ── List viewer helpers ────────────────────────────────────────────────

fn list_total_lines(step: &data::ReviewStep, expanded: &[bool]) -> usize {
    match &step.viewer {
        Some(data::ReviewViewer::Projects) => {
            let projects = data::load_projects();
            let mut total = 0;
            for (i, proj) in projects.iter().enumerate() {
                total += 1;
                if expanded.get(i).copied().unwrap_or(true) {
                    total += proj.items.len();
                }
            }
            total
        }
        Some(data::ReviewViewer::Waiting) => {
            let items = data::load_waiting_for();
            let mut total = 0;
            for (i, proj) in items.iter().enumerate() {
                total += 1;
                if expanded.get(i).copied().unwrap_or(true) {
                    total += proj.items.len();
                }
            }
            total
        }
        Some(data::ReviewViewer::Someday) => {
            let items = data::load_someday_maybe();
            let mut total = 0;
            for (i, proj) in items.iter().enumerate() {
                total += 1;
                if expanded.get(i).copied().unwrap_or(true) {
                    total += proj.items.len();
                }
            }
            total
        }
        _ => 0,
    }
}

fn list_item_at_line(step: &data::ReviewStep, expanded: &[bool], cursor: usize) -> Option<usize> {
    match &step.viewer {
        Some(data::ReviewViewer::Projects) | Some(data::ReviewViewer::Waiting) | Some(data::ReviewViewer::Someday) => {
            let mut line = 0;
            let projects = match &step.viewer {
                Some(data::ReviewViewer::Projects) => data::load_projects(),
                Some(data::ReviewViewer::Waiting) => data::load_waiting_for(),
                Some(data::ReviewViewer::Someday) => data::load_someday_maybe(),
                _ => return None,
            };
            for (i, proj) in projects.iter().enumerate() {
                if line == cursor { return Some(i); }
                line += 1;
                if expanded.get(i).copied().unwrap_or(true) {
                    for _ in &proj.items {
                        if line == cursor { return Some(i); }
                        line += 1;
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn list_item_at_line_with_item(expanded: &[bool], cursor: usize, projects: &[data::Project]) -> Option<(usize, usize)> {
    let mut line = 0;
    for (i, proj) in projects.iter().enumerate() {
        if line == cursor { return Some((i, 0)); }
        line += 1;
        if expanded.get(i).copied().unwrap_or(true) {
            for (j, _) in proj.items.iter().enumerate() {
                if line == cursor { return Some((i, j)); }
                line += 1;
            }
        }
    }
    None
}

fn auto_scroll_list(list: &mut ListState) {
    let (_, rows) = ui::terminal_size();
    let visible = (rows / 2) as usize;
    if list.cursor < list.scroll {
        list.scroll = list.cursor;
    } else if list.cursor >= list.scroll + visible {
        list.scroll = list.cursor - visible + 1;
    }
}

fn auto_scroll_calendar(cal: &mut CalendarState) {
    let (_, rows) = ui::terminal_size();
    let visible = (rows / 2) as usize;
    if cal.cursor < cal.scroll {
        cal.scroll = cal.cursor;
    } else if cal.cursor >= cal.scroll + visible {
        cal.scroll = cal.cursor - visible + 1;
    }
}

fn auto_scroll_checklists(checklists: &mut ChecklistsState) {
    let (_, rows) = ui::terminal_size();
    let visible = (rows / 2) as usize;
    if checklists.cursor < checklists.scroll {
        checklists.scroll = checklists.cursor;
    } else if checklists.cursor >= checklists.scroll + visible {
        checklists.scroll = checklists.cursor - visible + 1;
    }
}

fn auto_scroll_brainstorm(bs: &mut BrainstormState) {
    let (_, rows) = ui::terminal_size();
    let visible = (rows / 2) as usize;
    if bs.cursor < bs.scroll {
        bs.scroll = bs.cursor;
    } else if bs.cursor >= bs.scroll + visible {
        bs.scroll = bs.cursor - visible + 1;
    }
}

fn open_viewer_editor(viewer: &data::ReviewViewer) -> io::Result<()> {
    let path = match viewer {
        data::ReviewViewer::Calendar => data::calendar_path(),
        data::ReviewViewer::Projects => data::projects_path(),
        data::ReviewViewer::Waiting => data::waiting_for_path(),
        data::ReviewViewer::Someday => data::someday_maybe_path(),
        data::ReviewViewer::Trigger => data::trigger_list_file(),
        data::ReviewViewer::Checklists => data::checklists_dir().join("checklists.md"),
        data::ReviewViewer::Altitudes => data::data_dir().join("altitudes.md"),
        data::ReviewViewer::Brainstorm => data::brainstorm_path(),
    };
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
    std::process::Command::new(&editor).arg(&path).status()?;
    Ok(())
}

// ── Text helpers ───────────────────────────────────────────────────────

fn truncate_display(text: &str, show: u16) -> String {
    let show = show as usize;
    let mut result = String::new();
    let mut col = 0;
    for ch in text.chars() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        if col + w > show { break; }
        result.push(ch);
        col += w;
    }
    result
}

fn push_styled_line(out: &mut Vec<(String, Color, bool)>, line: &str) {
    let trimmed = line.trim_end();
    if trimmed.is_empty() {
        out.push((String::new(), ui::FG, false));
        return;
    }
    if trimmed.starts_with("### ") {
        let text = trimmed.trim_start_matches("### ");
        out.push((format!("  {}", text), ui::ACCENT_SOFT, true));
        return;
    }
    if trimmed.starts_with("## ") {
        let text = trimmed.trim_start_matches("## ");
        out.push((format!("  {}", text), ui::ACCENT, true));
        return;
    }
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        let text = trimmed[2..].trim_start();
        push_inline_styled(out, "  \u{2022} ", text);
        return;
    }
    push_inline_styled(out, "  ", trimmed);
}

fn push_inline_styled(out: &mut Vec<(String, Color, bool)>, prefix: &str, text: &str) {
    let cleaned_text = text.replace("**", "");
    let has_bold = text.contains("**");
    let display = format!("{}{}", prefix, cleaned_text);
    let color = if has_bold { ui::FG_BRIGHT } else { ui::FG };
    let bold = has_bold;
    out.push((display, color, bold));
}
