use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use super::super::state::PeripheralState;

pub fn draw(f: &mut Frame, area: Rect, state: &PeripheralState, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Peripherals (SVD) ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.peripherals.is_empty() {
        let msg = Paragraph::new("No SVD data loaded");
        f.render_widget(msg, inner);
        return;
    }

    let lines: Vec<Line> = state
        .peripherals
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let selected = state.selected_peripheral == Some(i);
            let style = if selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if selected { "▸ " } else { "  " };
            Line::from(Span::styled(format!("{}{}", prefix, name), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines).scroll((state.scroll_offset as u16, 0));
    f.render_widget(paragraph, inner);
}
