// Data types, file I/O, and markdown parsing

use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ── Paths ──────────────────────────────────────────────────────────────

pub fn data_dir() -> PathBuf {
    dirs::home_dir().unwrap().join("data").join("gtd")
}

pub fn trigger_list_file() -> PathBuf {
    // Search order: triggerlist.md in cwd, then ~/data/gtd/triggerlist.md
    let cwd_path = PathBuf::from("triggerlist.md");
    if cwd_path.exists() {
        return cwd_path;
    }
    let gtd_path = data_dir().join("triggerlist.md");
    if gtd_path.exists() {
        return gtd_path;
    }
    cwd_path // Return default, caller handles missing file
}

pub fn ensure_dirs() {
    // No directories to ensure for now
}

// ── Trigger List Parsing (gtd-trigger) ─────────────────────────────────

#[derive(Debug, Clone)]
pub struct TriggerItem {
    pub section: String,
    pub text: String,
}

pub fn parse_trigger_list(filepath: &PathBuf) -> Option<(Vec<TriggerItem>, Vec<(String, Vec<String>)>)> {
    let content = fs::read_to_string(filepath).ok()?;
    let mut sections: Vec<(String, Vec<String>)> = Vec::new();
    let mut current_section: Option<String> = None;
    let mut current_items: Vec<String> = Vec::new();

    for line in content.lines() {
        let stripped = line.trim();

        // Any heading level starts a new section
        if stripped.starts_with('#') {
            let heading = stripped.trim_start_matches('#').trim();
            if !heading.is_empty() {
                // Save previous section
                if current_section.is_some() || !current_items.is_empty() {
                    sections.push((
                        current_section.clone().unwrap_or_else(|| "Untitled".into()),
                        current_items,
                    ));
                    current_items = Vec::new();
                }
                current_section = Some(heading.to_string());
                continue;
            }
        }

        // List items: - item, * item, 1. item, 1) item
        let item_text = if let Some(rest) = stripped.strip_prefix("- ") {
            Some(rest)
        } else if let Some(rest) = stripped.strip_prefix("* ") {
            Some(rest)
        } else if let Some(rest) = stripped.strip_prefix("• ") {
            Some(rest)
        } else {
            // Numbered items: 1. or 1)
            if let Some(dot_pos) = stripped.find('.') {
                let prefix = &stripped[..dot_pos];
                if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
                    Some(stripped[dot_pos + 1..].trim())
                } else {
                    None
                }
            } else if let Some(paren_pos) = stripped.find(')') {
                let prefix = &stripped[..paren_pos];
                if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
                    Some(stripped[paren_pos + 1..].trim())
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(text) = item_text {
            current_items.push(text.trim().to_string());
        }
    }

    // Save last section
    if current_section.is_some() || !current_items.is_empty() {
        sections.push((
            current_section.unwrap_or_else(|| "Untitled".into()),
            current_items,
        ));
    }

    if sections.is_empty() {
        return None;
    }

    // Flatten into triggers
    let triggers: Vec<TriggerItem> = sections
        .iter()
        .flat_map(|(section, items)| {
            items.iter().map(move |item| TriggerItem {
                section: section.clone(),
                text: item.clone(),
            })
        })
        .collect();

    if triggers.is_empty() {
        None
    } else {
        Some((triggers, sections))
    }
}

// ── Weekly Review Steps ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewViewer {
    Calendar,
    Projects,
    Waiting,
    Someday,
    Trigger,
    Checklists,
    Altitudes,
    Brainstorm,
}

#[derive(Debug, Clone)]
pub struct ReviewStep {
    pub phase: &'static str,
    pub phase_color: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    pub viewer: Option<ReviewViewer>,
}

pub fn review_steps() -> Vec<ReviewStep> {
    vec![
        // Phase: Get Clear
        ReviewStep {
            phase: "Get Clear",
            phase_color: "link",
            title: "Collect Loose Papers & Materials",
            desc: "Gather all scraps of paper, receipts, sticky notes, business\n\
                   cards, and anything else that's been accumulating. Put them\n\
                   in your physical inbox.",
            viewer: Some(ReviewViewer::Altitudes),
        },
        ReviewStep {
            phase: "Get Clear",
            phase_color: "link",
            title: "Process Physical Inbox to Zero",
            desc: "Go through every item in your physical inbox. For each item:\n\
                   \u{2022} Is it actionable? \u{2192} Decide next action & organize\n\
                   \u{2022} Not actionable? \u{2192} Trash, reference, or incubate\n\
                   \u{2022} Get your inbox to empty.",
            viewer: None,
        },
        ReviewStep {
            phase: "Get Clear",
            phase_color: "link",
            title: "Process Digital Inboxes to Zero",
            desc: "Process all digital inboxes:\n\
                   \u{2022} Email inbox \u{2192} zero\n\
                   \u{2022} Voicemail \u{2192} zero\n\
                   \u{2022} Text messages \u{2192} zero\n\
                   \u{2022} Any app notifications \u{2192} clear",
            viewer: None,
        },
        ReviewStep {
            phase: "Get Clear",
            phase_color: "link",
            title: "Empty Your Head",
            desc: "Do a quick mind sweep. Capture anything still bouncing\n\
                   around in your head \u{2014} open loops, ideas, concerns, tasks.\n\
                   Get it all out and into your inbox or trigger list.\n\n\
                   Tip: Run gtd inbox for a focused capture session.",
            viewer: None,
        },
        // Phase: Get Current
        ReviewStep {
            phase: "Get Current",
            phase_color: "accent",
            title: "Review 'Next Actions' Lists",
            desc: "Look at every item on your next action lists.\n\
                   \u{2022} Check off completed items\n\
                   \u{2022} Update any stale actions\n\
                   \u{2022} Make sure each project has a next action",
            viewer: Some(ReviewViewer::Projects),
        },
        ReviewStep {
            phase: "Get Current",
            phase_color: "accent",
            title: "Review Previous Calendar",
            desc: "Look back at last week's calendar.\n\
                   \u{2022} Any incomplete items to carry forward?\n\
                   \u{2022} Any follow-ups needed from meetings?\n\
                   \u{2022} Capture any loose ends.",
            viewer: Some(ReviewViewer::Calendar),
        },
        ReviewStep {
            phase: "Get Current",
            phase_color: "accent",
            title: "Review Upcoming Calendar",
            desc: "Look at the next 2-4 weeks of your calendar.\n\
                   \u{2022} Any prep needed for upcoming events?\n\
                   \u{2022} Any travel arrangements to make?\n\
                   \u{2022} Any items that need to be delegated or deferred?",
            viewer: Some(ReviewViewer::Calendar),
        },
        ReviewStep {
            phase: "Get Current",
            phase_color: "accent",
            title: "Review 'Waiting For' List",
            desc: "Check each item you're waiting on from others.\n\
                   \u{2022} Any need follow-up?\n\
                   \u{2022} Any that have come in and need acknowledgment?\n\
                   \u{2022} Any that are no longer needed?",
            viewer: Some(ReviewViewer::Waiting),
        },
        ReviewStep {
            phase: "Get Current",
            phase_color: "accent",
            title: "Review 'Project' Lists",
            desc: "Review your full project list (any outcome requiring 2+ steps).\n\
                   \u{2022} Is each project still active and relevant?\n\
                   \u{2022} Does each have a clearly defined next action?\n\
                   \u{2022} Any projects to mark complete?\n\
                   \u{2022} Any new projects to add?",
            viewer: Some(ReviewViewer::Projects),
        },
        ReviewStep {
            phase: "Get Current",
            phase_color: "accent",
            title: "Review 'Someday/Maybe' List",
            desc: "Look through your someday/maybe list.\n\
                   \u{2022} Any items ready to activate as projects?\n\
                   \u{2022} Any items no longer interesting? Remove them.\n\
                   \u{2022} Any new someday/maybe items to add?",
            viewer: Some(ReviewViewer::Someday),
        },
        ReviewStep {
            phase: "Get Current",
            phase_color: "accent",
            title: "Review Any Relevant Checklists",
            desc: "Check any reference checklists you maintain.\n\
                   \u{2022} Travel packing lists\n\
                   \u{2022} Weekly routines\n\
                   \u{2022} Project templates\n\
                   \u{2022} Any other recurring checklists",
            viewer: Some(ReviewViewer::Checklists),
        },
        // Phase: Get Creative
        ReviewStep {
            phase: "Get Creative",
            phase_color: "purple",
            title: "Review Trigger List for New Ideas",
            desc: "Use your trigger list to generate new ideas.\n\
                   \u{2022} Run gtd trigger to walk through it\n\
                   \u{2022} Anything new come to mind?\n\
                   \u{2022} Any new projects, actions, or someday/maybe items?\n\n\
                   Tip: Run gtd trigger for a guided trigger list session.",
            viewer: Some(ReviewViewer::Trigger),
        },
        ReviewStep {
            phase: "Get Creative",
            phase_color: "purple",
            title: "Brainstorm New Ideas",
            desc: "Review your brainstorm ideas and wild thoughts.\n\
                   \u{2022} What ideas are worth pursuing?\n\
                   \u{2022} Any patterns in what you're avoiding?\n\
                   \u{2022} Press 'b' to brainstorm 30 new ideas with AI",
            viewer: Some(ReviewViewer::Brainstorm),
        },
    ]
}
// ── Profile (About Me) ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub core_values: Vec<String>,
    #[serde(default)]
    pub roles: Vec<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: String::new(),
            purpose: String::new(),
            core_values: Vec::new(),
            roles: Vec::new(),
        }
    }
}

pub fn purpose_file() -> PathBuf {
    data_dir().join("purpose.md")
}

fn parse_profile_md(content: &str) -> Profile {
    let mut profile = Profile::default();
    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(heading) = trimmed.strip_prefix("## ") {
            current_section = heading.trim().to_lowercase();
            continue;
        }

        if let Some(item) = trimmed.strip_prefix("- ") {
            let item = item.trim();
            if item.is_empty() { continue; }

            match current_section.as_str() {
                "identity" => {
                    if let Some(val) = item.strip_prefix("Name:") {
                        profile.name = val.trim().to_string();
                    } else if let Some(val) = item.strip_prefix("Purpose:") {
                        profile.purpose = val.trim().to_string();
                    }
                }
                "core values" => {
                    profile.core_values.push(item.to_string());
                }
                "roles" => {
                    profile.roles.push(item.to_string());
                }

                _ => {}
            }
        }
    }
    profile
}

pub fn load_profile() -> Profile {
    let path = purpose_file();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            return parse_profile_md(&contents);
        }
    }
    Profile::default()
}

// ── Goals (H3) ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum GoalStatus {
    Active,
    Completed,
    Deferred,
    Dropped,
}

impl GoalStatus {
    pub fn _as_str(&self) -> &str {
        match self {
            GoalStatus::Active => "active",
            GoalStatus::Completed => "completed",
            GoalStatus::Deferred => "deferred",
            GoalStatus::Dropped => "dropped",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "completed" => GoalStatus::Completed,
            "deferred" => GoalStatus::Deferred,
            "dropped" => GoalStatus::Dropped,
            _ => GoalStatus::Active,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Goal {
    pub text: String,
    pub category: String,
    pub target_date: Option<String>,
    pub status: GoalStatus,
}

impl Default for Goal {
    fn default() -> Self {
        Self {
            text: String::new(),
            category: String::new(),
            target_date: None,
            status: GoalStatus::Active,
        }
    }
}

pub fn goals_file() -> PathBuf {
    data_dir().join("goals.md")
}

fn parse_goals_md(content: &str) -> Vec<Goal> {
    let mut goals = Vec::new();
    let mut current_section = "active";

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(heading) = trimmed.strip_prefix("## ") {
            let h = heading.trim().to_lowercase();
            if h.contains("completed") {
                current_section = "completed";
            } else if h.contains("deferred") {
                current_section = "deferred";
            } else if h.contains("dropped") {
                current_section = "dropped";
            } else {
                current_section = "active";
            }
            continue;
        }

        if let Some(item) = trimmed.strip_prefix("- ") {
            let item = item.trim();
            if item.is_empty() { continue; }

            let mut goal = Goal::default();
            goal.status = GoalStatus::from_str(current_section);

            // Extract inline HTML comments: <!-- key: value -->
            let mut text = item.to_string();
            while let Some(start) = text.find("<!--") {
                if let Some(end_offset) = text[start..].find("-->") {
                    let abs_end = start + end_offset;
                    let comment = &text[start + 4..abs_end];
                    for part in comment.split_whitespace() {
                        if let Some(val) = part.strip_prefix("category:") {
                            goal.category = val.trim().to_string();
                        } else if let Some(val) = part.strip_prefix("target:") {
                            goal.target_date = Some(val.trim().to_string());
                        }
                    }
                    text.replace_range(start..abs_end + 3, "");
                } else {
                    break;
                }
            }
            goal.text = text.trim().to_string();
            goals.push(goal);
            continue;
        }

        // Parse HTML comment metadata: <!-- category: X target: YYYY-MM-DD -->
        if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            let inner = trimmed[4..trimmed.len() - 3].trim();
            if let Some(last_goal) = goals.last_mut() {
                for part in inner.split_whitespace() {
                    if let Some(val) = part.strip_prefix("category:") {
                        last_goal.category = val.trim().to_string();
                    } else if let Some(val) = part.strip_prefix("target:") {
                        last_goal.target_date = Some(val.trim().to_string());
                    }
                }
            }
        }
    }
    goals
}

pub fn load_goals() -> Vec<Goal> {
    let path = goals_file();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            return parse_goals_md(&contents);
        }
    }
    Vec::new()
}

// ── Areas of Focus (H2) ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Area {
    pub name: String,
    pub description: String,
    pub standards: Vec<String>,
}

pub fn areas_file() -> PathBuf {
    data_dir().join("areas.md")
}

fn parse_areas_md(content: &str) -> Vec<Area> {
    let mut areas: Vec<Area> = Vec::new();
    let mut current_area: Option<Area> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip title and description lines
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            continue;
        }
        if trimmed.starts_with('>') {
            continue;
        }

        // New area section
        if let Some(heading) = trimmed.strip_prefix("## ") {
            if let Some(area) = current_area.take() {
                areas.push(area);
            }
            current_area = Some(Area {
                name: heading.trim().to_string(),
                description: String::new(),
                standards: Vec::new(),
            });
            continue;
        }

        // Description line (italic text like *description*)
        if trimmed.starts_with('*') && trimmed.ends_with('*') && trimmed.len() > 2 {
            if let Some(ref mut area) = current_area {
                area.description = trimmed[1..trimmed.len()-1].to_string();
            }
            continue;
        }

        // Standards items
        if let Some(item) = trimmed.strip_prefix("- ") {
            let item = item.trim();
            if item.is_empty() { continue; }
            if let Some(ref mut area) = current_area {
                area.standards.push(item.to_string());
            }
            continue;
        }
    }

    if let Some(area) = current_area {
        areas.push(area);
    }

    areas
}

pub fn load_areas() -> Vec<Area> {
    let path = areas_file();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            return parse_areas_md(&contents);
        }
    }
    Vec::new()
}

// ── Vision (H4) ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VisionArea {
    pub category: String,
    pub vision_text: String,
    pub picture_of_success: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Vision {
    pub time_horizon: String,
    pub areas: Vec<VisionArea>,
}

impl Default for Vision {
    fn default() -> Self {
        Self {
            time_horizon: "3 years".into(),
            areas: Vec::new(),
        }
    }
}

pub fn vision_file() -> PathBuf {
    data_dir().join("vision.md")
}

fn parse_vision_md(content: &str) -> Vision {
    let mut vision = Vision::default();
    let mut current_area: Option<VisionArea> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(heading) = trimmed.strip_prefix("# ") {
            if !trimmed.starts_with("## ") {
                vision.time_horizon = heading.trim().to_string();
                continue;
            }
        }

        if let Some(heading) = trimmed.strip_prefix("## ") {
            if let Some(area) = current_area.take() {
                vision.areas.push(area);
            }
            current_area = Some(VisionArea {
                category: heading.trim().to_string(),
                vision_text: String::new(),
                picture_of_success: Vec::new(),
            });
            continue;
        }

        if let Some(heading) = trimmed.strip_prefix("### ") {
            if let Some(ref mut _area) = current_area {
                if heading.trim().to_lowercase().contains("success") {
                    continue;
                }
            }
        }

        if let Some(item) = trimmed.strip_prefix("- ") {
            let item = item.trim();
            if item.is_empty() { continue; }
            if let Some(ref mut area) = current_area {
                if area.vision_text.is_empty() {
                    area.vision_text = item.to_string();
                } else {
                    area.picture_of_success.push(item.to_string());
                }
            }
            continue;
        }

        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some(ref mut area) = current_area {
                if area.vision_text.is_empty() {
                    area.vision_text = trimmed.to_string();
                } else {
                    area.vision_text.push(' ');
                    area.vision_text.push_str(trimmed);
                }
            }
        }
    }

    if let Some(area) = current_area {
        vision.areas.push(area);
    }

    vision
}

fn _render_vision_md(vision: &Vision) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}

", vision.time_horizon));

    for area in &vision.areas {
        out.push_str(&format!("## {}
", area.category));
        out.push_str(&format!("{}

", area.vision_text));
        if !area.picture_of_success.is_empty() {
            out.push_str("### Picture of Success
");
            for item in &area.picture_of_success {
                out.push_str(&format!("- {}
", item));
            }
            out.push_str("
");
        }
    }

    out
}

pub fn load_vision() -> Vision {
    let path = vision_file();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            return parse_vision_md(&contents);
        }
    }
    Vision::default()
}

pub fn _save_vision(vision: &Vision) {
    ensure_dirs();
    let path = vision_file();
    let md = _render_vision_md(vision);
    fs::write(path, md).ok();
}

// ── Weekly Board ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WeeklyBoard {
    pub score: Option<u8>,
    pub score_note: Option<String>,
    pub partner_notes: Option<String>,
    pub accomplishments: Vec<String>,
    pub struggles: Vec<String>,
}

impl Default for WeeklyBoard {
    fn default() -> Self {
        Self {
            score: None,
            score_note: None,
            partner_notes: None,
            accomplishments: Vec::new(),
            struggles: Vec::new(),

        }
    }
}

pub fn weekly_dir() -> PathBuf {
    data_dir().join("weekly")
}

pub fn weekly_board_path(date_str: &str) -> PathBuf {
    weekly_dir().join(format!("{}.md", date_str))
}

pub fn load_weekly_board(date_str: &str) -> WeeklyBoard {
    let path = weekly_board_path(date_str);
    if !path.exists() {
        return WeeklyBoard::default();
    }
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return WeeklyBoard::default(),
    };
    parse_markdown_board(&contents)
}

pub fn save_weekly_board(board: &WeeklyBoard, date_str: &str) {
    fs::create_dir_all(weekly_dir()).ok();
    let path = weekly_board_path(date_str);
    let md = render_markdown_board(board, date_str);
    fs::write(path, md).ok();
}

fn parse_markdown_board(content: &str) -> WeeklyBoard {
    let mut board = WeeklyBoard::default();
    let mut current_section = "";
    let mut section_lines: Vec<String> = Vec::new();

    fn flush_section(board: &mut WeeklyBoard, section: &str, lines: &[String]) {
        match section {
            "Partner Notes" => {
                let text = lines.join("
").trim().to_string();
                if text.is_empty() { return; }
                // Extract score from first line: "Score: N/10 — summary"
                let first_line = lines.first().map(|s| s.as_str()).unwrap_or("");
                if let Some(score_start) = first_line.find("Score:") {
                    let after = &first_line[score_start + 6..].trim();
                    if let Some(slash) = after.find('/') {
                        if let Ok(score) = after[..slash].trim().parse::<u8>() {
                            board.score = Some(score);
                        }
                    }
                    // Extract score note after " — " or " - "
                    if let Some(dash) = first_line.find(" — ") {
                        let after_dash = &first_line[dash + " — ".len()..];
                        board.score_note = Some(after_dash.trim().to_string());
                    } else if let Some(dash) = first_line.find(" - ") {
                        let after_dash = &first_line[dash + 3..];
                        board.score_note = Some(after_dash.trim().to_string());
                    }
                }
                // Partner notes = everything except the score line
                let notes: Vec<&str> = lines.iter().skip(1).map(|s| s.as_str()).collect();
                let notes_text = notes.join("
").trim().to_string();
                if !notes_text.is_empty() {
                    board.partner_notes = Some(notes_text);
                }
            }
            "Accomplishments" => {
                for line in lines {
                    let trimmed = line.trim();
                    if let Some(item) = trimmed.strip_prefix("- ") {
                        board.accomplishments.push(item.to_string());
                    }
                }
            }
            "Struggles" => {
                for line in lines {
                    let trimmed = line.trim();
                    if let Some(item) = trimmed.strip_prefix("- ") {
                        board.struggles.push(item.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    for line in content.lines() {
        // ## heading = section boundary
        if let Some(heading) = line.strip_prefix("## ") {
            flush_section(&mut board, current_section, &section_lines);
            current_section = heading.trim();
            section_lines.clear();
            continue;
        }

        // Skip the top-level # heading
        if line.starts_with("# ") && !line.starts_with("## ") {
            continue;
        }

        section_lines.push(line.to_string());
    }

    // Flush last section
    flush_section(&mut board, current_section, &section_lines);

    board
}

fn render_markdown_board(board: &WeeklyBoard, date_str: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Weekly Board — {}\n\n", date_str));

    // Partner Notes
    out.push_str("## Partner Notes\n");
    if let Some(score) = board.score {
        let note = board.score_note.as_deref().unwrap_or("");
        if note.is_empty() {
            out.push_str(&format!("Score: {}/10\n", score));
        } else {
            out.push_str(&format!("Score: {}/10 — {}\n", score, note));
        }
    }
    if let Some(ref notes) = board.partner_notes {
        out.push_str("\n");
        out.push_str(notes);
        out.push_str("\n");
    }
    out.push_str("\n");

    // Accomplishments
    out.push_str("## Accomplishments\n");
    if board.accomplishments.is_empty() {
        out.push_str("\n");
    } else {
        for item in &board.accomplishments {
            out.push_str(&format!("- {}\n", item));
        }
    }
    out.push_str("\n");

    // Struggles
    out.push_str("## Struggles\n");
    if board.struggles.is_empty() {
        out.push_str("\n");
    } else {
        for item in &board.struggles {
            out.push_str(&format!("- {}\n", item));
        }
    }
    out.push_str("\n");

    out
}

/// Find the most recent weekly board before a given date.
pub fn _previous_weekly_board(before: &str) -> Option<(String, WeeklyBoard)> {
    let dir = weekly_dir();
    if !dir.exists() {
        return None;
    }
    let mut dates: Vec<String> = fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.strip_suffix(".md")
                .map(|s| s.to_string())
        })
        .filter(|d| d.as_str() < before)
        .collect();
    dates.sort();
    dates.last().map(|d| {
        let board = load_weekly_board(d);
        (d.clone(), board)
    })
}

/// List all weekly board dates, sorted ascending.
pub fn list_weekly_board_dates() -> Vec<String> {
    let dir = weekly_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut dates: Vec<String> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".md")
                .or_else(|| name.strip_suffix(".yaml"))
                .map(|s| s.to_string())?;
            if stem == "template" { return None; }
            Some(stem)
        })
        .collect();
    dates.sort();
    dates
}

/// A next action from a project — the `[>]` marked item.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NextAction {
    pub project_name: String,
    pub action: String,
    pub goal: Option<String>,
}

/// Collect all next actions ([>] items) from every project.
/// Each project can have at most one next action.
#[allow(dead_code)]
pub fn load_next_actions() -> Vec<NextAction> {
    let projects = load_projects();
    let mut actions = Vec::new();
    for proj in &projects {
        if let Some(item) = proj.items.iter().find(|i| i.starts_with("[>] ")) {
            actions.push(NextAction {
                project_name: proj.name.clone(),
                action: item[4..].to_string(),
                goal: proj.goal.clone(),
            });
        }
    }
    actions
}

// ── Calendar ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub description: String,
}

pub fn calendar_path() -> PathBuf {
    data_dir().join("calendar.md")
}

pub fn load_calendar() -> Vec<CalendarEvent> {
    let path = calendar_path();
    if !path.exists() {
        return Vec::new();
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let current_year = chrono::Local::now().year();
    let mut year = current_year;
    let mut events = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(y) = trimmed.strip_prefix("## ") {
            if let Ok(y) = y.trim().parse() {
                year = y;
            }
        } else if let Some(rest) = trimmed.strip_prefix("- ") {
            // Format: MM/DD description
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let date_parts: Vec<&str> = parts[0].split('/').collect();
                if date_parts.len() == 2 {
                    if let (Ok(month), Ok(day)) = (date_parts[0].parse(), date_parts[1].parse()) {
                        events.push(CalendarEvent {
                            year,
                            month,
                            day,
                            description: parts[1].trim().to_string(),
                        });
                    }
                }
            }
        }
    }
    events
}

// ── Projects ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectStatus {
    Active,
    Completed,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub items: Vec<String>,
    pub goal: Option<String>,
    #[allow(dead_code)]
    pub status: ProjectStatus,
}

pub fn projects_path() -> PathBuf {
    data_dir().join("projects.md")
}

pub fn load_projects() -> Vec<Project> {
    load_md_list(&projects_path())
}

fn load_md_list(path: &PathBuf) -> Vec<Project> {
    load_md_list_with_status(path).0
}

/// Load projects from a markdown file, returning (active, completed) split.
/// Recognizes `## Completed` as a section boundary (case-insensitive).
fn load_md_list_with_status(path: &PathBuf) -> (Vec<Project>, Vec<Project>) {
    if !path.exists() {
        return (Vec::new(), Vec::new());
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (Vec::new(), Vec::new()),
    };
    let mut active = Vec::new();
    let mut completed = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_items: Vec<String> = Vec::new();
    let mut current_goal: Option<String> = None;
    let mut current_section = ProjectStatus::Active;

    let flush_project = |current_name: &mut Option<String>,
                          current_items: &mut Vec<String>,
                          current_goal: &mut Option<String>,
                          current_section: &ProjectStatus,
                          active: &mut Vec<Project>,
                          completed: &mut Vec<Project>| {
        if let Some(name) = current_name.take() {
            let project = Project {
                name,
                items: current_items.clone(),
                goal: current_goal.take(),
                status: current_section.clone(),
            };
            match current_section {
                ProjectStatus::Active => active.push(project),
                ProjectStatus::Completed => completed.push(project),
            }
            current_items.clear();
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            let h = heading.trim().to_lowercase();
            if h.contains("completed") {
                // Flush any active project before switching section
                flush_project(&mut current_name, &mut current_items, &mut current_goal, &current_section, &mut active, &mut completed);
                current_section = ProjectStatus::Completed;
                continue;
            } else {
                // Flush previous project (in whatever section we're in)
                flush_project(&mut current_name, &mut current_items, &mut current_goal, &current_section, &mut active, &mut completed);
                current_name = Some(heading.trim().to_string());
                // If we were in completed section, stay there.
                // If a user writes "## Active" explicitly, switch back.
                if h == "active" {
                    current_section = ProjectStatus::Active;
                }
            }
        } else if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            // Parse HTML comment metadata
            let inner = trimmed[4..trimmed.len() - 3].trim();
            for part in inner.split_whitespace() {
                if let Some(val) = part.strip_prefix("goal:") {
                    current_goal = Some(val.trim().to_string());
                }
            }
        } else if let Some(item) = trimmed.strip_prefix("- ") {
            if current_name.is_some() {
                current_items.push(item.to_string());
            }
        }
    }
    // Flush last project
    flush_project(&mut current_name, &mut current_items, &mut current_goal, &current_section, &mut active, &mut completed);

    (active, completed)
}

pub fn save_projects(projects: &[Project]) {
    let path = projects_path();
    let mut content = String::new();
    for proj in projects {
        content.push_str(&format!("## {}\n", proj.name));
        for item in &proj.items {
            content.push_str(&format!("- {}\n", item));
        }
        content.push('\n');
    }
    fs::write(path, content).ok();
}

/// Save projects to file, writing completed projects under a ## Completed section.
#[allow(dead_code)]
pub fn save_projects_with_completed(active: &[Project], completed: &[Project]) {
    let path = projects_path();
    let mut content = String::new();
    for proj in active {
        content.push_str(&format!("## {}\n", proj.name));
        if let Some(ref goal) = proj.goal {
            content.push_str(&format!("<!-- goal: {} -->\n", goal));
        }
        for item in &proj.items {
            content.push_str(&format!("- {}\n", item));
        }
        content.push('\n');
    }
    if !completed.is_empty() {
        content.push_str("## Completed\n");
        for proj in completed {
            content.push_str(&format!("## {}\n", proj.name));
            for item in &proj.items {
                content.push_str(&format!("- {}\n", item));
            }
            content.push('\n');
        }
    }
    fs::write(path, content).ok();
}

/// Load completed projects from the projects file.
#[allow(dead_code)]
pub fn load_completed_projects() -> Vec<Project> {
    load_md_list_with_status(&projects_path()).1
}

// ── Waiting For ──────────────────────────────────────────────────────────

pub fn waiting_for_path() -> PathBuf {
    data_dir().join("waiting-for.md")
}

pub fn load_waiting_for() -> Vec<Project> {
    load_md_list(&waiting_for_path())
}

// ── Agendas ─────────────────────────────────────────────────────────────

pub fn agendas_path() -> PathBuf {
    data_dir().join("agendas.md")
}

pub fn load_agendas() -> Vec<Project> {
    load_md_list(&agendas_path())
}

// ── Someday Maybe ───────────────────────────────────────────────────────

pub fn someday_maybe_path() -> PathBuf {
    data_dir().join("someday-maybe.md")
}

pub fn load_someday_maybe() -> Vec<Project> {
    load_md_list(&someday_maybe_path())
}

// ── Checklists (for review viewer) ──────────────────────────────────────

pub fn checklists_dir() -> PathBuf {
    data_dir().join("checklists")
}

pub fn load_checklist_files() -> Vec<String> {
    let dir = checklists_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir).ok();
        return Vec::new();
    }
    
    let mut files: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    files.sort();
    files
}

pub fn checklist_path(name: &str) -> PathBuf {
    checklists_dir().join(format!("{}.md", name))
}

// ── Brainstorm ──────────────────────────────────────────────────────────

pub fn brainstorm_path() -> PathBuf {
    data_dir().join("brainstorm.md")
}
