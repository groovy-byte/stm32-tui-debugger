use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
use super::state::PaneId;

pub enum Command {
    Quit,
    NextPane,
    PrevPane,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    Select,
    Back,
    ToggleExpand,
    AddExpression,
    RemoveExpression,
    HaltTarget,
    ResumeTarget,
    ResetTarget,
    StepOver,
    StepInto,
    RefreshTasks,
    InspectTask,
}

pub fn map_key(key: &KeyEvent, focused: PaneId) -> Option<Command> {
    // Global keys (work regardless of focused pane)
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Command::Quit),
        (KeyModifiers::NONE, KeyCode::Char('q')) => return Some(Command::Quit),
        (KeyModifiers::NONE, KeyCode::Tab) => return Some(Command::NextPane),
        (KeyModifiers::SHIFT, KeyCode::BackTab) => return Some(Command::PrevPane),
        (KeyModifiers::NONE, KeyCode::F(5)) => return Some(Command::ResumeTarget),
        (KeyModifiers::NONE, KeyCode::F(6)) => return Some(Command::HaltTarget),
        (KeyModifiers::NONE, KeyCode::F(7)) => return Some(Command::ResetTarget),
        (KeyModifiers::NONE, KeyCode::F(10)) => return Some(Command::StepOver),
        (KeyModifiers::NONE, KeyCode::F(11)) => return Some(Command::StepInto),
        _ => {}
    }

    // Navigation keys (context-sensitive)
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Command::ScrollUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Command::ScrollDown),
        KeyCode::PageUp => Some(Command::PageUp),
        KeyCode::PageDown => Some(Command::PageDown),
        KeyCode::Enter => match focused {
            PaneId::Tasks => Some(Command::InspectTask),
            _ => Some(Command::Select),
        },
        KeyCode::Esc => Some(Command::Back),
        KeyCode::Char(' ') if focused == PaneId::Peripherals => Some(Command::ToggleExpand),
        KeyCode::Char('a') if focused == PaneId::Expressions => Some(Command::AddExpression),
        KeyCode::Char('d') if focused == PaneId::Expressions => Some(Command::RemoveExpression),
        KeyCode::Char('r') if focused == PaneId::Tasks => Some(Command::RefreshTasks),
        KeyCode::Char('i') if focused == PaneId::Tasks => Some(Command::InspectTask),
        _ => None,
    }
}
