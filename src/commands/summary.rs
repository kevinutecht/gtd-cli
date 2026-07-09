// Summary — plain text overview of your GTD system

use std::io;

use chrono::{Datelike, Local};

use crate::data;

pub fn run() -> io::Result<()> {
    let profile = data::load_profile();
    let vision = data::load_vision();
    let goals = data::load_goals();
    let areas = data::load_areas();
    let projects = data::load_projects();
    let inbox_items = load_inbox_open();
    let events = data::load_calendar();
    let agendas = data::load_agendas();
    let waiting = data::load_waiting_for();

    // ── Purpose (H5) ────────────────────────────────────────
    if !profile.purpose.is_empty() {
        println!("## Purpose");
        println!();
        println!("> {}", profile.purpose);
        println!();
    }

    // ── Vision (H4) ─────────────────────────────────────────
    if !vision.areas.is_empty() {
        println!("## Vision ({})", vision.time_horizon);
        println!();
        for area in &vision.areas {
            println!("- **{}:** {}", area.category, area.vision_text);
            for item in &area.picture_of_success {
                println!("  - {}", item);
            }
        }
        println!();
    }

    // ── Areas of Focus (H2) ─────────────────────────────────
    if !areas.is_empty() {
        println!("## Areas of Focus ({})", areas.len());
        println!();
        for area in &areas {
            if area.description.is_empty() {
                println!("- **{}**", area.name);
            } else {
                println!("- **{}** — {}", area.name, area.description);
            }
            for std in &area.standards {
                println!("  - {}", std);
            }
        }
        println!();
    }

    // ── Goals (H3) ──────────────────────────────────────────
    let active_goals: Vec<_> = goals.iter()
        .filter(|g| g.status == data::GoalStatus::Active)
        .collect();

    println!("## Goals ({})", active_goals.len());
    println!();
    if active_goals.is_empty() {
        println!("No active goals");
    } else {
        for goal in &active_goals {
            if goal.category.is_empty() {
                println!("- [>] {}", goal.text);
            } else {
                println!("- [>] {} ({})", goal.text, goal.category);
            }
        }
    }
    println!();

    // ── Projects (H1) ───────────────────────────────────────
    // (Moved after Areas to follow the horizon flow: H5→H4→H3→H2→H1)
    let active_projects: Vec<_> = projects.iter()
        .filter(|p| !p.items.is_empty())
        .collect();

    if !active_projects.is_empty() {
        println!("## Projects ({})", active_projects.len());
        println!();
        for proj in &active_projects {
            if let Some(ref goal) = proj.goal {
                println!("- {} → {}", proj.name, goal);
            } else {
                println!("- {} → ???", proj.name);
            }
            for item in &proj.items {
                println!("  - {}", item);
            }
        }
        println!();
    }

    // ── Waiting For ─────────────────────────────────────────
    if !waiting.is_empty() {
        println!("## Waiting For ({})", waiting.len());
        println!();
        for item in &waiting {
            println!("- {}", item.name);
            for subitem in &item.items {
                println!("  - {}", subitem);
            }
        }
        println!();
    }

    // ── Agendas ─────────────────────────────────────────────
    if !agendas.is_empty() {
        println!("## Agendas ({})", agendas.len());
        println!();
        for agenda in &agendas {
            println!("- {}", agenda.name);
            for item in &agenda.items {
                println!("  - {}", item);
            }
        }
        println!();
    }

    // ── Calendar (first 5 upcoming) ─────────────────────────
    if !events.is_empty() {
        let today = Local::now();
        let today_num = today.year() * 10000 + today.month() as i32 * 100 + today.day() as i32;

        let upcoming: Vec<_> = events.iter()
            .filter(|e| {
                let event_num = e.year * 10000 + e.month as i32 * 100 + e.day as i32;
                event_num >= today_num
            })
            .take(5)
            .collect();

        if !upcoming.is_empty() {
            println!("## Upcoming");
            println!();
            for event in &upcoming {
                let event_num = event.year * 10000 + event.month as i32 * 100 + event.day as i32;
                let is_today = event_num == today_num;
                let date_str = format!("{:04}-{:02}-{:02}", event.year, event.month, event.day);
                if is_today {
                    println!("- **{}** {}", date_str, event.description);
                } else {
                    println!("- {} {}", date_str, event.description);
                }
            }
            println!();
        }
    }

    // ── Next Actions ─────────────────────────────────────────
    let next_actions = data::load_next_actions();
    if !next_actions.is_empty() {
        println!("## Next Actions ({})", next_actions.len());
        println!();
        for action in &next_actions {
            println!("- [>] {} → {}", action.project_name, action.action);
        }
        println!();
    }

    // ── Accountability ──────────────────────────────────────
    let accountability = load_latest_accountability();
    if let Some(ref acc) = accountability {
        println!("## Accountability (from {})", acc.date);
        println!();
        println!("**Pattern to Watch:**");
        println!("- {}", acc.pattern);
        println!();
        println!("**One Challenge:**");
        println!("- {}", acc.challenge);
        println!();
    }

    // ── Inbox ───────────────────────────────────────────────
    if !inbox_items.is_empty() {
        println!("## Inbox ({})", inbox_items.len());
        println!();
        for item in &inbox_items {
            println!("- [ ] {}", item);
        }
        println!();
    }

    Ok(())
}

struct AccountabilityNote {
    date: String,
    pattern: String,
    challenge: String,
}

fn load_latest_accountability() -> Option<AccountabilityNote> {
    let weekly_dir = data::data_dir().join("weekly");
    if !weekly_dir.exists() {
        return None;
    }

    let mut entries: Vec<String> = std::fs::read_dir(&weekly_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name != "template.md")
        .collect();

    entries.sort();
    entries.reverse();

    for filename in &entries {
        let filepath = weekly_dir.join(filename);
        let content = match std::fs::read_to_string(&filepath) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let pattern = extract_section(&content, "Pattern to Watch:");
        let challenge = extract_section(&content, "One Challenge:");

        if let (Some(pattern), Some(challenge)) = (pattern, challenge) {
            let date = filename.trim_end_matches(".md").to_string();
            return Some(AccountabilityNote { date, pattern, challenge });
        }
    }

    None
}

fn extract_section(content: &str, header: &str) -> Option<String> {
    let header_pos = content.find(header)?;
    let after_header = &content[header_pos + header.len()..];

    let mut lines = Vec::new();
    for line in after_header.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        if trimmed.starts_with("## ") {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("- ") {
            lines.push(rest.to_string());
        } else {
            lines.push(trimmed.to_string());
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

fn load_inbox_open() -> Vec<String> {
    let filepath = data::data_dir().join("inbox.md");
    if !filepath.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&filepath) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            items.push(rest.to_string());
        }
    }
    items
}
