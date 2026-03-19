use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
};
use super::state::{TuiState, PaneId};
use super::panes;

pub fn draw(f: &mut Frame, state: &TuiState) {
    let size = f.size();

    // Main vertical split: top (70%) + bottom (27%) + status bar (1 row)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(27),
            Constraint::Length(1),
        ])
        .split(size);

    // Top horizontal split: source (55%) + right column (45%)
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(main_chunks[0]);

    // Right column vertical split: peripherals (50%) + tasks (50%)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(top_chunks[1]);

    // Bottom horizontal split: expressions (50%) + console (50%)
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_chunks[1]);

    // Draw panes
    panes::source::draw(f, top_chunks[0], &state.source_state, state.focused == PaneId::Source);
    panes::peripherals::draw(f, right_chunks[0], &state.peripheral_state, state.focused == PaneId::Peripherals);
    panes::tasks::draw(f, right_chunks[1], &state.tasks_state, state.focused == PaneId::Tasks);
    panes::expressions::draw(f, bottom_chunks[0], &state.expression_state, &state.input, state.input_mode, &state.completion, state.focused == PaneId::Expressions);
    panes::console::draw(f, bottom_chunks[1], &state.console_state, state.focused == PaneId::Console);

    // Status bar
    draw_status_bar(f, main_chunks[2], state);
}

fn draw_status_bar(f: &mut Frame, area: Rect, state: &TuiState) {
    let status = match &state.status_message {
        Some(msg) => msg.as_str(),
        None => "Tab: switch pane | F5: resume | F6: halt | F7: reset | q: quit",
    };
    let bar = Paragraph::new(Span::styled(status, Style::default().fg(Color::White)))
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(bar, area);
}
