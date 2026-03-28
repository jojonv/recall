use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

#[derive(Debug, Clone)]
pub struct Note {
    pub timestamp: DateTime<Local>,
    pub text: String,
}

impl Note {
    pub fn new(text: String) -> Self {
        Self {
            timestamp: Local::now(),
            text,
        }
    }

    pub fn from_parts(timestamp: DateTime<Local>, text: String) -> Self {
        Self { timestamp, text }
    }

    pub fn to_markdown(&self) -> String {
        format!(
            "- [ ] {}\n{}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.text
        )
    }

    pub fn from_markdown(lines: &[&str]) -> Option<Self> {
        if lines.is_empty() {
            return None;
        }

        // Parse the first line: "- [ ] 2026-03-28 14:30:00"
        let line = lines[0];
        let naive_datetime = Self::parse_note_header(line)?;

        // Note: from_local_datetime can return ambiguous results during DST transitions.
        // For simplicity, we take the earliest valid mapping.
        let timestamp = match Local.from_local_datetime(&naive_datetime) {
            chrono::LocalResult::Single(dt) => dt,
            chrono::LocalResult::Ambiguous(dt1, _) => dt1,
            chrono::LocalResult::None => return None,
        };

        // The text is in the following lines until the next note or EOF
        let text = lines[1..].join("\n").trim().to_string();

        Some(Self { timestamp, text })
    }

    pub fn is_note_header(line: &str) -> bool {
        Self::parse_note_header(line).is_some()
    }

    pub fn parse_note_header(line: &str) -> Option<NaiveDateTime> {
        if !line.starts_with("- [ ] ") || line.len() != 25 {
            return None;
        }

        let timestamp_str = &line[6..];
        NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S").ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Local};

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
        assert!(!Note::is_note_header("- [ ] 2026-03-28 14:30")); // Missing seconds
        assert!(!Note::is_note_header("- [ ] Buy milk"));
        assert!(!Note::is_note_header("- [x] 2026-03-28 14:30:00")); // Wrong marker
        assert!(!Note::is_note_header("No prefix 2026-03-28 14:30:00"));
        
        // Boundary cases
        assert!(!Note::is_note_header("- [ ] 2026-03-28 14:30:00extra")); // Trailing chars
        assert!(!Note::is_note_header("- [ ] 2026-03-28 14:30:0")); // Too short (24 chars)
    }
}
