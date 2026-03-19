use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use super::super::state::ConsoleState;

pub fn draw(f: &mut Frame, area: Rect, state: &ConsoleState, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Console / RTT ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block.clone(), area);

    let visible = inner.height as usize;
    let total = state.lines.len();
    let scroll = if state.auto_scroll {
        total.saturating_sub(visible)
    } else {
        state.scroll_offset
    };

    let lines: Vec<Line> = state
        .lines
        .iter()
        .skip(scroll)
        .take(visible)
        .map(|line| {
            let style = if line.starts_with("[ERROR]") {
                Style::default().fg(Color::Red)
            } else if line.starts_with("[WARN]") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            };
            Line::from(Span::styled(line.as_str(), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}
