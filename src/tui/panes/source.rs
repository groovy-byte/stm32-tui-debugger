use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use super::super::state::SourceState;

pub fn draw(f: &mut Frame, area: Rect, state: &SourceState, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = match &state.file_path {
        Some(p) => format!(" Source: {} ", p),
        None => " Source (no file) ".to_string(),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let visible_lines = inner.height as usize;
    let start = state.scroll_offset;
    let end = (start + visible_lines).min(state.lines.len());

    let lines: Vec<Line> = state.lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_num = start + i + 1;
            let is_current = state.current_line == Some(line_num);
            let num_span = Span::styled(
                format!("{:>4} ", line_num),
                Style::default().fg(Color::DarkGray),
            );
            let code_style = if is_current {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default()
            };
            let code_span = Span::styled(line.as_str(), code_style);
            Line::from(vec![num_span, code_span])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}
