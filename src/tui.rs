use crate::note::Note;
use crate::storage::Storage;
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
}

impl App {
    fn new(notes: Vec<Note>) -> Self {
        let mut state = ListState::default();
        if !notes.is_empty() {
            state.select(Some(0));
        }

        Self { state, notes }
    }

    fn build_list_items(&self) -> Vec<ListItem<'static>> {
        self.notes
            .iter()
            .map(|note| {
                let content = format!(
                    "{} - {}",
                    note.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    note.text.lines().next().unwrap_or("")
                );
                let style = if note.done {
                    Style::default()
                        .add_modifier(Modifier::DIM)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default()
                };
                ListItem::new(content).style(style)
            })
            .collect()
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.notes.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.notes.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
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
                    if let Some(i) = app.state.selected() {
                        if i < app.notes.len() {
                            // Clone, toggle, and save before mutating
                            let mut notes_to_save = app.notes.clone();
                            notes_to_save[i].toggle_done();
                            if storage.save_notes(&notes_to_save).is_ok() {
                                app.notes = notes_to_save;
                            }
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

    #[test]
    fn test_app_navigation() {
        let notes = vec![
            Note::new("Note 1".to_string()),
            Note::new("Note 2".to_string()),
        ];
        let mut app = App::new(notes);

        // Initial selection should be 0 if items are not empty
        assert_eq!(app.state.selected(), Some(0));

        // Move to next
        app.next();
        assert_eq!(app.state.selected(), Some(1));

        // Wrap around to 0
        app.next();
        assert_eq!(app.state.selected(), Some(0));

        // Move to previous (should wrap back to 1)
        app.previous();
        assert_eq!(app.state.selected(), Some(1));
    }

    #[test]
    fn test_app_empty_navigation() {
        let mut app = App::new(vec![]);
        assert_eq!(app.state.selected(), None);

        app.next();
        assert_eq!(app.state.selected(), Some(0)); // Depending on logic, might select 0

        // Toggle on empty app should not panic - directly toggle via index
        if let Some(i) = app.state.selected() {
            if i < app.notes.len() {
                app.notes[i].toggle_done();
            }
        }
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
}
