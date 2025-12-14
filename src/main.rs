use std::{io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use anyhow::Result;
use clap::Parser;
use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::fs::File;

mod app;
mod config;
mod ui;
mod ai;

use app::{App, AppMode};

use tui_textarea::TextArea;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional file to open
    filename: Option<String>,

    /// Reset configuration (delete config.json)
    #[arg(long)]
    reset: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("neuronano.log").unwrap_or_else(|_| File::create("/dev/null").unwrap()),
    );

    let cli = Cli::parse();

    if cli.reset {
        if std::fs::remove_file("config.json").is_ok() {
            log::info!("Configuration reset: config.json deleted.");
            println!("Configuration reset.");
            return Ok(());
        } else {
            log::warn!("Failed to delete config.json (maybe it didn't exist).");
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(cli.filename);

    // Run app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App<'_>) -> Result<()> {
    loop {
        // Check for AI response
        if let Some(rx) = &mut app.ai_response_rx {
            if let Ok(response) = rx.try_recv() {
                app.textarea = TextArea::from(response.lines().map(|s| s.to_string()));
                app.set_processing(false);
            }
        }

        terminal.draw(|f| ui::ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.mode {
                    AppMode::Normal => match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                            if app.is_modified {
                                app.mode = AppMode::ConfirmQuit;
                            } else {
                                app.quit();
                            }
                        }
                        (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                            app.enter_prompt_mode();
                        }
                        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                            app.textarea.cut();
                            app.mark_dirty();
                        }
                        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                            app.textarea.paste();
                            app.mark_dirty();
                        }
                        (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                            if app.filename != "[No Name]" {
                                if let Err(e) = app.save_file() {
                                    app.set_status(&format!("Error: {}", e));
                                }
                            } else {
                                app.prompt_save_as();
                            }
                        }
                        (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                            app.enter_search_mode();
                        }
                        _ => {
                            if app.textarea.input(key) {
                                app.mark_dirty();
                            }
                        }
                    },
                    AppMode::Prompting => match key.code {
                        KeyCode::Esc => {
                            app.exit_prompt_mode();
                        }
                        KeyCode::Enter => {
                            let api_key = app.config.api_key.clone();
                            let current_code = app.textarea.lines().join("\n");
                            let filename = app.filename.clone();
                            let prompt = app.prompt_textarea.lines().join("\n");
                            let tx = app.ai_response_tx.clone();

                            app.set_processing(true);

                            tokio::spawn(async move {
                                let result = ai::request_gemini(api_key, current_code, filename, prompt).await;
                                match result {
                                    Ok(content) => {
                                        log::info!("Response received successfully.");
                                        let _ = tx.send(content).await;
                                    }
                                    Err(e) => {
                                        log::error!("Gemini Request Failed: {}", e);
                                        let _ = tx.send(format!("Error: {}", e)).await;
                                    }
                                }
                            });
                        }
                        _ => {
                            app.prompt_textarea.input(key);
                        }
                    },
                    AppMode::Setup => match key.code {
                        KeyCode::Esc => app.quit(),
                        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
                        KeyCode::Enter => app.save_config(),
                        _ => {
                            app.setup_textarea.input(key);
                        }
                    },
                    AppMode::Processing => {
                        // Ignore input while processing, or allow quit
                        if let KeyCode::Char('q') = key.code {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                app.quit();
                            }
                        }
                    },
                    AppMode::Search => match key.code {
                        KeyCode::Esc => app.exit_search_mode(),
                        KeyCode::Enter => {
                            if let Some(query) = app.search_textarea.lines().first() {
                                let query = query.to_string();
                                // Simple linear search
                                let lines = app.textarea.lines();
                                for (i, line) in lines.iter().enumerate() {
                                    if let Some(col) = line.find(&query) {
                                        app.textarea.move_cursor(tui_textarea::CursorMove::Jump(i as u16, col as u16));
                                        break;
                                    }
                                }
                            }
                            app.exit_search_mode();
                        }
                        _ => {
                            app.search_textarea.input(key);
                        }
                    },
                    AppMode::SaveAs => match key.code {
                        KeyCode::Esc => {
                            app.mode = AppMode::Normal;
                        }
                        KeyCode::Enter => {
                            if let Some(name) = app.filename_input.lines().first() {
                                if !name.trim().is_empty() {
                                    app.filename = name.trim().to_string();
                                    if let Err(e) = app.save_file() {
                                        app.set_status(&format!("Error: {}", e));
                                    }
                                    app.mode = AppMode::Normal;
                                }
                            }
                        }
                        _ => {
                            app.filename_input.input(key);
                        }
                    },
                    AppMode::ConfirmQuit => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // Try to save first
                            if app.filename == "[No Name]" {
                                app.prompt_save_as();
                            } else {
                                if let Err(e) = app.save_file() {
                                    app.set_status(&format!("Error saving: {}", e));
                                    app.mode = AppMode::Normal; // Go back to fix
                                } else {
                                    app.quit();
                                }
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            app.quit(); // Quit without saving
                        }
                        KeyCode::Esc => {
                            app.mode = AppMode::Normal;
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}