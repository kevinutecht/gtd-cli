// Waiting For — open waiting-for.md in editor

use std::io;
use std::process::Command;

use crate::data;

pub fn run() -> io::Result<()> {
    let path = data::waiting_for_path();
    open_in_editor(&path)
}

fn open_in_editor(path: &std::path::Path) -> io::Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let status = Command::new(&editor)
        .arg(path)
        .status()?;
    
    if !status.success() {
        eprintln!("Editor exited with status: {}", status);
    }
    Ok(())
}
