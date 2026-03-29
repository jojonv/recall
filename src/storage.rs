use crate::note::Note;
use std::fs::{create_dir_all, read_to_string, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct Storage {
    file_path: PathBuf,
}

impl Storage {
    /// Create a new Storage instance with the specified file path.
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { file_path: path })
    }

    fn ensure_parent_dir(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.file_path.parent() {
            if !parent.exists() && !parent.as_os_str().is_empty() {
                create_dir_all(parent)?;
            }
        }
        Ok(())
    }

    pub fn load_notes(&self) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
        if !self.file_path.exists() {
            self.ensure_parent_dir()?;
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
            if Note::is_note_header(line) && !current_note_lines.is_empty() {
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
        self.ensure_parent_dir()?;
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.file_path)?;

        writeln!(file, "{}", note.to_markdown())?;
        Ok(())
    }

    pub fn save_notes(&self, notes: &[Note]) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_parent_dir()?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;

        for note in notes {
            writeln!(file, "{}", note.to_markdown())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};
    use tempfile::tempdir;

    #[test]
    fn test_storage_roundtrip() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("notes.md");
        let storage = Storage::new(file_path).unwrap();

        let note = Note::new("Test persistent note".to_string());
        storage.add_note(&note).unwrap();

        let loaded = storage.load_notes().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, "Test persistent note");
    }

    #[test]
    fn test_multi_note_roundtrip() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("notes.md");
        let storage = Storage::new(file_path).unwrap();

        let dt1 = Local.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        let note1 = Note::from_parts(dt1, "Note 1".to_string());

        let dt2 = Local.with_ymd_and_hms(2026, 3, 28, 11, 0, 0).unwrap();
        let note2 = Note::from_parts(dt2, "Note 2".to_string());

        storage.add_note(&note1).unwrap();
        storage.add_note(&note2).unwrap();

        let loaded = storage.load_notes().unwrap();
        assert_eq!(loaded.len(), 2);
        // Sorted by timestamp DESC (newest first)
        assert_eq!(loaded[0].text, "Note 2");
        assert_eq!(loaded[1].text, "Note 1");
    }

    #[test]
    fn test_multiline_note_roundtrip() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("notes.md");
        let storage = Storage::new(file_path).unwrap();

        let text = "Line 1\nLine 2\nLine 3";
        let note = Note::new(text.to_string());
        storage.add_note(&note).unwrap();

        let loaded = storage.load_notes().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, text);
    }

    #[test]
    fn test_empty_file_load() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("notes.md");
        let storage = Storage::new(file_path.clone()).unwrap();

        // Ensure file exists but is empty
        std::fs::File::create(&file_path).unwrap();

        let loaded = storage.load_notes().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_load_notes_with_garbage_content() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("notes.md");
        let storage = Storage::new(file_path.clone()).unwrap();

        std::fs::write(
            &file_path,
            "This is garbage\nIt doesn't start with the note prefix",
        )
        .unwrap();

        let loaded = storage.load_notes().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_pre_existing_directory() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().to_path_buf();
        std::fs::create_dir_all(&path).unwrap();

        let file_path = path.join("notes.md");
        let storage = Storage::new(file_path.clone()).unwrap();

        // File shouldn't exist yet
        assert!(!file_path.exists());

        storage.load_notes().unwrap(); // this creates the file
        assert!(file_path.exists());
    }

    #[test]
    fn test_nested_directory_creation() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("a/b/c/notes.md");

        // Ensure parent directories do not exist
        assert!(!tmp.path().join("a").exists());

        let storage = Storage::new(file_path.clone()).unwrap();

        // Constructor shouldn't create dirs anymore
        assert!(!tmp.path().join("a").exists());

        storage.load_notes().unwrap();

        // Parent directories should be created now
        assert!(tmp.path().join("a/b/c").exists());
        assert!(file_path.exists());
    }

    #[test]
    fn test_note_body_with_note_like_line_is_not_split() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("notes.md");
        let storage = Storage::new(file_path).unwrap();

        let text = "Line 1\n- [ ] Buy milk\nLine 3";
        let note = Note::new(text.to_string());
        storage.add_note(&note).unwrap();

        let loaded = storage.load_notes().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, text);
    }

    #[test]
    fn test_note_body_with_real_timestamp_is_split() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("notes.md");
        let storage = Storage::new(file_path).unwrap();

        // This simulates someone manually typing a full header inside a note body
        let text = "Line 1\n- [ ] 2026-03-28 14:30:00\nLine 3";
        let note = Note::new(text.to_string());
        storage.add_note(&note).unwrap();

        let loaded = storage.load_notes().unwrap();
        // Since we split on anything matching Note::is_note_header, this should split.
        // It's an acceptable edge case.
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_save_notes_roundtrip() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("notes.md");
        let storage = Storage::new(file_path).unwrap();

        let dt1 = Local.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        let note1 = Note::from_parts_with_done(dt1, "Note 1".to_string(), true);

        let dt2 = Local.with_ymd_and_hms(2026, 3, 28, 11, 0, 0).unwrap();
        let note2 = Note::from_parts_with_done(dt2, "Note 2".to_string(), false);

        let notes = vec![note1, note2];
        storage.save_notes(&notes).unwrap();

        let loaded = storage.load_notes().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].text, "Note 2");
        assert!(!loaded[0].done);
        assert_eq!(loaded[1].text, "Note 1");
        assert!(loaded[1].done);
    }

    #[test]
    fn test_save_notes_with_toggles() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("notes.md");
        let storage = Storage::new(file_path).unwrap();

        // Create initial notes
        let note1 = Note::new("First note".to_string());
        let note2 = Note::new("Second note".to_string());
        storage.add_note(&note1).unwrap();
        storage.add_note(&note2).unwrap();

        // Load, toggle first note, and save
        let mut notes = storage.load_notes().unwrap();
        notes[0].toggle_done();
        storage.save_notes(&notes).unwrap();

        // Reload and verify
        let loaded = storage.load_notes().unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded[0].done);
        assert!(!loaded[1].done);
    }
}
