use clap::Parser;
use recall::{note::Note, storage::{Storage, resolve_storage_path}, tui::run_tui};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional note text to add
    note_text: Option<String>,

    /// Explicit add command
    #[arg(long)]
    add: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let storage_path = resolve_storage_path()?;
    let storage = Storage::new(storage_path)?;

    // If note text is provided, add it
    if let Some(note_text) = cli.note_text {
        let note = Note::new(note_text);
        storage.add_note(&note)?;
        println!("Note added!");
        return Ok(());
    }

    // If explicit add command is used
    if let Some(note_text) = cli.add {
        let note = Note::new(note_text);
        storage.add_note(&note)?;
        println!("Note added!");
        return Ok(());
    }

    // Otherwise, launch TUI
    let notes = storage.load_notes()?;
    run_tui(notes)?;

    Ok(())
}
