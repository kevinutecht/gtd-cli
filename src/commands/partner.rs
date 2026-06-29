// Partner — write accountability partner notes into a weekly board
// Called by pi's accountability-partner skill

use std::io;

use crate::data;

/// Write partner notes to a weekly board.
/// Usage: gtd partner write <date> [--score N] [--note "text"]
///   or: gtd partner write <date> < notes_from_stdin
pub fn run_write(date_str: &str, score: Option<u8>, notes: &str) -> io::Result<()> {
    let mut board = data::load_weekly_board(date_str);

    board.score = Some(score.unwrap_or(0));
    board.partner_notes = Some(wrap_notes(notes.trim(), 110));

    data::save_weekly_board(&board, date_str);

    eprintln!("Partner notes written to {}", data::weekly_board_path(date_str).display());
    Ok(())
}

/// Wrap note text at `max_width` characters, preserving paragraph breaks
/// (blank lines) and list item structure (- bullets, ### headings, etc.).
fn wrap_notes(text: &str, max_width: usize) -> String {
    let mut result = String::new();
    for paragraph in text.split("\n\n") {
        let trimmed = paragraph.trim();
        if trimmed.is_empty() { continue; }

        // Check if this paragraph is a list or heading block — preserve structure
        let is_structured = trimmed.lines().any(|l| {
            let t = l.trim();
            t.starts_with("###") || t.starts_with("##") ||
            t.starts_with("- ") || t.starts_with("* ") ||
            t.starts_with("\u{2022} ") || t.starts_with("Score:")
        });

        if is_structured {
            // Wrap each line individually, preserving prefixes
            for line in trimmed.lines() {
                let t = line.trim();
                if t.is_empty() {
                    result.push('\n');
                    continue;
                }
                // Detect prefix (bullets, headings, etc.)
                let (prefix, content) = if let Some(rest) = t.strip_prefix("### ") {
                    ("### ", rest)
                } else if let Some(rest) = t.strip_prefix("## ") {
                    ("## ", rest)
                } else if let Some(rest) = t.strip_prefix("- ") {
                    ("- ", rest)
                } else if let Some(rest) = t.strip_prefix("* ") {
                    ("* ", rest)
                } else if let Some(rest) = t.strip_prefix("\u{2022} ") {
                    ("\u{2022} ", rest)
                } else if t.starts_with("Score:") {
                    result.push_str(t);
                    result.push('\n');
                    continue;
                } else {
                    ("", t)
                };
                let indent = " ".repeat(prefix.len());
                for (i, wrapped) in wrap_paragraph(content, max_width.saturating_sub(prefix.len())).iter().enumerate() {
                    if i == 0 {
                        result.push_str(prefix);
                    } else {
                        result.push_str(&indent);
                    }
                    result.push_str(wrapped);
                    result.push('\n');
                }
            }
        } else {
            // Plain paragraph — wrap as a block
            for wrapped in wrap_paragraph(trimmed, max_width) {
                result.push_str(&wrapped);
                result.push('\n');
            }
        }
        result.push('\n');
    }
    result
}

/// Wrap a single paragraph (no internal line breaks) at `max_width`.
fn wrap_paragraph(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.len() + 1 + word.len() > max_width {
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

/// Import an old-style markdown battleboard into the yaml format.
pub fn run_import(filepath: &str) -> io::Result<()> {
    let content = std::fs::read_to_string(filepath)
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("{}: {}", filepath, e)))?;

    // Try to extract date from filename: battleboard_YYYY-MM-DD.md
    let filename = std::path::Path::new(filepath)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    let date_str = filename
        .strip_prefix("battleboard_")
        .and_then(|s| s.strip_suffix(".md"))
        .unwrap_or("unknown");

    let mut board = data::WeeklyBoard::default();

    // Parse the markdown
    let _section = "";
    let mut partner_notes_lines: Vec<String> = Vec::new();
    let mut in_partner_notes = false;
    let mut in_accomplishments = false;
    let mut in_struggles = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect score from partner notes header
        if trimmed.contains("Score:") && trimmed.contains("/10") {
            if let Some(score_start) = trimmed.find("Score:") {
                let after_score = &trimmed[score_start + 6..].trim();
                if let Some(slash) = after_score.find('/') {
                    if let Ok(score) = after_score[..slash].trim().parse::<u8>() {
                        board.score = Some(score);
                    }
                }
            }
            // Also extract the summary text after the dash
            if let Some(dash) = trimmed.find(" \u{2014} ") {
                board.score_note = Some(trimmed[dash + " \u{2014} ".len()..].trim().to_string());
            } else if let Some(dash) = trimmed.find(" - ") {
                board.score_note = Some(trimmed[dash + 3..].trim().to_string());
            }
        }

        // Section detection
        if trimmed.starts_with("## ") {
            let heading = trimmed[3..].trim();

            // Close previous sections
            #[allow(unused_assignments)]
            if in_partner_notes {
                board.partner_notes = Some(wrap_notes(&partner_notes_lines.join("\n").trim(), 110));
                partner_notes_lines.clear();
                in_partner_notes = false;
            }

            if heading.contains("ACCOUNTABILITY PARTNER") || heading.contains("PARTNER NOTES") {
                in_partner_notes = true;
                in_accomplishments = false;
                in_struggles = false;
                continue;
            } else if heading.contains("LAST WEEK") || heading.contains("PAST WEEK") {
                in_partner_notes = false;
                in_accomplishments = false;
                in_struggles = false;
                continue;
            } else if heading.contains("THIS WEEK") || heading.contains("FOCUS") {
                in_partner_notes = false;
                in_accomplishments = false;
                in_struggles = false;
                continue;
            } else {
                in_partner_notes = false;
                in_accomplishments = false;
                in_struggles = false;
            }
        }

        if trimmed.starts_with("### ") {
            let heading = trimmed[4..].trim();

            if in_partner_notes {
                partner_notes_lines.push(line.to_string());
            } else if heading.contains("Accomplishment") {
                in_accomplishments = true;
                in_struggles = false;
            } else if heading.contains("Struggle") {
                in_accomplishments = false;
                in_struggles = true;
            } else {
                in_accomplishments = false;
                in_struggles = false;
            }
            continue;
        }

        // Collect partner notes
        if in_partner_notes {
            partner_notes_lines.push(line.to_string());
            continue;
        }

        // Parse list items
        if let Some(item) = parse_list_item(trimmed) {
            if in_accomplishments {
                board.accomplishments.push(item.to_string());
            } else if in_struggles {
                board.struggles.push(item.to_string());
            }
        }
    }

    // Close any open partner notes section
    if in_partner_notes && !partner_notes_lines.is_empty() {
        board.partner_notes = Some(wrap_notes(&partner_notes_lines.join("\n").trim(), 110));
    }

    data::save_weekly_board(&board, date_str);
    eprintln!("Imported {} \u{2192} {}", filepath, data::weekly_board_path(date_str).display());
    Ok(())
}

fn parse_list_item(line: &str) -> Option<&str> {
    if let Some(rest) = line.strip_prefix("- ") {
        Some(rest.trim())
    } else if let Some(rest) = line.strip_prefix("* ") {
        Some(rest.trim())
    } else if let Some(rest) = line.strip_prefix("\u{2022} ") {
        Some(rest.trim())
    } else {
        None
    }
}
