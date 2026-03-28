use crate::note::Note;
use dirs::home_dir;
use std::fs::{create_dir_all, read_to_string, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct Storage {
    file_path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut path = home_dir().ok_or("Could not find home directory")?;
        path.push(".recall");

        // Create the directory if it doesn't exist
        create_dir_all(&path)?;

        path.push("notes.md");

        Ok(Self { file_path: path })
    }


    pub fn load_notes(&self) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
        // Create the file if it doesn't exist
        if !self.file_path.exists() {
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(&self.file_path)?;
            return Ok(vec![]);
        }

        let content = read_to_string(&self.file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let mut notes = Vec::new();
        let mut current_note_lines = Vec::new();

        for line in lines {
            if line.starts_with("- [ ] ") && !current_note_lines.is_empty() {
                // Start of a new note, process the previous one
                if let Some(note) = Note::from_markdown(current_note_lines.as_slice()) {
                    notes.push(note);
                }
                current_note_lines.clear();
                current_note_lines.push(line);
            } else {
                current_note_lines.push(line);
            }
        }

        // Process the last note
        if !current_note_lines.is_empty() {
            if let Some(note) = Note::from_markdown(current_note_lines.as_slice()) {
                notes.push(note);
            }
        }

        // Sort by timestamp, newest first
        notes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(notes)
    }

    pub fn add_note(&self, note: &Note) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.file_path)?;

        writeln!(file, "{}", note.to_markdown())?;

        Ok(())
    }
}
