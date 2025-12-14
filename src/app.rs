use tui_textarea::TextArea;
use crate::config::Config;
use tokio::sync::mpsc;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Prompting,
    Setup,
    Processing,
    Search,
    SaveAs,
    ConfirmQuit,
}

pub struct App<'a> {
    pub textarea: TextArea<'a>,
    pub prompt_textarea: TextArea<'a>,
    pub setup_textarea: TextArea<'a>,
    pub search_textarea: TextArea<'a>,
    pub filename_input: TextArea<'a>,
    pub should_quit: bool,
    pub mode: AppMode,
    pub filename: String,
    pub config: Config,
    pub ai_response_tx: mpsc::Sender<String>,
    pub ai_response_rx: Option<mpsc::Receiver<String>>,
    pub is_modified: bool,
    pub status_message: Option<String>,
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
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

        let mut filename_input = TextArea::default();
        filename_input.set_placeholder_text("Enter filename...");
        filename_input.set_block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL).title(" Save As "));

        let config = Config::load().unwrap_or(Config::default());
        let mode = if config.api_key.is_empty() {
            AppMode::Setup
        } else {
            AppMode::Normal
        };



        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        let (tx, rx) = mpsc::channel(1);

        Self {
            textarea,
            prompt_textarea,
            setup_textarea,
            search_textarea,
            filename_input,
            should_quit: false,
            mode,
            filename: filename.unwrap_or_else(|| String::from("[No Name]")),
            config,
            ai_response_tx: tx,
            ai_response_rx: Some(rx),
            is_modified: false,
            status_message: None,
            syntax_set,
            theme_set,
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

    pub fn save_file(&mut self) -> anyhow::Result<()> {
        if self.filename == "[No Name]" || self.filename.is_empty() {
            return Err(anyhow::anyhow!("No filename specified"));
        }

        let content = self.textarea.lines().join("\n");
        fs::write(&self.filename, content)?;
        
        self.is_modified = false;
        self.set_status("File Saved!");
        Ok(())
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
    }

    pub fn prompt_save_as(&mut self) {
        self.mode = AppMode::SaveAs;
        // Pre-fill with current filename if it's not [No Name]
        if self.filename != "[No Name]" {
            self.filename_input = TextArea::from(vec![self.filename.clone()]);
        } else {
             self.filename_input = TextArea::default();
        }
        self.filename_input.set_block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL).title(" Save As "));
    }

    pub fn mark_dirty(&mut self) {
        self.is_modified = true;
        self.status_message = None; // Clear status on edit
    }

    pub fn detect_language(&self) -> Option<String> {
        if let Some(syntax) = self.syntax_set.find_syntax_for_file(&self.filename).ok().flatten() {
            return Some(syntax.name.clone());
        }
        None
    }
}