use std::collections::VecDeque;

use crate::tui::widgets::{CompletionList, TextInput};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaneId {
    Source,
    Peripherals,
    Tasks,
    Expressions,
    Console,
}

impl PaneId {
    pub fn all() -> &'static [PaneId] {
        &[
            PaneId::Source,
            PaneId::Peripherals,
            PaneId::Tasks,
            PaneId::Expressions,
            PaneId::Console,
        ]
    }

    pub fn next(&self) -> PaneId {
        match self {
            PaneId::Source => PaneId::Peripherals,
            PaneId::Peripherals => PaneId::Tasks,
            PaneId::Tasks => PaneId::Expressions,
            PaneId::Expressions => PaneId::Console,
            PaneId::Console => PaneId::Source,
        }
    }

    pub fn prev(&self) -> PaneId {
        match self {
            PaneId::Source => PaneId::Console,
            PaneId::Peripherals => PaneId::Source,
            PaneId::Tasks => PaneId::Peripherals,
            PaneId::Expressions => PaneId::Tasks,
            PaneId::Console => PaneId::Expressions,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    InputExpression,
    InputCommand,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Normal
    }
}

pub struct TuiState {
    pub focused: PaneId,
    pub source_state: SourceState,
    pub peripheral_state: PeripheralState,
    pub tasks_state: TasksState,
    pub expression_state: ExpressionState,
    pub console_state: ConsoleState,
    pub status_message: Option<String>,
    pub running: bool,
    pub input_mode: InputMode,
    pub input: TextInput,
    pub completion: CompletionList,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            focused: PaneId::Source,
            source_state: SourceState::new(),
            peripheral_state: PeripheralState::new(),
            tasks_state: TasksState::new(),
            expression_state: ExpressionState::new(),
            console_state: ConsoleState::new(),
            status_message: None,
            running: true,
            input_mode: InputMode::Normal,
            input: TextInput::new(),
            completion: CompletionList::new(),
        }
    }
}

impl Default for TuiState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SourceState {
    pub file_path: Option<String>,
    pub lines: Vec<String>,
    pub scroll_offset: usize,
    pub current_line: Option<usize>,
}

impl SourceState {
    pub fn new() -> Self {
        Self {
            file_path: None,
            lines: Vec::new(),
            scroll_offset: 0,
            current_line: None,
        }
    }
}

impl Default for SourceState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PeripheralState {
    pub peripherals: Vec<String>,
    pub selected_peripheral: Option<usize>,
    pub expanded_registers: Vec<String>,
    pub scroll_offset: usize,
}

impl PeripheralState {
    pub fn new() -> Self {
        Self {
            peripherals: Vec::new(),
            selected_peripheral: None,
            expanded_registers: Vec::new(),
            scroll_offset: 0,
        }
    }
}

impl Default for PeripheralState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ExpressionState {
    pub entries: Vec<ExpressionEntry>,
    pub selected: Option<usize>,
    pub scroll_offset: usize,
}

impl ExpressionState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            selected: None,
            scroll_offset: 0,
        }
    }
}

impl Default for ExpressionState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ExpressionEntry {
    pub name: String,
    pub value: String,
    pub changed: bool,
}

pub struct ConsoleState {
    pub lines: VecDeque<String>,
    pub max_lines: usize,
    pub scroll_offset: usize,
    pub auto_scroll: bool,
}

impl ConsoleState {
    pub fn new() -> Self {
        Self {
            lines: VecDeque::new(),
            max_lines: 1000,
            scroll_offset: 0,
            auto_scroll: true,
        }
    }
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TasksState {
    pub tasks: Vec<TaskEntry>,
    pub selected: Option<usize>,
    pub scroll_offset: usize,
    pub rtos_detected: bool,
}

impl TasksState {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            selected: None,
            scroll_offset: 0,
            rtos_detected: false,
        }
    }
}

impl Default for TasksState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TaskEntry {
    pub name: String,
    pub tcb_address: u32,
    pub priority: u32,
    pub state: String,
    pub stack_top: u32,
    pub stack_base: u32,
}
