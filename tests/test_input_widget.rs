// Comprehensive tests for Phase 1: TextInput widget, InputMode, and keybinding routing.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use stm32_tui_debugger::tui::keybindings::{map_key, Command};
use stm32_tui_debugger::tui::state::{InputMode, PaneId};
use stm32_tui_debugger::tui::widgets::TextInput;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn shift_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::SHIFT)
}

fn ctrl_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

// ============================================================================
// Section 1: TextInput construction
// ============================================================================

#[test]
fn new_creates_empty_buffer() {
    let ti = TextInput::new();
    assert!(ti.is_empty());
    assert_eq!(ti.content(), "");
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn default_creates_same_as_new() {
    let ti = TextInput::default();
    assert!(ti.is_empty());
    assert_eq!(ti.content(), "");
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn default_prompt_is_angle_bracket() {
    let ti = TextInput::new();
    assert_eq!(ti.prompt, "> ");
}

#[test]
fn with_prompt_sets_custom_prompt() {
    let ti = TextInput::with_prompt(">> ");
    assert_eq!(ti.prompt, ">> ");
    assert!(ti.is_empty());
}

#[test]
fn is_empty_false_after_insert() {
    let mut ti = TextInput::new();
    ti.insert_char('x');
    assert!(!ti.is_empty());
}

// ============================================================================
// Section 2: Character insertion & cursor movement
// ============================================================================

#[test]
fn insert_single_char_at_beginning() {
    let mut ti = TextInput::new();
    ti.insert_char('a');
    assert_eq!(ti.content(), "a");
    assert_eq!(ti.cursor_position(), 1);
}

#[test]
fn insert_multiple_chars_sequential() {
    let mut ti = TextInput::new();
    for c in "hello".chars() {
        ti.insert_char(c);
    }
    assert_eq!(ti.content(), "hello");
    assert_eq!(ti.cursor_position(), 5);
}

#[test]
fn insert_char_at_end() {
    let mut ti = TextInput::new();
    ti.insert_char('a');
    ti.insert_char('b');
    ti.insert_char('c');
    assert_eq!(ti.content(), "abc");
    assert_eq!(ti.cursor_position(), 3);
}

#[test]
fn insert_char_at_middle() {
    let mut ti = TextInput::new();
    ti.insert_char('a');
    ti.insert_char('c');
    // cursor is at 2, move left so cursor is at 1
    ti.move_left();
    ti.insert_char('b');
    assert_eq!(ti.content(), "abc");
    assert_eq!(ti.cursor_position(), 2);
}

#[test]
fn insert_char_at_beginning_via_home() {
    let mut ti = TextInput::new();
    ti.insert_char('b');
    ti.insert_char('c');
    ti.home();
    ti.insert_char('a');
    assert_eq!(ti.content(), "abc");
    assert_eq!(ti.cursor_position(), 1);
}

#[test]
fn cursor_advances_after_each_insert() {
    let mut ti = TextInput::new();
    assert_eq!(ti.cursor_position(), 0);
    ti.insert_char('x');
    assert_eq!(ti.cursor_position(), 1);
    ti.insert_char('y');
    assert_eq!(ti.cursor_position(), 2);
    ti.insert_char('z');
    assert_eq!(ti.cursor_position(), 3);
}

#[test]
fn move_left_decrements_cursor() {
    let mut ti = TextInput::new();
    ti.insert_char('a');
    ti.insert_char('b');
    ti.move_left();
    assert_eq!(ti.cursor_position(), 1);
}

#[test]
fn move_left_at_zero_is_noop() {
    let mut ti = TextInput::new();
    ti.move_left(); // already at 0
    assert_eq!(ti.cursor_position(), 0);
    ti.insert_char('a');
    ti.home();
    ti.move_left();
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn move_right_increments_cursor() {
    let mut ti = TextInput::new();
    ti.insert_char('a');
    ti.insert_char('b');
    ti.home();
    ti.move_right();
    assert_eq!(ti.cursor_position(), 1);
}

#[test]
fn move_right_at_end_is_noop() {
    let mut ti = TextInput::new();
    ti.insert_char('a');
    // cursor at 1 = len, move_right should not exceed
    ti.move_right();
    assert_eq!(ti.cursor_position(), 1);
}

#[test]
fn home_moves_to_zero() {
    let mut ti = TextInput::new();
    for c in "hello".chars() {
        ti.insert_char(c);
    }
    assert_eq!(ti.cursor_position(), 5);
    ti.home();
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn end_moves_to_buffer_len() {
    let mut ti = TextInput::new();
    for c in "hello".chars() {
        ti.insert_char(c);
    }
    ti.home();
    assert_eq!(ti.cursor_position(), 0);
    ti.end();
    assert_eq!(ti.cursor_position(), 5);
}

#[test]
fn move_left_right_round_trip() {
    let mut ti = TextInput::new();
    for c in "abc".chars() {
        ti.insert_char(c);
    }
    // cursor at 3
    ti.move_left();
    ti.move_left();
    ti.move_right();
    assert_eq!(ti.cursor_position(), 2);
}

#[test]
fn rapid_inserts_build_correct_string() {
    let mut ti = TextInput::new();
    let text = "The quick brown fox jumps!";
    for c in text.chars() {
        ti.insert_char(c);
    }
    assert_eq!(ti.content(), text);
    assert_eq!(ti.cursor_position(), text.len());
}

#[test]
fn insert_digits_and_symbols() {
    let mut ti = TextInput::new();
    for c in "0x1234_ABCD".chars() {
        ti.insert_char(c);
    }
    assert_eq!(ti.content(), "0x1234_ABCD");
}

// ============================================================================
// Section 3: Deletion
// ============================================================================

#[test]
fn backspace_at_start_is_noop() {
    let mut ti = TextInput::new();
    ti.backspace();
    assert_eq!(ti.content(), "");
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn backspace_removes_char_before_cursor() {
    let mut ti = TextInput::new();
    ti.insert_char('a');
    ti.insert_char('b');
    ti.insert_char('c');
    ti.backspace();
    assert_eq!(ti.content(), "ab");
    assert_eq!(ti.cursor_position(), 2);
}

#[test]
fn backspace_in_middle() {
    let mut ti = TextInput::new();
    for c in "abc".chars() {
        ti.insert_char(c);
    }
    ti.move_left(); // cursor at 2
    ti.backspace(); // removes 'b', cursor at 1
    assert_eq!(ti.content(), "ac");
    assert_eq!(ti.cursor_position(), 1);
}

#[test]
fn delete_char_at_end_is_noop() {
    let mut ti = TextInput::new();
    ti.insert_char('a');
    ti.delete_char(); // cursor at end, no-op
    assert_eq!(ti.content(), "a");
    assert_eq!(ti.cursor_position(), 1);
}

#[test]
fn delete_char_removes_at_cursor() {
    let mut ti = TextInput::new();
    for c in "abc".chars() {
        ti.insert_char(c);
    }
    ti.home();
    ti.delete_char(); // removes 'a', cursor stays at 0
    assert_eq!(ti.content(), "bc");
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn delete_char_cursor_stays() {
    let mut ti = TextInput::new();
    for c in "abc".chars() {
        ti.insert_char(c);
    }
    ti.home();
    ti.move_right(); // cursor at 1
    ti.delete_char(); // removes 'b'
    assert_eq!(ti.content(), "ac");
    assert_eq!(ti.cursor_position(), 1);
}

#[test]
fn backspace_single_char_empties() {
    let mut ti = TextInput::new();
    ti.insert_char('x');
    ti.backspace();
    assert!(ti.is_empty());
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn delete_all_chars_one_by_one() {
    let mut ti = TextInput::new();
    for c in "abcd".chars() {
        ti.insert_char(c);
    }
    ti.home();
    ti.delete_char(); // remove 'a'
    ti.delete_char(); // remove 'b'
    ti.delete_char(); // remove 'c'
    ti.delete_char(); // remove 'd'
    assert!(ti.is_empty());
    assert_eq!(ti.cursor_position(), 0);
}

// ============================================================================
// Section 4: Submit & Cancel
// ============================================================================

#[test]
fn submit_returns_buffer_content() {
    let mut ti = TextInput::new();
    for c in "hello".chars() {
        ti.insert_char(c);
    }
    let result = ti.submit();
    assert_eq!(result, "hello");
}

#[test]
fn submit_clears_buffer_and_cursor() {
    let mut ti = TextInput::new();
    for c in "hello".chars() {
        ti.insert_char(c);
    }
    let _ = ti.submit();
    assert!(ti.is_empty());
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn submit_empty_returns_empty_no_history() {
    let mut ti = TextInput::new();
    let result = ti.submit();
    assert_eq!(result, "");
    // history should still be empty — history_up should be a no-op
    ti.history_up();
    assert!(ti.is_empty());
}

#[test]
fn submit_adds_nonempty_to_history() {
    let mut ti = TextInput::new();
    for c in "cmd1".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    // Now up-arrow should recall "cmd1"
    ti.history_up();
    assert_eq!(ti.content(), "cmd1");
}

#[test]
fn cancel_clears_everything() {
    let mut ti = TextInput::new();
    // Add some history first
    for c in "prev".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    // Type something and start navigating history
    for c in "curr".chars() {
        ti.insert_char(c);
    }
    ti.history_up(); // saves "curr" as saved_buffer, loads "prev"
    // Now cancel
    ti.cancel();
    assert!(ti.is_empty());
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn clear_clears_buffer_not_history() {
    let mut ti = TextInput::new();
    for c in "cmd1".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "typing".chars() {
        ti.insert_char(c);
    }
    ti.clear();
    assert!(ti.is_empty());
    assert_eq!(ti.cursor_position(), 0);
    // History should still be intact
    ti.history_up();
    assert_eq!(ti.content(), "cmd1");
}

// ============================================================================
// Section 5: History navigation
// ============================================================================

#[test]
fn history_up_no_history_is_noop() {
    let mut ti = TextInput::new();
    ti.history_up();
    assert!(ti.is_empty());
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn history_up_saves_buffer_loads_last() {
    let mut ti = TextInput::new();
    for c in "first".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "current".chars() {
        ti.insert_char(c);
    }
    ti.history_up();
    assert_eq!(ti.content(), "first");
}

#[test]
fn history_up_cursor_at_end() {
    let mut ti = TextInput::new();
    for c in "abc".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    ti.history_up();
    assert_eq!(ti.cursor_position(), 3); // len of "abc"
}

#[test]
fn history_up_multiple_walks_backward() {
    let mut ti = TextInput::new();
    for c in "first".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "second".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "third".chars() {
        ti.insert_char(c);
    }
    ti.submit();

    ti.history_up();
    assert_eq!(ti.content(), "third");
    ti.history_up();
    assert_eq!(ti.content(), "second");
    ti.history_up();
    assert_eq!(ti.content(), "first");
}

#[test]
fn history_up_at_oldest_stays() {
    let mut ti = TextInput::new();
    for c in "only".chars() {
        ti.insert_char(c);
    }
    ti.submit();

    ti.history_up();
    assert_eq!(ti.content(), "only");
    ti.history_up(); // should stay at "only"
    assert_eq!(ti.content(), "only");
}

#[test]
fn history_down_back_to_saved_buffer() {
    let mut ti = TextInput::new();
    for c in "hist".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "current".chars() {
        ti.insert_char(c);
    }
    ti.history_up(); // saves "current", loads "hist"
    assert_eq!(ti.content(), "hist");
    ti.history_down(); // back to saved "current"
    assert_eq!(ti.content(), "current");
}

#[test]
fn history_down_no_index_is_noop() {
    let mut ti = TextInput::new();
    ti.history_down();
    assert!(ti.is_empty());

    for c in "test".chars() {
        ti.insert_char(c);
    }
    ti.history_down(); // no history_index, should be noop
    assert_eq!(ti.content(), "test");
}

#[test]
fn full_history_cycle() {
    let mut ti = TextInput::new();
    // Submit two commands
    for c in "cmd1".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "cmd2".chars() {
        ti.insert_char(c);
    }
    ti.submit();

    // Type partial
    for c in "partial".chars() {
        ti.insert_char(c);
    }

    // Navigate up through history
    ti.history_up(); // "cmd2"
    assert_eq!(ti.content(), "cmd2");
    ti.history_up(); // "cmd1"
    assert_eq!(ti.content(), "cmd1");
    ti.history_up(); // stays at "cmd1"
    assert_eq!(ti.content(), "cmd1");

    // Navigate down
    ti.history_down(); // "cmd2"
    assert_eq!(ti.content(), "cmd2");
    ti.history_down(); // back to "partial"
    assert_eq!(ti.content(), "partial");
    ti.history_down(); // no-op, already at bottom
    assert_eq!(ti.content(), "partial");
}

#[test]
fn submit_resets_history_index() {
    let mut ti = TextInput::new();
    for c in "first".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "second".chars() {
        ti.insert_char(c);
    }
    ti.submit();

    // Navigate into history
    ti.history_up();
    assert_eq!(ti.content(), "second");

    // Type something new and submit from history
    ti.clear();
    for c in "third".chars() {
        ti.insert_char(c);
    }
    ti.submit();

    // Now history_up should go to "third" (most recent)
    ti.history_up();
    assert_eq!(ti.content(), "third");
}

#[test]
fn cancel_resets_history_index() {
    let mut ti = TextInput::new();
    for c in "cmd".chars() {
        ti.insert_char(c);
    }
    ti.submit();

    ti.history_up();
    assert_eq!(ti.content(), "cmd");
    ti.cancel();

    // After cancel, history_up should restart from end
    ti.history_up();
    assert_eq!(ti.content(), "cmd");
}

#[test]
fn history_preserves_across_multiple_submits() {
    let mut ti = TextInput::new();
    for i in 0..5 {
        let s = format!("entry{}", i);
        for c in s.chars() {
            ti.insert_char(c);
        }
        ti.submit();
    }
    // Walk all the way up
    ti.history_up();
    assert_eq!(ti.content(), "entry4");
    ti.history_up();
    assert_eq!(ti.content(), "entry3");
    ti.history_up();
    assert_eq!(ti.content(), "entry2");
    ti.history_up();
    assert_eq!(ti.content(), "entry1");
    ti.history_up();
    assert_eq!(ti.content(), "entry0");
    ti.history_up(); // stays
    assert_eq!(ti.content(), "entry0");
}

#[test]
fn history_down_cursor_at_end_of_entry() {
    let mut ti = TextInput::new();
    for c in "short".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "longer_entry".chars() {
        ti.insert_char(c);
    }
    ti.submit();

    ti.history_up(); // "longer_entry"
    assert_eq!(ti.cursor_position(), 12);
    ti.history_up(); // "short"
    assert_eq!(ti.cursor_position(), 5);
    ti.history_down(); // "longer_entry"
    assert_eq!(ti.cursor_position(), 12);
}

// ============================================================================
// Section 6: Keybinding routing in input mode
// ============================================================================

#[test]
fn input_mode_char_maps_to_input_char() {
    let cmd = map_key(&key(KeyCode::Char('a')), PaneId::Source, InputMode::InputExpression);
    assert!(matches!(cmd, Some(Command::InputChar('a'))));
}

#[test]
fn input_mode_q_does_not_quit() {
    let cmd = map_key(&key(KeyCode::Char('q')), PaneId::Source, InputMode::InputExpression);
    assert!(matches!(cmd, Some(Command::InputChar('q'))));
}

#[test]
fn input_mode_esc_maps_to_cancel() {
    let cmd = map_key(&key(KeyCode::Esc), PaneId::Source, InputMode::InputExpression);
    assert!(matches!(cmd, Some(Command::InputCancel)));
}

#[test]
fn input_mode_enter_maps_to_submit() {
    let cmd = map_key(&key(KeyCode::Enter), PaneId::Source, InputMode::InputExpression);
    assert!(matches!(cmd, Some(Command::InputSubmit)));
}

#[test]
fn input_mode_backspace_maps_to_input_backspace() {
    let cmd = map_key(
        &key(KeyCode::Backspace),
        PaneId::Source,
        InputMode::InputExpression,
    );
    assert!(matches!(cmd, Some(Command::InputBackspace)));
}

#[test]
fn input_mode_delete_maps_to_input_delete() {
    let cmd = map_key(
        &key(KeyCode::Delete),
        PaneId::Source,
        InputMode::InputExpression,
    );
    assert!(matches!(cmd, Some(Command::InputDelete)));
}

#[test]
fn input_mode_left_right() {
    let left = map_key(&key(KeyCode::Left), PaneId::Source, InputMode::InputExpression);
    let right = map_key(&key(KeyCode::Right), PaneId::Source, InputMode::InputExpression);
    assert!(matches!(left, Some(Command::InputLeft)));
    assert!(matches!(right, Some(Command::InputRight)));
}

#[test]
fn input_mode_home_end() {
    let home = map_key(&key(KeyCode::Home), PaneId::Source, InputMode::InputExpression);
    let end = map_key(&key(KeyCode::End), PaneId::Source, InputMode::InputExpression);
    assert!(matches!(home, Some(Command::InputHome)));
    assert!(matches!(end, Some(Command::InputEnd)));
}

#[test]
fn input_mode_history_up_down() {
    let up = map_key(&key(KeyCode::Up), PaneId::Source, InputMode::InputExpression);
    let down = map_key(&key(KeyCode::Down), PaneId::Source, InputMode::InputExpression);
    assert!(matches!(up, Some(Command::InputHistoryUp)));
    assert!(matches!(down, Some(Command::InputHistoryDown)));
}

#[test]
fn input_mode_f5_returns_none() {
    // In input mode, F5 has no mapping in map_input_key
    let cmd = map_key(&key(KeyCode::F(5)), PaneId::Source, InputMode::InputExpression);
    assert!(cmd.is_none());
}

#[test]
fn input_mode_ctrl_c_does_not_quit() {
    // In input mode, Ctrl+C should NOT produce Quit — goes through map_input_key
    let cmd = map_key(
        &ctrl_key(KeyCode::Char('c')),
        PaneId::Source,
        InputMode::InputExpression,
    );
    // map_input_key matches on key.code; Char('c') → InputChar('c')
    assert!(matches!(cmd, Some(Command::InputChar('c'))));
}

#[test]
fn input_mode_tab_returns_none() {
    let cmd = map_key(&key(KeyCode::Tab), PaneId::Source, InputMode::InputExpression);
    assert!(cmd.is_none());
}

#[test]
fn input_command_mode_same_routing() {
    let cmd = map_key(&key(KeyCode::Char('x')), PaneId::Console, InputMode::InputCommand);
    assert!(matches!(cmd, Some(Command::InputChar('x'))));

    let cmd = map_key(&key(KeyCode::Enter), PaneId::Console, InputMode::InputCommand);
    assert!(matches!(cmd, Some(Command::InputSubmit)));

    let cmd = map_key(&key(KeyCode::Esc), PaneId::Console, InputMode::InputCommand);
    assert!(matches!(cmd, Some(Command::InputCancel)));
}

#[test]
fn normal_mode_q_quits() {
    let cmd = map_key(&key(KeyCode::Char('q')), PaneId::Source, InputMode::Normal);
    assert!(matches!(cmd, Some(Command::Quit)));
}

#[test]
fn normal_mode_tab_next_pane() {
    let cmd = map_key(&key(KeyCode::Tab), PaneId::Source, InputMode::Normal);
    assert!(matches!(cmd, Some(Command::NextPane)));
}

#[test]
fn normal_mode_f5_resumes() {
    let cmd = map_key(&key(KeyCode::F(5)), PaneId::Source, InputMode::Normal);
    assert!(matches!(cmd, Some(Command::ResumeTarget)));
}

#[test]
fn normal_mode_ctrl_c_quits() {
    let cmd = map_key(&ctrl_key(KeyCode::Char('c')), PaneId::Source, InputMode::Normal);
    assert!(matches!(cmd, Some(Command::Quit)));
}

#[test]
fn normal_mode_j_scrolls_down() {
    let cmd = map_key(&key(KeyCode::Char('j')), PaneId::Source, InputMode::Normal);
    assert!(matches!(cmd, Some(Command::ScrollDown)));
}

#[test]
fn normal_mode_a_add_expression_in_expressions_pane() {
    let cmd = map_key(
        &key(KeyCode::Char('a')),
        PaneId::Expressions,
        InputMode::Normal,
    );
    assert!(matches!(cmd, Some(Command::AddExpression)));
}

// ============================================================================
// Section 7: Edge cases
// ============================================================================

#[test]
fn insert_then_backspace_is_empty() {
    let mut ti = TextInput::new();
    ti.insert_char('z');
    assert!(!ti.is_empty());
    ti.backspace();
    assert!(ti.is_empty());
}

#[test]
fn very_long_string() {
    let mut ti = TextInput::new();
    for _ in 0..1000 {
        ti.insert_char('a');
    }
    assert_eq!(ti.content().len(), 1000);
    assert_eq!(ti.cursor_position(), 1000);
    // Backspace all
    for _ in 0..1000 {
        ti.backspace();
    }
    assert!(ti.is_empty());
}

#[test]
fn with_prompt_does_not_affect_buffer() {
    let mut ti = TextInput::with_prompt("$ ");
    ti.insert_char('a');
    assert_eq!(ti.content(), "a");
    assert_eq!(ti.cursor_position(), 1);
    let result = ti.submit();
    assert_eq!(result, "a");
    assert_eq!(ti.prompt, "$ ");
}

#[test]
fn submit_preserves_history_across_sessions() {
    let mut ti = TextInput::new();
    for c in "alpha".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "beta".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "gamma".chars() {
        ti.insert_char(c);
    }
    ti.submit();

    // Recall in reverse
    ti.history_up();
    assert_eq!(ti.content(), "gamma");
    ti.history_up();
    assert_eq!(ti.content(), "beta");
    ti.history_up();
    assert_eq!(ti.content(), "alpha");

    // Submit from history and verify old + new all accessible
    ti.submit(); // submits "alpha" again
    ti.history_up();
    assert_eq!(ti.content(), "alpha"); // most recently submitted
    ti.history_up();
    assert_eq!(ti.content(), "gamma");
}

#[test]
fn empty_buffer_operations_stable() {
    let mut ti = TextInput::new();
    // All no-ops on empty
    ti.backspace();
    ti.delete_char();
    ti.move_left();
    ti.move_right();
    ti.home();
    ti.end();
    assert!(ti.is_empty());
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn input_mode_default_is_normal() {
    let mode = InputMode::default();
    assert_eq!(mode, InputMode::Normal);
}

#[test]
fn input_mode_clone_copy() {
    let mode = InputMode::InputExpression;
    let cloned = mode;
    assert_eq!(cloned, InputMode::InputExpression);
    let copied: InputMode = mode;
    assert_eq!(copied, InputMode::InputExpression);
}

#[test]
fn backspace_multiple_from_end() {
    let mut ti = TextInput::new();
    for c in "abcde".chars() {
        ti.insert_char(c);
    }
    ti.backspace(); // "abcd"
    ti.backspace(); // "abc"
    assert_eq!(ti.content(), "abc");
    assert_eq!(ti.cursor_position(), 3);
}

#[test]
fn move_left_right_boundaries_stress() {
    let mut ti = TextInput::new();
    for c in "ab".chars() {
        ti.insert_char(c);
    }
    // cursor at 2
    ti.move_right(); // noop, still 2
    ti.move_right(); // noop, still 2
    assert_eq!(ti.cursor_position(), 2);
    ti.move_left(); // 1
    ti.move_left(); // 0
    ti.move_left(); // noop, still 0
    ti.move_left(); // noop, still 0
    assert_eq!(ti.cursor_position(), 0);
}

#[test]
fn history_saved_buffer_restored_after_cancel_and_renavigation() {
    let mut ti = TextInput::new();
    for c in "h1".chars() {
        ti.insert_char(c);
    }
    ti.submit();
    for c in "h2".chars() {
        ti.insert_char(c);
    }
    ti.submit();

    // Type something, navigate history, cancel
    for c in "wip".chars() {
        ti.insert_char(c);
    }
    ti.history_up();
    ti.cancel(); // clears buffer, resets history_index and saved_buffer

    // After cancel, history is still intact
    ti.history_up();
    assert_eq!(ti.content(), "h2");
    ti.history_down();
    // saved_buffer was cleared by cancel, so should be ""
    assert_eq!(ti.content(), "");
}

#[test]
fn normal_mode_shift_backtab_prev_pane() {
    let cmd = map_key(
        &shift_key(KeyCode::BackTab),
        PaneId::Source,
        InputMode::Normal,
    );
    assert!(matches!(cmd, Some(Command::PrevPane)));
}
