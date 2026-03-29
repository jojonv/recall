use clap::{Parser, Subcommand};
use recall::{
    config::{load_config, resolve_file_path, Config},
    note::Note,
    storage::Storage,
    tui::run_tui,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new note
    Add {
        /// Note text to add
        #[arg(required = true, num_args = 1..)]
        text: Vec<String>,
    },
    /// List notes to stdout
    List,
    /// Catch-all for notebook shortcuts like `r w "note"` or `r w list`
    #[command(external_subcommand)]
    External(Vec<String>),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = load_config(None)?;
    let default_file_path = resolve_file_path(&config)?;
    let default_storage = Storage::new(default_file_path)?;

    // Handle commands
    match cli.command {
        Some(Commands::Add { text }) => {
            let note_text = text.join(" ");
            if note_text.trim().is_empty() {
                return Err("Note text cannot be empty".into());
            }
            add_note(&default_storage, note_text)?;
        }
        Some(Commands::List) => {
            list_notes(&default_storage)?;
        }
        Some(Commands::External(args)) => {
            handle_external_command(args, &config, &default_storage)?;
        }
        None => {
            let notes = default_storage.load_notes()?;
            run_tui(notes, &default_storage)?;
        }
    }

    Ok(())
}

fn add_note(storage: &Storage, text: String) -> Result<(), Box<dyn std::error::Error>> {
    let note = Note::new(text);
    storage.add_note(&note)?;
    println!("Note added!");
    Ok(())
}

fn list_notes(storage: &Storage) -> Result<(), Box<dyn std::error::Error>> {
    let notes = storage.load_notes()?;
    if notes.is_empty() {
        println!("No notes found.");
        return Ok(());
    }

    for note in notes {
        println!("{}", note.display_line());
    }
    Ok(())
}

fn handle_external_command(
    args: Vec<String>,
    config: &Config,
    default_storage: &Storage,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        let notes = default_storage.load_notes()?;
        run_tui(notes, default_storage)?;
        return Ok(());
    }

    let first = &args[0];

    // Check if first arg is a notebook name
    if let Some(notebook_path) = config.resolve_notebook_path(first) {
        let storage = Storage::new(notebook_path)?;

        // Check if second arg is a command
        if args.len() >= 2 {
            match args[1].as_str() {
                "list" => {
                    return list_notes(&storage);
                }
                "add" => {
                    if args.len() >= 3 {
                        let text = args[2..].join(" ");
                        return add_note(&storage, text);
                    } else {
                        return Err("Usage: r <notebook> add <note text>".into());
                    }
                }
                _ => {
                    // Rest is note text for this notebook
                    let text = args[1..].join(" ");
                    if text.trim().is_empty() {
                        return Err("Note text cannot be empty".into());
                    }
                    return add_note(&storage, text);
                }
            }
        } else {
            // Just notebook name, no command -> launch TUI for this notebook
            let notes = storage.load_notes()?;
            run_tui(notes, &storage)?;
            return Ok(());
        }
    }

    // Not a notebook - treat all args as note text for default notebook
    let text = args.join(" ");
    if text.trim().is_empty() {
        return Err("Note text cannot be empty".into());
    }
    add_note(default_storage, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use recall::config::Config;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_config(work_path: &str, personal_path: &str) -> Config {
        let mut notebooks = HashMap::new();
        notebooks.insert("w".to_string(), work_path.to_string());
        notebooks.insert("p".to_string(), personal_path.to_string());

        Config {
            file: None,
            notebooks: Some(notebooks),
        }
    }

    #[test]
    fn test_external_notebook_shorthand() {
        let temp_dir = TempDir::new().unwrap();
        let work_path = temp_dir.path().join("work.md");
        let personal_path = temp_dir.path().join("personal.md");

        let config =
            create_test_config(work_path.to_str().unwrap(), personal_path.to_str().unwrap());

        let default_storage =
            Storage::new(temp_dir.path().join("default.md").to_path_buf()).unwrap();

        // Test: notebook shortcut adds to correct notebook
        let result = handle_external_command(
            vec!["w".to_string(), "work note".to_string()],
            &config,
            &default_storage,
        );
        assert!(result.is_ok());

        // Verify note was added to work file
        let work_storage = Storage::new(work_path.to_path_buf()).unwrap();
        let notes = work_storage.load_notes().unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].text, "work note");
    }

    #[test]
    fn test_external_notebook_list() {
        let temp_dir = TempDir::new().unwrap();
        let work_path = temp_dir.path().join("work.md");
        let personal_path = temp_dir.path().join("personal.md");

        let config =
            create_test_config(work_path.to_str().unwrap(), personal_path.to_str().unwrap());

        // Add a note to work notebook first
        let work_storage = Storage::new(work_path.to_path_buf()).unwrap();
        work_storage
            .add_note(&Note::new("test work note".to_string()))
            .unwrap();

        let default_storage =
            Storage::new(temp_dir.path().join("default.md").to_path_buf()).unwrap();

        // Test: notebook + list should list notes from that notebook
        // Since this prints to stdout, we just verify it doesn't error
        let result = handle_external_command(
            vec!["w".to_string(), "list".to_string()],
            &config,
            &default_storage,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_external_notebook_add_explicit() {
        let temp_dir = TempDir::new().unwrap();
        let work_path = temp_dir.path().join("work.md");
        let personal_path = temp_dir.path().join("personal.md");

        let config =
            create_test_config(work_path.to_str().unwrap(), personal_path.to_str().unwrap());

        let default_storage =
            Storage::new(temp_dir.path().join("default.md").to_path_buf()).unwrap();

        // Test: notebook + add + text
        let result = handle_external_command(
            vec![
                "p".to_string(),
                "add".to_string(),
                "personal".to_string(),
                "note".to_string(),
            ],
            &config,
            &default_storage,
        );
        assert!(result.is_ok());

        // Verify note was added to personal file
        let personal_storage = Storage::new(personal_path.to_path_buf()).unwrap();
        let notes = personal_storage.load_notes().unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].text, "personal note");
    }

    #[test]
    fn test_external_non_notebook_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let work_path = temp_dir.path().join("work.md");
        let personal_path = temp_dir.path().join("personal.md");

        let config =
            create_test_config(work_path.to_str().unwrap(), personal_path.to_str().unwrap());

        let default_path = temp_dir.path().join("default.md");
        let default_storage = Storage::new(default_path.to_path_buf()).unwrap();

        // Test: non-notebook args should add to default notebook
        let result = handle_external_command(
            vec!["not".to_string(), "a".to_string(), "notebook".to_string()],
            &config,
            &default_storage,
        );
        assert!(result.is_ok());

        // Verify note was added to default file
        let notes = default_storage.load_notes().unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].text, "not a notebook");
    }

    #[test]
    fn test_external_notebook_add_without_text_error() {
        let temp_dir = TempDir::new().unwrap();
        let work_path = temp_dir.path().join("work.md");
        let personal_path = temp_dir.path().join("personal.md");

        let config =
            create_test_config(work_path.to_str().unwrap(), personal_path.to_str().unwrap());

        let default_storage =
            Storage::new(temp_dir.path().join("default.md").to_path_buf()).unwrap();

        // Test: notebook + add without text should error
        let result = handle_external_command(
            vec!["w".to_string(), "add".to_string()],
            &config,
            &default_storage,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Usage"));
    }
}
