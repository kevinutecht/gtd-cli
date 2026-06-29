// Altitudes — alignment view of all horizons (used by review)

use std::io;

use crossterm::style::Color;

use crate::data;

#[allow(dead_code)]
pub struct AltitudesState {
    pub lines: Vec<(String, Color, bool)>,
    pub scroll: usize,
}

#[allow(dead_code)]
impl AltitudesState {
    pub fn new() -> Self {
        Self {
            lines: build_altitudes_lines(),
            scroll: 0,
        }
    }
}

pub fn build_altitudes_lines() -> Vec<(String, Color, bool)> {
    let mut lines = Vec::new();
    
    // H5 - Purpose
    let profile = data::load_profile();
    lines.push(("═══ H5 · Purpose ═══".to_string(), Color::Yellow, true));
    if !profile.name.is_empty() {
        lines.push((format!("  Name: {}", profile.name), Color::White, false));
    }
    if !profile.purpose.is_empty() {
        lines.push((format!("  Purpose: {}", profile.purpose), Color::White, false));
    }
    if !profile.core_values.is_empty() {
        lines.push((format!("  Values: {}", profile.core_values.join(", ")), Color::White, false));
    }
    if !profile.roles.is_empty() {
        lines.push((format!("  Roles: {}", profile.roles.join(", ")), Color::White, false));
    }
    lines.push(("".to_string(), Color::White, false));
    
    // H4 - Vision
    let vision = data::load_vision();
    lines.push(("═══ H4 · Vision (3-year) ═══".to_string(), Color::Yellow, true));
    if vision.areas.is_empty() {
        lines.push(("  No vision areas defined.".to_string(), Color::DarkGrey, false));
    } else {
        for area in &vision.areas {
            lines.push((format!("  {}:", area.category), Color::Cyan, true));
            if !area.picture_of_success.is_empty() {
                for item in &area.picture_of_success {
                    lines.push((format!("    • {}", item), Color::White, false));
                }
            }
        }
    }
    lines.push(("".to_string(), Color::White, false));
    
    // H3 - Goals
    let goals = data::load_goals();
    lines.push(("═══ H3 · Goals ═══".to_string(), Color::Yellow, true));
    let active_goals: Vec<_> = goals.iter().filter(|g| g.status == data::GoalStatus::Active).collect();
    if active_goals.is_empty() {
        lines.push(("  No active goals.".to_string(), Color::DarkGrey, false));
    } else {
        for goal in &active_goals {
            let target = goal.target_date.as_deref().unwrap_or("no date");
            lines.push((format!("  • {} (target: {})", goal.text, target), Color::White, false));
        }
    }
    lines.push(("".to_string(), Color::White, false));
    
    // H2 - Areas
    let areas = data::load_areas();
    lines.push(("═══ H2 · Areas of Focus ═══".to_string(), Color::Yellow, true));
    if areas.is_empty() {
        lines.push(("  No areas defined.".to_string(), Color::DarkGrey, false));
    } else {
        for area in &areas {
            lines.push((format!("  • {}", area.name), Color::White, false));
        }
    }
    lines.push(("".to_string(), Color::White, false));
    
    // H1 - Projects
    let projects = data::load_projects();
    lines.push(("═══ H1 · Projects ═══".to_string(), Color::Yellow, true));
    if projects.is_empty() {
        lines.push(("  No projects.".to_string(), Color::DarkGrey, false));
    } else {
        for project in &projects {
            lines.push((format!("  • {}", project.name), Color::White, false));
        }
    }
    
    lines
}

#[allow(dead_code)]
pub fn run() -> io::Result<()> {
    // Standalone command removed — use review instead
    println!("Altitudes view is available during weekly review (gtd review)");
    Ok(())
}
