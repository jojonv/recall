use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

#[derive(Debug, Clone)]
pub struct Note {
    pub timestamp: DateTime<Local>,
    pub text: String,
    pub done: bool,
}

impl Note {
    pub fn new(text: String) -> Self {
        Self {
            timestamp: Local::now(),
            text,
            done: false,
        }
    }

    pub fn from_parts(timestamp: DateTime<Local>, text: String) -> Self {
        Self {
            timestamp,
            text,
            done: false,
        }
    }

    pub fn from_parts_with_done(timestamp: DateTime<Local>, text: String, done: bool) -> Self {
        Self {
            timestamp,
            text,
            done,
        }
    }

    pub fn toggle_done(&mut self) {
        self.done = !self.done;
    }

    pub fn to_markdown(&self) -> String {
        let checkbox = if self.done { "[x]" } else { "[ ]" };
        format!(
            "- {} {}\n{}",
            checkbox,
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.text
        )
    }

    pub fn from_markdown(lines: &[&str]) -> Option<Self> {
        if lines.is_empty() {
            return None;
        }

        // Parse the first line: "- [ ] 2026-03-28 14:30:00" or "- [x] 2026-03-28 14:30:00"
        let line = lines[0];
        let (naive_datetime, done) = Self::parse_note_header(line)?;

        // Note: from_local_datetime can return ambiguous results during DST transitions.
        // For simplicity, we take the earliest valid mapping.
        let timestamp = match Local.from_local_datetime(&naive_datetime) {
            chrono::LocalResult::Single(dt) => dt,
            chrono::LocalResult::Ambiguous(dt1, _) => dt1,
            chrono::LocalResult::None => return None,
        };

        // The text is in the following lines until the next note or EOF
        let text = lines[1..].join("\n").trim().to_string();

        Some(Self {
            timestamp,
            text,
            done,
        })
    }

    pub fn is_note_header(line: &str) -> bool {
        Self::parse_note_header(line).is_some()
    }

    pub fn parse_note_header(line: &str) -> Option<(NaiveDateTime, bool)> {
        // Expected formats: "- [ ] 2026-03-28 14:30:00" (25 chars) or "- [x] 2026-03-28 14:30:00" (25 chars)
        let done = if line.starts_with("- [x] ") || line.starts_with("- [X] ") {
            true
        } else if line.starts_with("- [ ] ") {
            false
        } else {
            return None;
        };

        if line.len() != 25 {
            return None;
        }

        let timestamp_str = &line[6..];
        NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|dt| (dt, done))
    }

    pub fn display_line(&self) -> String {
        let checkbox = if self.done { "[x]" } else { "[ ]" };
        let first_line = self.text.lines().next().unwrap_or("");
        format!(
            "{} {} {}",
            checkbox,
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            first_line
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};

    #[test]
    fn test_note_to_markdown() {
        let dt = Local.with_ymd_and_hms(2026, 3, 28, 14, 30, 0).unwrap();
        let note = Note::from_parts(dt, "Test Note".to_string());
        let md = note.to_markdown();
        assert_eq!(md, "- [ ] 2026-03-28 14:30:00\nTest Note");
    }

    #[test]
    fn test_note_from_markdown() {
        let lines = ["- [ ] 2026-03-28 14:30:00", "Line 1", "Line 2"];
        let note = Note::from_markdown(&lines).unwrap();
        assert_eq!(note.text, "Line 1\nLine 2");
        assert_eq!(
            note.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-03-28 14:30:00"
        );
    }

    #[test]
    fn test_note_from_markdown_trims_whitespace() {
        let lines = ["- [ ] 2026-03-28 14:30:00", "  Line 1  ", "  Line 2  "];
        let note = Note::from_markdown(&lines).unwrap();
        assert_eq!(note.text, "Line 1  \n  Line 2");
    }

    #[test]
    fn test_note_from_invalid_markdown() {
        let lines = ["Invalid Line", "More text"];
        assert!(Note::from_markdown(&lines).is_none());
    }

    #[test]
    fn test_is_note_header() {
        assert!(Note::is_note_header("- [ ] 2026-03-28 14:30:00"));
        assert!(Note::is_note_header("- [x] 2026-03-28 14:30:00"));
        assert!(Note::is_note_header("- [X] 2026-03-28 14:30:00"));
        assert!(!Note::is_note_header("- [ ] 2026-03-28 14:30")); // Missing seconds
        assert!(!Note::is_note_header("- [ ] Buy milk"));
        assert!(!Note::is_note_header("No prefix 2026-03-28 14:30:00"));

        // Boundary cases
        assert!(!Note::is_note_header("- [ ] 2026-03-28 14:30:00extra")); // Trailing chars
        assert!(!Note::is_note_header("- [ ] 2026-03-28 14:30:0")); // Too short (24 chars)
    }

    #[test]
    fn test_note_to_markdown_done() {
        let dt = Local.with_ymd_and_hms(2026, 3, 28, 14, 30, 0).unwrap();
        let note = Note::from_parts_with_done(dt, "Test Note".to_string(), true);
        let md = note.to_markdown();
        assert_eq!(md, "- [x] 2026-03-28 14:30:00\nTest Note");
    }

    #[test]
    fn test_note_from_markdown_done() {
        let lines = ["- [x] 2026-03-28 14:30:00", "Line 1", "Line 2"];
        let note = Note::from_markdown(&lines).unwrap();
        assert_eq!(note.text, "Line 1\nLine 2");
        assert!(note.done);
        assert_eq!(
            note.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-03-28 14:30:00"
        );
    }

    #[test]
    fn test_note_from_markdown_unchecked() {
        let lines = ["- [ ] 2026-03-28 14:30:00", "Line 1"];
        let note = Note::from_markdown(&lines).unwrap();
        assert!(!note.done);
    }

    #[test]
    fn test_note_from_markdown_done_uppercase() {
        let lines = ["- [X] 2026-03-28 14:30:00", "Line 1"];
        let note = Note::from_markdown(&lines).unwrap();
        assert!(note.done);
    }

    #[test]
    fn test_toggle_done() {
        let dt = Local.with_ymd_and_hms(2026, 3, 28, 14, 30, 0).unwrap();
        let mut note = Note::from_parts(dt, "Test".to_string());
        assert!(!note.done);
        note.toggle_done();
        assert!(note.done);
        note.toggle_done();
        assert!(!note.done);
    }

    #[test]
    fn test_display_line_open() {
        let dt = Local.with_ymd_and_hms(2026, 3, 28, 14, 30, 0).unwrap();
        let note = Note::from_parts(dt, "This is my note\nWith multiple lines".to_string());
        let line = note.display_line();
        assert_eq!(line, "[ ] 2026-03-28 14:30:00 This is my note");
    }

    #[test]
    fn test_display_line_done() {
        let dt = Local.with_ymd_and_hms(2026, 3, 28, 14, 30, 0).unwrap();
        let note = Note::from_parts_with_done(dt, "Completed task".to_string(), true);
        let line = note.display_line();
        assert_eq!(line, "[x] 2026-03-28 14:30:00 Completed task");
    }

    #[test]
    fn test_display_line_empty() {
        let dt = Local.with_ymd_and_hms(2026, 3, 28, 14, 30, 0).unwrap();
        let note = Note::from_parts(dt, "".to_string());
        let line = note.display_line();
        assert_eq!(line, "[ ] 2026-03-28 14:30:00 ");
    }
}
