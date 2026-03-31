use crate::note::Note;
use crate::storage::Storage;
use chrono::{Duration, Local, NaiveDate};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::layout::Alignment;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io;

enum DisplayItem {
    DayHeader(String),
    Note(usize),
}

/// RAII guard to ensure terminal state is restored even on panic.
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

struct App {
    state: ListState,
    notes: Vec<Note>,
    display_items: Vec<DisplayItem>,
}

impl App {
    fn new(notes: Vec<Note>) -> Self {
        let mut state = ListState::default();
        let mut display_items = Vec::new();

        if !notes.is_empty() {
            display_items = Self::build_display_items(&notes);
            state.select(Self::first_note_index(&display_items));
        }

        Self {
            state,
            notes,
            display_items,
        }
    }

    fn first_note_index(display_items: &[DisplayItem]) -> Option<usize> {
        display_items
            .iter()
            .position(|item| matches!(item, DisplayItem::Note(_)))
    }

    fn build_display_items(notes: &[Note]) -> Vec<DisplayItem> {
        let mut display_items = Vec::new();
        let today = Local::now().date_naive();
        let yesterday = today - Duration::days(1);

        let mut last_date: Option<NaiveDate> = None;

        for (idx, note) in notes.iter().enumerate() {
            let note_date = note.timestamp.date_naive();

            if last_date != Some(note_date) {
                let header = if note_date == today {
                    "Today".to_string()
                } else if note_date == yesterday {
                    "Yesterday".to_string()
                } else {
                    note_date.format("%Y-%m-%d").to_string()
                };
                display_items.push(DisplayItem::DayHeader(header));
                last_date = Some(note_date);
            }

            display_items.push(DisplayItem::Note(idx));
        }

        display_items
    }

    fn format_note_line(note: &Note) -> String {
        format!(
            "{} - {}",
            note.timestamp.format("%H:%M:%S"),
            note.text.lines().next().unwrap_or("")
        )
    }

    fn build_list_items(&self) -> Vec<ListItem<'static>> {
        self.display_items
            .iter()
            .map(|item| match item {
                DisplayItem::DayHeader(text) => ListItem::new(text.clone()).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                DisplayItem::Note(idx) => {
                    let note = &self.notes[*idx];
                    let content = Self::format_note_line(note);
                    let style = if note.done {
                        Style::default()
                            .add_modifier(Modifier::DIM)
                            .add_modifier(Modifier::CROSSED_OUT)
                    } else {
                        Style::default()
                    };
                    ListItem::new(content).style(style)
                }
            })
            .collect()
    }

    fn next(&mut self) {
        let current = self.state.selected().unwrap_or(0);
        let len = self.display_items.len();

        if len == 0 {
            return;
        }

        let mut next_idx = (current + 1) % len;
        loop {
            if matches!(self.display_items.get(next_idx), Some(DisplayItem::Note(_))) {
                self.state.select(Some(next_idx));
                return;
            }
            if next_idx == current {
                break;
            }
            next_idx = (next_idx + 1) % len;
        }
    }

    fn previous(&mut self) {
        let current = self.state.selected().unwrap_or(0);
        let len = self.display_items.len();

        if len == 0 {
            return;
        }

        let mut prev_idx = (current + len - 1) % len;
        loop {
            if matches!(self.display_items.get(prev_idx), Some(DisplayItem::Note(_))) {
                self.state.select(Some(prev_idx));
                return;
            }
            if prev_idx == current {
                break;
            }
            prev_idx = (prev_idx + len - 1) % len;
        }
    }
}

pub fn run_tui(notes: Vec<Note>, storage: &Storage) -> Result<(), Box<dyn std::error::Error>> {
    let mut guard = TerminalGuard::new()?;
    let mut app = App::new(notes);

    loop {
        guard.terminal.draw(|f| {
            let chunks = Layout::default()
                .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
                .split(f.area());

            let items = app.build_list_items();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Recall Notes"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[0], &mut app.state);

            let status_text = "q/Esc: quit | ↑/k: up | ↓/j: down | d: done";
            let status = Paragraph::new(status_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            f.render_widget(status, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('j') | KeyCode::Down => app.next(),
                KeyCode::Char('k') | KeyCode::Up => app.previous(),
                KeyCode::Char('d') => {
                    if let Some(i) = app.state.selected()
                        && let Some(DisplayItem::Note(note_idx)) = app.display_items.get(i)
                        && *note_idx < app.notes.len()
                    {
                        let mut notes_to_save = app.notes.clone();
                        notes_to_save[*note_idx].toggle_done();
                        if storage.save_notes(&notes_to_save).is_ok() {
                            app.notes = notes_to_save;
                            app.display_items = App::build_display_items(&app.notes);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::Note;
    use chrono::{Datelike, TimeZone};

    #[test]
    fn test_app_navigation() {
        let notes = vec![
            Note::new("Note 1".to_string()),
            Note::new("Note 2".to_string()),
        ];
        let mut app = App::new(notes);

        // Initial selection should point to first Note item (index 1 after DayHeader)
        let initial = app.state.selected().unwrap();
        assert!(matches!(app.display_items[initial], DisplayItem::Note(_)));

        // Move to next
        app.next();
        let second = app.state.selected().unwrap();
        assert_ne!(initial, second);
        assert!(matches!(app.display_items[second], DisplayItem::Note(_)));

        // Move to previous should go back to first
        app.previous();
        assert_eq!(app.state.selected(), Some(initial));
    }

    #[test]
    fn test_app_empty_navigation() {
        let mut app = App::new(vec![]);
        assert!(app.display_items.is_empty());
        assert_eq!(app.state.selected(), None);

        // Navigation on empty app should not panic
        app.next();
        app.previous();
    }

    #[test]
    fn test_app_toggle_done() {
        let notes = vec![
            Note::new("Note 1".to_string()),
            Note::new("Note 2".to_string()),
        ];
        let mut app = App::new(notes);

        assert!(!app.notes[0].done);
        app.notes[0].toggle_done();
        assert!(app.notes[0].done);
        app.notes[0].toggle_done();
        assert!(!app.notes[0].done);

        // Move to next and toggle
        app.next();
        assert!(!app.notes[1].done);
        app.notes[1].toggle_done();
        assert!(app.notes[1].done);
    }

    #[test]
    fn test_build_display_items_same_day() {
        // Use a fixed time in the middle of the day to avoid crossing date boundaries
        // Use a date in definitely in the past (2026-03-28)
        let base_time = Local.with_ymd_and_hms(2026, 3, 28, 12, 0, 0).unwrap();
        let notes = vec![
            Note::from_parts(base_time, "Morning note".to_string()),
            Note::from_parts(base_time + Duration::hours(2), "Afternoon note".to_string()),
            Note::from_parts(base_time + Duration::hours(4), "Evening note".to_string()),
        ];

        let display_items = App::build_display_items(&notes);

        // Should have exactly 4 items: 1 header + 3 notes
        assert_eq!(display_items.len(), 4);
        assert!(matches!(display_items[0], DisplayItem::DayHeader(_)));
        assert!(matches!(display_items[1], DisplayItem::Note(0)));
        assert!(matches!(display_items[2], DisplayItem::Note(1)));
        assert!(matches!(display_items[3], DisplayItem::Note(2)));

        // Header should be the date since base_time is in the past
        if let DisplayItem::DayHeader(header) = &display_items[0] {
            assert_eq!(header, "2026-03-28");
        } else {
            panic!("Expected DayHeader");
        }
    }

    #[test]
    fn test_build_display_items_multiple_days() {
        let yesterday = Local::now() - Duration::days(1);
        let today = Local::now();
        let notes = vec![
            Note::from_parts(yesterday, "Yesterday note".to_string()),
            Note::from_parts(today, "Today note".to_string()),
        ];

        let display_items = App::build_display_items(&notes);

        // Should have 4 items: 2 headers + 2 notes
        assert_eq!(display_items.len(), 4);

        // Verify structure: Header, Note, Header, Note
        let headers: Vec<_> = display_items
            .iter()
            .filter_map(|item| match item {
                DisplayItem::DayHeader(text) => Some(text.clone()),
                _ => None,
            })
            .collect();

        assert_eq!(headers.len(), 2);

        let notes_indices: Vec<_> = display_items
            .iter()
            .filter_map(|item| match item {
                DisplayItem::Note(idx) => Some(*idx),
                _ => None,
            })
            .collect();

        assert_eq!(notes_indices, vec![0, 1]);
    }

    #[test]
    fn test_build_display_items_old_date() {
        let old_date = Local.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        let notes = vec![Note::from_parts(old_date, "Old note".to_string())];

        let display_items = App::build_display_items(&notes);

        assert_eq!(display_items.len(), 2);
        if let DisplayItem::DayHeader(header) = &display_items[0] {
            assert_eq!(header, "2026-03-28");
        } else {
            panic!("Expected DayHeader with date");
        }
        assert!(matches!(display_items[1], DisplayItem::Note(0)));
    }

    #[test]
    fn test_wrap_around_single_note() {
        let note_date = Local.with_ymd_and_hms(2026, 3, 30, 12, 0, 0).unwrap();
        let notes = vec![Note::from_parts(note_date, "Only note".to_string())];

        let mut app = App::new(notes);

        // Initial selection should be the lone note (index 1 after header)
        let initial = app.state.selected().unwrap_or(0);
        assert_eq!(initial, 1);

        // next() should wrap around and still select the note
        app.next();
        let after_next = app.state.selected().unwrap_or(0);
        assert_eq!(after_next, 1);

        // previous() should wrap around and still select the note
        app.previous();
        let after_prev = app.state.selected().unwrap_or(0);
        assert_eq!(after_prev, 1);
    }

    #[test]
    fn test_wrap_around_cross_group() {
        let day1 = Local.with_ymd_and_hms(2026, 3, 29, 12, 0, 0).unwrap();
        let day2 = Local.with_ymd_and_hms(2026, 3, 30, 12, 0, 0).unwrap();
        let notes = vec![
            Note::from_parts(day1, "Day 1 note".to_string()),
            Note::from_parts(day2, "Day 2 note".to_string()),
        ];

        let mut app = App::new(notes);

        // Start at first note
        let first_note = app.state.selected().unwrap_or(0);
        assert!(matches!(app.display_items[first_note], DisplayItem::Note(_)));

        // Move to second note
        app.next();
        let second_note = app.state.selected().unwrap_or(0);
        assert_ne!(first_note, second_note);
        assert!(matches!(app.display_items[second_note], DisplayItem::Note(_)));

        // next() again should wrap back to first note
        app.next();
        let wrapped = app.state.selected().unwrap_or(0);
        assert_eq!(wrapped, first_note);

        // previous() from first note should wrap to last note
        app.previous();
        let wrapped_back = app.state.selected().unwrap_or(0);
        assert_eq!(wrapped_back, second_note);
    }

    #[test]
    fn test_relative_labels() {
        let today = Local::now().date_naive();
        let yesterday = today - Duration::days(1);

        let notes = vec![
            Note::from_parts(
                Local
                    .with_ymd_and_hms(today.year(), today.month(), today.day(), 12, 0, 0)
                    .unwrap(),
                "Today note".to_string(),
            ),
            Note::from_parts(
                Local.with_ymd_and_hms(
                    yesterday.year(),
                    yesterday.month(),
                    yesterday.day(),
                    12,
                    0,
                    0,
                )
                .unwrap(),
                "Yesterday note".to_string(),
            ),
        ];

        let display_items = App::build_display_items(&notes);

        // Should have 4 items: 2 headers + 2 notes
        assert_eq!(display_items.len(), 4);

        // Get the headers
        let headers: Vec<_> = display_items
            .iter()
            .filter_map(|item| match item {
                DisplayItem::DayHeader(text) => Some(text.clone()),
                _ => None,
            })
            .collect();

        // Should have exactly 2 headers
        assert_eq!(headers.len(), 2);

        // First header should be the note created first (can be either Today or Yesterday depending on order)
        // Second header should be the other one
        let has_today = headers.iter().any(|h| h == "Today");
        let has_yesterday = headers.iter().any(|h| h == "Yesterday");

        assert!(
            has_today,
            "Expected to find 'Today' label in headers, got: {:?}",
            headers
        );
        assert!(
            has_yesterday,
            "Expected to find 'Yesterday' label in headers, got: {:?}",
            headers
        );
    }

    #[test]
    fn test_format_note_line_shows_time_only() {
        let timestamp = Local.with_ymd_and_hms(2026, 3, 31, 14, 30, 45).unwrap();
        let note = Note::from_parts(timestamp, "Test note".to_string());
        let formatted = App::format_note_line(&note);

        assert_eq!(formatted, "14:30:45 - Test note");
    }

    #[test]
    fn test_format_note_line_multiline_shows_first_line() {
        let timestamp = Local.with_ymd_and_hms(2026, 3, 31, 14, 30, 45).unwrap();
        let note = Note::from_parts(timestamp, "First line\nSecond line\nThird line".to_string());
        let formatted = App::format_note_line(&note);

        assert_eq!(formatted, "14:30:45 - First line");
    }

    #[test]
    fn test_format_note_line_done_note_same_format() {
        let timestamp = Local.with_ymd_and_hms(2026, 3, 31, 14, 30, 45).unwrap();
        let mut note = Note::from_parts(timestamp, "Test note".to_string());
        let formatted_undone = App::format_note_line(&note);

        note.toggle_done();
        let formatted_done = App::format_note_line(&note);

        assert_eq!(formatted_undone, formatted_done);
        assert_eq!(formatted_done, "14:30:45 - Test note");
    }
}
