use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use stm32_tui_debugger::tui::keybindings::{map_key, Command};
use stm32_tui_debugger::tui::state::{ConsoleState, InputMode, PaneId, TuiState};

// ── PaneId::next() ──────────────────────────────────────────────────────

#[test]
fn should_cycle_source_to_peripherals() {
    assert_eq!(PaneId::Source.next(), PaneId::Peripherals);
}

#[test]
fn should_cycle_peripherals_to_tasks() {
    assert_eq!(PaneId::Peripherals.next(), PaneId::Tasks);
}

#[test]
fn should_cycle_tasks_to_expressions() {
    assert_eq!(PaneId::Tasks.next(), PaneId::Expressions);
}

#[test]
fn should_cycle_expressions_to_console() {
    assert_eq!(PaneId::Expressions.next(), PaneId::Console);
}

#[test]
fn should_cycle_console_to_source() {
    assert_eq!(PaneId::Console.next(), PaneId::Source);
}

#[test]
fn should_complete_full_forward_cycle() {
    let start = PaneId::Source;
    let result = start.next().next().next().next().next();
    assert_eq!(result, start);
}

// ── PaneId::prev() ──────────────────────────────────────────────────────

#[test]
fn should_reverse_source_to_console() {
    assert_eq!(PaneId::Source.prev(), PaneId::Console);
}

#[test]
fn should_reverse_peripherals_to_source() {
    assert_eq!(PaneId::Peripherals.prev(), PaneId::Source);
}

#[test]
fn should_reverse_expressions_to_tasks() {
    assert_eq!(PaneId::Expressions.prev(), PaneId::Tasks);
}

#[test]
fn should_reverse_tasks_to_peripherals() {
    assert_eq!(PaneId::Tasks.prev(), PaneId::Peripherals);
}

#[test]
fn should_reverse_console_to_expressions() {
    assert_eq!(PaneId::Console.prev(), PaneId::Expressions);
}

#[test]
fn should_complete_full_reverse_cycle() {
    let start = PaneId::Source;
    let result = start.prev().prev().prev().prev().prev();
    assert_eq!(result, start);
}

// ── PaneId::next() and prev() are inverses ──────────────────────────────

#[test]
fn should_be_inverse_next_then_prev() {
    for pane in PaneId::all() {
        assert_eq!(pane.next().prev(), *pane);
    }
}

#[test]
fn should_be_inverse_prev_then_next() {
    for pane in PaneId::all() {
        assert_eq!(pane.prev().next(), *pane);
    }
}

// ── PaneId::all() ───────────────────────────────────────────────────────

#[test]
fn should_list_five_panes() {
    assert_eq!(PaneId::all().len(), 5);
}

// ── TuiState::new() ─────────────────────────────────────────────────────

#[test]
fn should_default_focus_to_source() {
    let state = TuiState::new();
    assert_eq!(state.focused, PaneId::Source);
}

#[test]
fn should_default_running_to_true() {
    let state = TuiState::new();
    assert!(state.running);
}

#[test]
fn should_default_status_message_to_none() {
    let state = TuiState::new();
    assert!(state.status_message.is_none());
}

#[test]
fn should_default_source_state_empty() {
    let state = TuiState::new();
    assert!(state.source_state.lines.is_empty());
    assert!(state.source_state.file_path.is_none());
    assert!(state.source_state.current_line.is_none());
    assert_eq!(state.source_state.scroll_offset, 0);
}

#[test]
fn should_default_peripheral_state_empty() {
    let state = TuiState::new();
    assert!(state.peripheral_state.peripherals.is_empty());
    assert!(state.peripheral_state.selected_peripheral.is_none());
}

#[test]
fn should_default_expression_state_empty() {
    let state = TuiState::new();
    assert!(state.expression_state.entries.is_empty());
    assert!(state.expression_state.selected.is_none());
}

// ── ConsoleState ────────────────────────────────────────────────────────

#[test]
fn should_default_console_max_lines_to_1000() {
    let cs = ConsoleState::new();
    assert_eq!(cs.max_lines, 1000);
}

#[test]
fn should_default_console_auto_scroll() {
    let cs = ConsoleState::new();
    assert!(cs.auto_scroll);
}

#[test]
fn should_default_console_empty_lines() {
    let cs = ConsoleState::new();
    assert!(cs.lines.is_empty());
}

#[test]
fn should_default_console_scroll_offset_zero() {
    let cs = ConsoleState::new();
    assert_eq!(cs.scroll_offset, 0);
}

// ── Keybinding: global keys ─────────────────────────────────────────────

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn key_with_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

#[test]
fn should_map_q_to_quit() {
    let cmd = map_key(&key(KeyCode::Char('q')), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::Quit)));
}

#[test]
fn should_map_ctrl_c_to_quit() {
    let cmd = map_key(
        &key_with_mod(KeyCode::Char('c'), KeyModifiers::CONTROL),
        PaneId::Source,
        InputMode::Normal,
        false,
    );
    assert!(matches!(cmd, Some(Command::Quit)));
}

#[test]
fn should_map_tab_to_next_pane() {
    let cmd = map_key(&key(KeyCode::Tab), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::NextPane)));
}

#[test]
fn should_map_shift_backtab_to_prev_pane() {
    let cmd = map_key(
        &key_with_mod(KeyCode::BackTab, KeyModifiers::SHIFT),
        PaneId::Source,
        InputMode::Normal,
        false,
    );
    assert!(matches!(cmd, Some(Command::PrevPane)));
}

#[test]
fn should_map_f5_to_resume_target() {
    let cmd = map_key(&key(KeyCode::F(5)), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::ResumeTarget)));
}

#[test]
fn should_map_f6_to_halt_target() {
    let cmd = map_key(&key(KeyCode::F(6)), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::HaltTarget)));
}

#[test]
fn should_map_f7_to_reset_target() {
    let cmd = map_key(&key(KeyCode::F(7)), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::ResetTarget)));
}

#[test]
fn should_map_f10_to_step_over() {
    let cmd = map_key(&key(KeyCode::F(10)), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::StepOver)));
}

#[test]
fn should_map_f11_to_step_into() {
    let cmd = map_key(&key(KeyCode::F(11)), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::StepInto)));
}

// ── Keybinding: navigation ──────────────────────────────────────────────

#[test]
fn should_map_up_to_scroll_up() {
    let cmd = map_key(&key(KeyCode::Up), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::ScrollUp)));
}

#[test]
fn should_map_k_to_scroll_up() {
    let cmd = map_key(&key(KeyCode::Char('k')), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::ScrollUp)));
}

#[test]
fn should_map_down_to_scroll_down() {
    let cmd = map_key(&key(KeyCode::Down), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::ScrollDown)));
}

#[test]
fn should_map_j_to_scroll_down() {
    let cmd = map_key(&key(KeyCode::Char('j')), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::ScrollDown)));
}

#[test]
fn should_map_page_up() {
    let cmd = map_key(&key(KeyCode::PageUp), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::PageUp)));
}

#[test]
fn should_map_page_down() {
    let cmd = map_key(&key(KeyCode::PageDown), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::PageDown)));
}

#[test]
fn should_map_enter_to_select() {
    let cmd = map_key(&key(KeyCode::Enter), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::Select)));
}

#[test]
fn should_map_esc_to_back() {
    let cmd = map_key(&key(KeyCode::Esc), PaneId::Source, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::Back)));
}

// ── Keybinding: context-sensitive ───────────────────────────────────────

#[test]
fn should_map_space_to_toggle_expand_in_peripherals() {
    let cmd = map_key(&key(KeyCode::Char(' ')), PaneId::Peripherals, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::ToggleExpand)));
}

#[test]
fn should_not_map_space_in_source_pane() {
    let cmd = map_key(&key(KeyCode::Char(' ')), PaneId::Source, InputMode::Normal, false);
    assert!(cmd.is_none());
}

#[test]
fn should_map_a_to_add_expression_in_expressions() {
    let cmd = map_key(&key(KeyCode::Char('a')), PaneId::Expressions, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::AddExpression)));
}

#[test]
fn should_map_d_to_remove_expression_in_expressions() {
    let cmd = map_key(&key(KeyCode::Char('d')), PaneId::Expressions, InputMode::Normal, false);
    assert!(matches!(cmd, Some(Command::RemoveExpression)));
}

#[test]
fn should_not_map_a_outside_expressions_pane() {
    let cmd = map_key(&key(KeyCode::Char('a')), PaneId::Source, InputMode::Normal, false);
    assert!(cmd.is_none());
}

#[test]
fn should_return_none_for_unmapped_key() {
    let cmd = map_key(&key(KeyCode::Char('z')), PaneId::Source, InputMode::Normal, false);
    assert!(cmd.is_none());
}

// ── Global keys work from any pane ──────────────────────────────────────

#[test]
fn should_map_quit_from_all_panes() {
    for pane in PaneId::all() {
        let cmd = map_key(&key(KeyCode::Char('q')), *pane, InputMode::Normal, false);
        assert!(matches!(cmd, Some(Command::Quit)), "q should quit in {pane:?}");
    }
}

#[test]
fn should_map_tab_from_all_panes() {
    for pane in PaneId::all() {
        let cmd = map_key(&key(KeyCode::Tab), *pane, InputMode::Normal, false);
        assert!(
            matches!(cmd, Some(Command::NextPane)),
            "Tab should move pane in {pane:?}"
        );
    }
}
