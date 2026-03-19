use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use super::super::state::ExpressionState;

pub fn draw(f: &mut Frame, area: Rect, state: &ExpressionState, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Live Expressions ")
        .borders(Borders::ALL)
        .border_style(border_style);

    if state.entries.is_empty() {
        let paragraph = Paragraph::new("Press 'a' to add expression").block(block);
        f.render_widget(paragraph, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Value").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Δ").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .height(1);

    let rows: Vec<Row> = state
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let selected = state.selected == Some(i);
            let style = if selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            let change_indicator = if entry.changed { "●" } else { " " };
            let change_style = if entry.changed {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(entry.name.as_str()),
                Cell::from(entry.value.as_str()),
                Cell::from(change_indicator).style(change_style),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(50),
        Constraint::Percentage(10),
    ];

    let table = Table::new(rows, widths).header(header).block(block);

    f.render_widget(table, area);
}
