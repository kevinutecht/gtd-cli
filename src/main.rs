use clap::{Parser, Subcommand};

mod ui;
mod data;
mod commands;
#[derive(Parser)]
#[command(name = "gtd")]
#[command(about = "GTD CLI — inbox, projects, review, weekly assessment", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Walk through a GTD trigger list
    Trigger,
    /// Guided GTD weekly review
    Review,
    /// View/edit your purpose (name, values, roles)
    Purpose,
    /// View calendar (upcoming events)
    Cal,
    /// View active projects
    Projects,
    /// View items waiting on others
    Waiting,
    /// View per-person agendas
    Agendas,
    /// View someday/maybe list
    Someday,
    /// Quick overview of your GTD system
    Summary,
    /// View/manage goals (H3)
    Goals,
    /// View/edit vision (H4)
    Vision,
    /// View/edit areas of focus (H2)
    Areas,
    /// Write accountability partner notes (used by pi skill)
    Partner {
        #[command(subcommand)]
        action: PartnerAction,
    },
}

#[derive(Subcommand)]
enum PartnerAction {
    /// Write partner notes to a weekly board
    Write {
        /// Date (YYYY-MM-DD)
        date: String,
        /// Score (1-10)
        #[arg(short, long)]
        score: Option<u8>,
        /// Notes text (or pipe via stdin)
        #[arg(short, long)]
        note: Option<String>,
    },
    /// Import an old markdown weekly board into the current format
    Import {
        /// Path to the markdown battleboard file
        filepath: String,
    },
}

#[allow(dead_code)]
fn print_usage() {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    println!("\x1b[38;2;246;196;83m\x1b[1mgtd\x1b[0m \u{2014} GTD CLI");
    println!();
    println!("\x1b[38;2;155;163;178mWeekly:\x1b[0m");
    println!("  \x1b[38;2;240;201;135mgtd review\x1b[0m       Guided GTD weekly review + assessment");
    println!("  \x1b[38;2;240;201;135mgtd trigger\x1b[0m      Walk through GTD trigger list");
    println!();
    println!("\x1b[38;2;155;163;178mHorizons:\x1b[0m  (opens in {})", editor);
    println!("  \x1b[38;2;240;201;135mgtd purpose\x1b[0m      purpose.md — name, values, roles (H5)");
    println!("  \x1b[38;2;240;201;135mgtd vision\x1b[0m       vision.md — 3-year picture (H4)");
    println!("  \x1b[38;2;240;201;135mgtd goals\x1b[0m        goals.md — active targets (H3)");
    println!("  \x1b[38;2;240;201;135mgtd areas\x1b[0m        areas.md — life areas (H2)");
    println!("  \x1b[38;2;240;201;135mgtd projects\x1b[0m     projects.md — active projects (H1)");
    println!();
    println!("\x1b[38;2;155;163;178mLists:\x1b[0m  (opens in {})", editor);
    println!("  \x1b[38;2;240;201;135mgtd summary\x1b[0m      Quick overview of your GTD system");
    println!("  \x1b[38;2;240;201;135mgtd cal\x1b[0m          calendar.md — upcoming events");
    println!("  \x1b[38;2;240;201;135mgtd waiting\x1b[0m      waiting-for.md — items waiting on others");
    println!("  \x1b[38;2;240;201;135mgtd agendas\x1b[0m      agendas.md — per-person agendas");
    println!("  \x1b[38;2;240;201;135mgtd someday\x1b[0m      someday-maybe.md — someday/maybe list");
    println!();
}

fn main() {
    data::ensure_dirs();

    let cli = Cli::parse();

    let result = match cli.command {
        None => {
            print_usage();
            Ok(())
        }
        Some(Commands::Trigger) => commands::trigger::run(),
        Some(Commands::Review) => commands::review::run(),
        Some(Commands::Purpose) => commands::purpose::run(),
        Some(Commands::Cal) => commands::calendar::run(),
        Some(Commands::Projects) => commands::projects::run(),
        Some(Commands::Waiting) => commands::waiting_for::run(),
        Some(Commands::Agendas) => commands::agendas::run(),
        Some(Commands::Someday) => commands::someday_maybe::run(),
        Some(Commands::Summary) => commands::summary::run(),
        Some(Commands::Goals) => commands::goals::run(),
        Some(Commands::Vision) => commands::vision::run(),
        Some(Commands::Areas) => commands::areas::run(),

        Some(Commands::Partner { action }) => match action {
            PartnerAction::Write { date, score, note } => {
                let notes = note.unwrap_or_else(|| {
                    // Read from stdin if no --note provided
                    let mut buf = String::new();
                    std::io::stdin().read_line(&mut buf).ok();
                    buf
                });
                commands::partner::run_write(&date, score, &notes)
            }
            PartnerAction::Import { filepath } => {
                commands::partner::run_import(&filepath)
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
