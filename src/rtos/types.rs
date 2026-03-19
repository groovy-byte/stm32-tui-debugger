/// Configurable TCB field offsets for different FreeRTOS builds.
#[derive(Clone, Debug)]
pub struct TcbLayout {
    pub top_of_stack: usize,
    pub state_list_item: usize,
    pub event_list_item: usize,
    pub priority: usize,
    pub stack_base: usize,
    pub task_name: usize,
    pub task_name_len: usize,
}

impl Default for TcbLayout {
    fn default() -> Self {
        // Standard Cortex-M FreeRTOS without MPU
        Self {
            top_of_stack: 0,
            state_list_item: 4,
            event_list_item: 24,
            priority: 44,
            stack_base: 48,
            task_name: 52,
            task_name_len: 16,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskState {
    Running,
    Ready,
    Blocked,
    Suspended,
    Deleted,
    Unknown,
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TaskState::Running => write!(f, "Running"),
            TaskState::Ready => write!(f, "Ready"),
            TaskState::Blocked => write!(f, "Blocked"),
            TaskState::Suspended => write!(f, "Suspended"),
            TaskState::Deleted => write!(f, "Deleted"),
            TaskState::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TaskInfo {
    pub name: String,
    pub tcb_address: u32,
    pub priority: u32,
    pub state: TaskState,
    pub stack_top: u32,
    pub stack_base: u32,
    pub stack_high_water: Option<u32>,
}

impl TaskInfo {
    /// Estimate stack usage percentage (requires stack_high_water).
    pub fn stack_usage_percent(&self) -> Option<u32> {
        // Stack grows downward on ARM: base is high address, top is current low address
        if self.stack_base == 0 {
            return None;
        }
        let total = self.stack_top.saturating_sub(self.stack_base);
        if total == 0 {
            return None;
        }
        let used = self.stack_top.saturating_sub(self.stack_high_water.unwrap_or(self.stack_top));
        Some((used * 100) / total)
    }
}

/// Result of an RTOS polling cycle.
#[derive(Clone, Debug)]
pub struct RtosSnapshot {
    pub current_task_addr: u32,
    pub total_tasks: u32,
    pub tasks: Vec<TaskInfo>,
    pub error: Option<String>,
}
