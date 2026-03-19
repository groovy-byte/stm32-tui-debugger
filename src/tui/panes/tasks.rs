use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use super::super::state::TasksState;

pub fn draw(f: &mut Frame, area: Rect, state: &TasksState, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Tasks (FreeRTOS) ")
        .borders(Borders::ALL)
        .border_style(border_style);

    if !state.rtos_detected {
        let paragraph = Paragraph::new(
            "FreeRTOS not detected \u{2014} ensure pxCurrentTCB symbol is in ELF",
        )
        .block(block);
        f.render_widget(paragraph, area);
        return;
    }

    if state.tasks.is_empty() {
        let paragraph = Paragraph::new("No tasks found").block(block);
        f.render_widget(paragraph, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Pri").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("State").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Stack Top").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Stack Base").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .height(1);

    let rows: Vec<Row> = state
        .tasks
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let selected = state.selected == Some(i);
            let base_style = if selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let state_color = match entry.state.as_str() {
                "Running" => Color::Green,
                "Ready" => Color::Cyan,
                "Blocked" => Color::Yellow,
                "Suspended" => Color::DarkGray,
                "Deleted" => Color::Red,
                _ => Color::White,
            };

            Row::new(vec![
                Cell::from(entry.name.as_str()).style(base_style),
                Cell::from(format!("{}", entry.priority)).style(base_style),
                Cell::from(entry.state.as_str()).style(base_style.fg(state_color)),
                Cell::from(format!("0x{:08x}", entry.stack_top)).style(base_style),
                Cell::from(format!("0x{:08x}", entry.stack_base)).style(base_style),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(15),
        Constraint::Percentage(20),
        Constraint::Percentage(17),
        Constraint::Percentage(18),
    ];

    let table = Table::new(rows, widths).header(header).block(block);

    f.render_widget(table, area);
}
