use ratatui::{
    layout::{ Constraint, Direction, Layout },
    style::{ Color, Modifier, Style },
    text::Line,
    widgets::{ Block, Borders, List, ListItem, Paragraph },
    Frame,
};

use crate::App;

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(f.area());

    // Mesajlar Alanı
    let messages: Vec<ListItem> = app.messages
        .iter()
        .map(|m| ListItem::new(Line::from(m.clone())))
        .collect();

    let messages_list = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Sohbet"))
        .style(Style::default().fg(Color::Blue))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::LightYellow))
        .highlight_symbol(">>");

    f.render_stateful_widget(messages_list, chunks[0], &mut app.message_state);

    // Giriş alanı
    let input_paragraph = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Mesaj Yaz"));

    f.render_widget(input_paragraph, chunks[1]);

    // İmleç konumunu ayarla
    f.set_cursor_position((chunks[1].x + (app.input.chars().count() as u16) + 1, chunks[1].y + 1));
}
