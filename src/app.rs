use tui_textarea::TextArea;
use crate::config::Config;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Prompting,
    Setup,
    Processing,
    Search,
}

pub struct App<'a> {
    pub textarea: TextArea<'a>,
    pub prompt_textarea: TextArea<'a>,
    pub setup_textarea: TextArea<'a>,
    pub search_textarea: TextArea<'a>,
    pub should_quit: bool,
    pub mode: AppMode,
    pub filename: String,
    pub config: Config,
    pub ai_response_tx: mpsc::Sender<String>,
    pub ai_response_rx: Option<mpsc::Receiver<String>>,
}

use std::fs;

impl<'a> App<'a> {
    pub fn new(filename: Option<String>) -> Self {
        let textarea = if let Some(ref file) = filename {
            if let Ok(content) = fs::read_to_string(file) {
                let mut textarea = TextArea::from(content.lines().map(|s| s.to_string()));
                textarea.set_line_number_style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray));
                textarea
            } else {
                let mut textarea = TextArea::default();
                textarea.set_line_number_style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray));
                textarea
            }
        } else {
            let mut textarea = TextArea::default();
            textarea.set_line_number_style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray));
            textarea
        };
        
        let mut prompt_textarea = TextArea::default();
        prompt_textarea.set_placeholder_text("Describe your wish (e.g., 'Refactor this function')...");

        let mut setup_textarea = TextArea::default();
        setup_textarea.set_placeholder_text("Paste your Google Gemini API Key here...");

        let mut search_textarea = TextArea::default();
        search_textarea.set_placeholder_text("Search...");
        search_textarea.set_block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL).title(" Search "));

        let config = Config::load().unwrap_or(Config::default());
        let mode = if config.api_key.is_empty() {
            AppMode::Setup
        } else {
            AppMode::Normal
        };

        let (tx, rx) = mpsc::channel(1);

        Self {
            textarea,
            prompt_textarea,
            setup_textarea,
            search_textarea,
            should_quit: false,
            mode,
            filename: filename.unwrap_or_else(|| String::from("[No Name]")),
            config,
            ai_response_tx: tx,
            ai_response_rx: Some(rx),
        }
    }

    pub fn save_config(&mut self) {
        if let Some(key) = self.setup_textarea.lines().first() {
            self.config.api_key = key.trim().to_string();
            if let Err(e) = self.config.save() {
                // In a real app we might want to show an error message
                eprintln!("Failed to save config: {}", e);
            } else {
                self.mode = AppMode::Normal;
            }
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn enter_prompt_mode(&mut self) {
        self.mode = AppMode::Prompting;
    }

    pub fn exit_prompt_mode(&mut self) {
        self.mode = AppMode::Normal;
        // Optional: Clear prompt on exit or keep history? For now, let's keep it simple.
    }

    pub fn set_processing(&mut self, is_processing: bool) {
        if is_processing {
            self.mode = AppMode::Processing;
        } else {
            self.mode = AppMode::Normal;
        }
    }

    pub fn enter_search_mode(&mut self) {
        self.mode = AppMode::Search;
    }

    pub fn exit_search_mode(&mut self) {
        self.mode = AppMode::Normal;
        // Clear search text on exit? Maybe keep it for next time.
    }
}