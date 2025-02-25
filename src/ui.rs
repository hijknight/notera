use crossterm::{
    execute, event::{self, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use std::io::{stdout, Result};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    Terminal,
};

pub struct NoteUI;

impl NoteUI {
    pub fn draw_ui(notes: &str) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| {
                let size = f.size();

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // Header len
                        Constraint::Min(1)      // Notes section
                    ])
                    .split(size);

                let title = Block::default().borders(Borders::ALL).title(" Note Taker -- 'q' to exit");
                f.render_widget(title, layout[0]);

                let notes_display = Paragraph::new(notes)
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(notes_display, layout[1]);
            })?;

            // q to exit
            if let Ok(true) = event::poll(std::time::Duration::from_millis(100)) {
                if let Ok(event::Event::Key(key)) = event::read() {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }
}
