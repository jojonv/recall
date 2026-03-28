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
        if !line.starts_with("- [ ] ") {
            return None;
        }

        let timestamp_str = &line[6..];
        let naive_datetime = match NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
        {
            Ok(ndt) => ndt,
            Err(_) => return None,
        };

        let timestamp = Local.from_local_datetime(&naive_datetime).unwrap();

        // The text is in the following lines until the next note or EOF
        let text = lines[1..].join("\n").trim().to_string();

        Some(Self { timestamp, text })
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
        assert_eq!(note.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(), "2026-03-28 14:30:00");
    }

    #[test]
    fn test_note_from_invalid_markdown() {
        let lines = ["Invalid Line", "More text"];
        assert!(Note::from_markdown(&lines).is_none());
    }
}
