use crate::note::Note;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::layout::Alignment;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

pub fn run_tui(notes: Vec<Note>) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut selected = 0;
    let items: Vec<ListItem> = notes
        .iter()
        .map(|note| {
            let content = format!(
                "{} - {}",
                note.timestamp.format("%Y-%m-%d %H:%M:%S"),
                note.text.lines().next().unwrap_or("")
            );
            ListItem::new(content)
        })
        .collect();

    // Run the TUI
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
                .split(f.area());

            let items = items.clone();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Recall Notes"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_widget(list, chunks[0]);

            let status_text = "q/Esc: quit | ↑/k: up | ↓/j: down";
            let status = Paragraph::new(status_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            f.render_widget(status, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('j') | KeyCode::Down => {
                    if selected < items.len().saturating_sub(1) {
                        selected += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
