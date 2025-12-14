use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::{App, AppMode};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Editor
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    f.render_widget(&app.textarea, chunks[1]);
    render_footer(f, app, chunks[2]);

    if app.mode == AppMode::Prompting {
        render_ai_popup(f, app);
    } else if app.mode == AppMode::Setup {
        render_setup_screen(f, app);
    } else if app.mode == AppMode::Processing {
        render_processing_popup(f);
    } else if app.mode == AppMode::Search {
        render_search_bar(f, app);
    }
}

fn render_setup_screen(f: &mut Frame, app: &mut App) {
    f.render_widget(Clear, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(3), // Instructions
            Constraint::Length(3), // Input
            Constraint::Percentage(30),
        ])
        .split(f.area());

    let instructions = Paragraph::new(vec![
        Line::from(Span::styled("Welcome to NeuroNano!", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
        Line::from("To start, please get an API Key from https://aistudio.google.com/app/apikey"),
    ])
    .alignment(ratatui::layout::Alignment::Center)
    .block(Block::default().borders(Borders::NONE));

    f.render_widget(instructions, chunks[1]);

    let block = Block::default()
        .title(" API Key ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    
    app.setup_textarea.set_block(block);
    f.render_widget(&app.setup_textarea, chunks[2]);
}

fn render_processing_popup(f: &mut Frame) {
    let area = centered_rect(40, 10, f.area());
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Blue).fg(Color::White));
    
    let text = Paragraph::new("🧠 NeuroNano is thinking...")
        .alignment(ratatui::layout::Alignment::Center)
        .block(block);
        
    f.render_widget(text, area);
}

fn render_search_bar(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3), // Search bar
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // We render the search bar just above the footer
    f.render_widget(&app.search_textarea, chunks[1]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let header_style = Style::default().fg(Color::Black).bg(Color::Cyan);
    let header_text = Line::from(vec![
        Span::styled("  NeuroNano  ", header_style.add_modifier(Modifier::BOLD)),
        Span::styled(format!("  {}  ", app.filename), header_style),
    ]);
    
    let block = Block::default().style(header_style);
    let paragraph = Paragraph::new(header_text).block(block);
    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_style = Style::default().fg(Color::Black).bg(Color::White);
    
    let shortcuts = match app.mode {
        AppMode::Normal => Line::from(vec![
            Span::styled("^Q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Exit  "),
            Span::styled("^O", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Save  "),
            Span::styled("^K", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Cut  "),
            Span::styled("^U", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Paste  "),
            Span::styled("^F", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Search  "),
            Span::styled("^P", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" AI Prompt  "),
        ]),
        AppMode::Prompting => Line::from(vec![
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel  "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Generate  "),
        ]),
        AppMode::Setup => Line::from(vec![
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Quit  "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Save & Start  "),
        ]),
        AppMode::Processing => Line::from(vec![
            Span::raw(" Processing... Please wait. "),
        ]),
        AppMode::Search => Line::from(vec![
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel  "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Find  "),
        ]),
    };

    let block = Block::default().style(footer_style);
    let paragraph = Paragraph::new(shortcuts).block(block);
    f.render_widget(paragraph, area);
}

fn render_ai_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 20, f.area());
    
    f.render_widget(Clear, area); // Clear the area so the editor doesn't show through

    let block = Block::default()
        .title("✨ AI Magic Prompt")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    
    app.prompt_textarea.set_block(block);
    f.render_widget(&app.prompt_textarea, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}