use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use stm32_tui_debugger::rtos::{RtosSnapshot, TaskInfo, TaskState, TcbLayout};
use stm32_tui_debugger::tui::keybindings::{map_key, Command};
use stm32_tui_debugger::tui::state::InputMode;
use stm32_tui_debugger::tui::state::{PaneId, TasksState};

// ── Helper ──────────────────────────────────────────────────────────────

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn make_task(name: &str, priority: u32, state: TaskState) -> TaskInfo {
    TaskInfo {
        name: name.to_string(),
        tcb_address: 0x2000_0100,
        priority,
        state,
        stack_top: 0x2000_1000,
        stack_base: 0x2000_0000,
        stack_high_water: None,
    }
}

// ── 1. TcbLayout defaults ──────────────────────────────────────────────

#[test]
fn should_default_tcb_layout_to_cortex_m() {
    let layout = TcbLayout::default();
    assert_eq!(layout.top_of_stack, 0);
    assert_eq!(layout.state_list_item, 4);
    assert_eq!(layout.event_list_item, 24);
    assert_eq!(layout.priority, 44);
    assert_eq!(layout.stack_base, 48);
    assert_eq!(layout.task_name, 52);
    assert_eq!(layout.task_name_len, 16);
}

#[test]
fn should_allow_custom_tcb_layout() {
    let layout = TcbLayout {
        top_of_stack: 0,
        state_list_item: 8,
        event_list_item: 28,
        priority: 48,
        stack_base: 52,
        task_name: 56,
        task_name_len: 32,
    };
    assert_eq!(layout.state_list_item, 8);
    assert_eq!(layout.task_name_len, 32);
}

// ── 2. TaskState display ───────────────────────────────────────────────

#[test]
fn should_display_running() {
    assert_eq!(format!("{}", TaskState::Running), "Running");
}

#[test]
fn should_display_ready() {
    assert_eq!(format!("{}", TaskState::Ready), "Ready");
}

#[test]
fn should_display_blocked() {
    assert_eq!(format!("{}", TaskState::Blocked), "Blocked");
}

#[test]
fn should_display_suspended() {
    assert_eq!(format!("{}", TaskState::Suspended), "Suspended");
}

#[test]
fn should_display_deleted() {
    assert_eq!(format!("{}", TaskState::Deleted), "Deleted");
}

#[test]
fn should_display_unknown() {
    assert_eq!(format!("{}", TaskState::Unknown), "Unknown");
}

// ── 3. TaskState equality ──────────────────────────────────────────────

#[test]
fn should_equal_same_task_state() {
    assert_eq!(TaskState::Running, TaskState::Running);
    assert_eq!(TaskState::Blocked, TaskState::Blocked);
}

#[test]
fn should_not_equal_different_task_state() {
    assert_ne!(TaskState::Running, TaskState::Blocked);
    assert_ne!(TaskState::Ready, TaskState::Suspended);
}

// ── 4. TaskInfo creation and stack usage ────────────────────────────────

#[test]
fn should_create_task_info_with_correct_fields() {
    let task = make_task("MainTask", 5, TaskState::Running);
    assert_eq!(task.name, "MainTask");
    assert_eq!(task.priority, 5);
    assert_eq!(task.state, TaskState::Running);
    assert_eq!(task.stack_top, 0x2000_1000);
    assert_eq!(task.stack_base, 0x2000_0000);
    assert!(task.stack_high_water.is_none());
}

#[test]
fn should_return_zero_percent_when_no_high_water() {
    // When stack_high_water is None, it defaults to stack_top → used = 0
    let task = make_task("Idle", 0, TaskState::Ready);
    assert_eq!(task.stack_usage_percent(), Some(0));
}

#[test]
fn should_return_none_when_stack_base_is_zero() {
    let task = TaskInfo {
        name: "NoStack".to_string(),
        tcb_address: 0x2000_0100,
        priority: 1,
        state: TaskState::Running,
        stack_top: 0x2000_1000,
        stack_base: 0,
        stack_high_water: None,
    };
    assert_eq!(task.stack_usage_percent(), None);
}

#[test]
fn should_return_none_when_total_stack_is_zero() {
    // stack_top == stack_base → total = 0
    let task = TaskInfo {
        name: "ZeroStack".to_string(),
        tcb_address: 0x2000_0100,
        priority: 1,
        state: TaskState::Running,
        stack_top: 0x2000_0000,
        stack_base: 0x2000_0000,
        stack_high_water: None,
    };
    assert_eq!(task.stack_usage_percent(), None);
}

#[test]
fn should_compute_fifty_percent_stack_usage() {
    // total = 0x2000_1000 - 0x2000_0000 = 0x1000 (4096)
    // high_water = 0x2000_0800 → used = 0x2000_1000 - 0x2000_0800 = 0x800 (2048)
    // percent = (2048 * 100) / 4096 = 50
    let task = TaskInfo {
        name: "Half".to_string(),
        tcb_address: 0x2000_0100,
        priority: 3,
        state: TaskState::Running,
        stack_top: 0x2000_1000,
        stack_base: 0x2000_0000,
        stack_high_water: Some(0x2000_0800),
    };
    assert_eq!(task.stack_usage_percent(), Some(50));
}

#[test]
fn should_compute_full_stack_usage() {
    // high_water at stack_base → used = total → 100%
    let task = TaskInfo {
        name: "Full".to_string(),
        tcb_address: 0x2000_0100,
        priority: 3,
        state: TaskState::Running,
        stack_top: 0x2000_1000,
        stack_base: 0x2000_0000,
        stack_high_water: Some(0x2000_0000),
    };
    assert_eq!(task.stack_usage_percent(), Some(100));
}

#[test]
fn should_compute_quarter_stack_usage() {
    // total = 0x1000 (4096), high_water = 0x2000_0C00
    // used = 0x2000_1000 - 0x2000_0C00 = 0x400 (1024)
    // percent = (1024 * 100) / 4096 = 25
    let task = TaskInfo {
        name: "Quarter".to_string(),
        tcb_address: 0x2000_0100,
        priority: 2,
        state: TaskState::Ready,
        stack_top: 0x2000_1000,
        stack_base: 0x2000_0000,
        stack_high_water: Some(0x2000_0C00),
    };
    assert_eq!(task.stack_usage_percent(), Some(25));
}

// ── 5. RtosSnapshot ────────────────────────────────────────────────────

#[test]
fn should_create_empty_snapshot() {
    let snap = RtosSnapshot {
        current_task_addr: 0,
        total_tasks: 0,
        tasks: Vec::new(),
        error: None,
    };
    assert_eq!(snap.tasks.len(), 0);
    assert!(snap.error.is_none());
    assert_eq!(snap.current_task_addr, 0);
    assert_eq!(snap.total_tasks, 0);
}

#[test]
fn should_create_snapshot_with_tasks() {
    let tasks = vec![
        make_task("Main", 5, TaskState::Running),
        make_task("Idle", 0, TaskState::Ready),
        make_task("Comm", 3, TaskState::Blocked),
    ];
    let snap = RtosSnapshot {
        current_task_addr: 0x2000_0100,
        total_tasks: 3,
        tasks,
        error: None,
    };
    assert_eq!(snap.tasks.len(), 3);
    assert_eq!(snap.total_tasks, 3);
    assert_eq!(snap.tasks[0].name, "Main");
    assert_eq!(snap.tasks[1].name, "Idle");
    assert_eq!(snap.tasks[2].name, "Comm");
}

#[test]
fn should_preserve_task_ordering_in_snapshot() {
    let mut tasks = vec![
        make_task("Low", 1, TaskState::Ready),
        make_task("High", 10, TaskState::Running),
        make_task("Mid", 5, TaskState::Blocked),
    ];
    // Sort by priority descending then name (same as TaskReader does)
    tasks.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.name.cmp(&b.name)));

    assert_eq!(tasks[0].name, "High");
    assert_eq!(tasks[1].name, "Mid");
    assert_eq!(tasks[2].name, "Low");
}

#[test]
fn should_create_snapshot_with_error() {
    let snap = RtosSnapshot {
        current_task_addr: 0,
        total_tasks: 0,
        tasks: Vec::new(),
        error: Some("pxCurrentTCB not found".to_string()),
    };
    assert!(snap.error.is_some());
    assert_eq!(snap.error.unwrap(), "pxCurrentTCB not found");
}

// ── 6. PaneId::Tasks integration ───────────────────────────────────────

#[test]
fn should_include_tasks_in_pane_all() {
    let all = PaneId::all();
    assert!(all.contains(&PaneId::Tasks));
}

#[test]
fn should_cycle_peripherals_next_to_tasks() {
    assert_eq!(PaneId::Peripherals.next(), PaneId::Tasks);
}

#[test]
fn should_cycle_tasks_next_to_expressions() {
    assert_eq!(PaneId::Tasks.next(), PaneId::Expressions);
}

#[test]
fn should_reverse_expressions_prev_to_tasks() {
    assert_eq!(PaneId::Expressions.prev(), PaneId::Tasks);
}

#[test]
fn should_reverse_tasks_prev_to_peripherals() {
    assert_eq!(PaneId::Tasks.prev(), PaneId::Peripherals);
}

// ── 7. TasksState defaults ─────────────────────────────────────────────

#[test]
fn should_default_rtos_detected_to_false() {
    let state = TasksState::new();
    assert!(!state.rtos_detected);
}

#[test]
fn should_default_tasks_to_empty() {
    let state = TasksState::new();
    assert!(state.tasks.is_empty());
}

#[test]
fn should_default_selected_to_none() {
    let state = TasksState::new();
    assert!(state.selected.is_none());
}

#[test]
fn should_default_scroll_offset_to_zero() {
    let state = TasksState::new();
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn should_implement_default_trait_for_tasks_state() {
    let state = TasksState::default();
    assert!(!state.rtos_detected);
    assert!(state.tasks.is_empty());
    assert!(state.selected.is_none());
}

// ── 8. Keybindings for Tasks pane ──────────────────────────────────────

#[test]
fn should_map_r_to_refresh_tasks_in_tasks_pane() {
    let cmd = map_key(&key(KeyCode::Char('r')), PaneId::Tasks, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::RefreshTasks)));
}

#[test]
fn should_map_i_to_inspect_task_in_tasks_pane() {
    let cmd = map_key(&key(KeyCode::Char('i')), PaneId::Tasks, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::InspectTask)));
}

#[test]
fn should_map_enter_to_inspect_task_in_tasks_pane() {
    let cmd = map_key(&key(KeyCode::Enter), PaneId::Tasks, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::InspectTask)));
}

#[test]
fn should_not_map_r_to_refresh_in_source_pane() {
    let cmd = map_key(&key(KeyCode::Char('r')), PaneId::Source, InputMode::Normal, false);
    assert!(cmd.is_none());
}

#[test]
fn should_not_map_r_to_refresh_in_console_pane() {
    let cmd = map_key(&key(KeyCode::Char('r')), PaneId::Console, InputMode::Normal, false);
    assert!(cmd.is_none());
}

#[test]
fn should_not_map_i_outside_tasks_pane() {
    let cmd = map_key(&key(KeyCode::Char('i')), PaneId::Source, InputMode::Normal, false);
    assert!(cmd.is_none());
}

#[test]
fn should_map_enter_to_select_in_non_tasks_panes() {
    for pane in &[PaneId::Source, PaneId::Peripherals, PaneId::Expressions, PaneId::Console] {
        let cmd = map_key(&key(KeyCode::Enter), *pane, InputMode::Normal, false);
        assert!(
            matches!(cmd, Some(Command::Select)),
            "Enter should be Select in {pane:?}"
        );
    }
}

#[test]
fn should_map_global_keys_from_tasks_pane() {
    // Global keys must still work when Tasks pane is focused
    let quit = map_key(&key(KeyCode::Char('q')), PaneId::Tasks, InputMode::Normal, false);
    assert!(matches!(quit, Some(Command::Quit)));

    let tab = map_key(&key(KeyCode::Tab), PaneId::Tasks, InputMode::Normal, false);
    assert!(matches!(tab, Some(Command::NextPane)));

    let resume = map_key(&key(KeyCode::F(5)), PaneId::Tasks, InputMode::Normal, false);
    assert!(matches!(resume, Some(Command::ResumeTarget)));
}
