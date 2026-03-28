use crate::note::Note;
use dirs::data_dir;
use std::fs::{create_dir_all, read_to_string, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct Storage {
    file_path: PathBuf,
}

impl Storage {
    /// Create a new Storage instance at the specified directory.
    pub fn new(mut path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            create_dir_all(&path)?;
        }
        path.push("notes.md");
        Ok(Self { file_path: path })
    }

    pub fn load_notes(&self) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
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
                if let Some(note) = Note::from_markdown(current_note_lines.as_slice()) {
                    notes.push(note);
                }
                current_note_lines.clear();
                current_note_lines.push(line);
            } else {
                current_note_lines.push(line);
            }
        }

        if !current_note_lines.is_empty() {
            if let Some(note) = Note::from_markdown(current_note_lines.as_slice()) {
                notes.push(note);
            }
        }

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

/// Resolves the storage directory using the standard OS data directory.
pub fn resolve_storage_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = data_dir().ok_or("Could not find data directory")?;
    path.push("recall");
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_storage_roundtrip() {
        let tmp = tempdir().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf()).unwrap();
        
        let note = Note::new("Test persistent note".to_string());
        storage.add_note(&note).unwrap();
        
        let loaded = storage.load_notes().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, "Test persistent note");
    }

    #[test]
    fn test_resolve_path_returns_standard_dir() {
        let path = resolve_storage_path().unwrap();
        // Verifies it uses the standard 'recall' folder name
        assert!(path.to_string_lossy().contains("recall"));
    }
}
