mod app;
mod aws;
mod ui;
mod terminal;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use tokio::sync::mpsc;

use app::{App, AppEvent};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new().await?;

    // Event channel
    let (tx, mut rx) = mpsc::channel::<AppEvent>(100);

    // Run app
    let res = run_app(&mut terminal, &mut app, &mut rx, tx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: &mut mpsc::Receiver<AppEvent>,
    tx: mpsc::Sender<AppEvent>,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            if app.can_quit() {
                                return Ok(());
                            }
                        }
                        KeyCode::Char('r') => {
                            let tx = tx.clone();
                            app.refresh(tx).await?;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.next_item();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.previous_item();
                        }
                        KeyCode::Enter => {
                            let tx = tx.clone();
                            app.select_item(tx).await?;
                        }
                        KeyCode::Esc | KeyCode::Backspace => {
                            app.go_back();
                        }
                        KeyCode::Char('e') | KeyCode::Char('s') => {
                            // Execute command on container (e) or SSH to EC2 (s)
                            app.execute_command().await?;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Handle async events
        while let Ok(event) = rx.try_recv() {
            app.handle_event(event).await?;
        }

        if app.should_quit() {
            return Ok(());
        }
    }
}
