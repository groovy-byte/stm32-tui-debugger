use stm32_tui_debugger::symbols::elf_loader::ElfData;
use stm32_tui_debugger::symbols::types::{FunctionInfo, Variable};
use stm32_tui_debugger::tui::keybindings::{map_key, Command};
use stm32_tui_debugger::tui::state::{InputMode, PaneId};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// ─── Helpers ─────────────────────────────────────────────

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn var(name: &str, addr: u32, size: usize, type_name: &str, is_global: bool) -> Variable {
    Variable {
        name: name.into(),
        address: addr,
        size,
        type_name: type_name.into(),
        is_global,
    }
}

fn func(name: &str, addr: u32, size: u32) -> FunctionInfo {
    FunctionInfo {
        name: name.into(),
        address: addr,
        size,
        source: None,
    }
}

fn test_elf() -> ElfData {
    ElfData {
        raw_data: vec![],
        variables: vec![
            var("adc_value", 0x2000_0000, 4, "uint32_t", true),
            var("ADC_Buffer", 0x2000_0004, 8, "uint16_t[4]", true),
            var("motor_speed", 0x2000_0010, 2, "uint16_t", false),
            var("Motor_Dir", 0x2000_0012, 1, "uint8_t", true),
            var("temperature", 0x2000_0020, 4, "float", true),
            var("adc_calibration", 0x2000_0030, 4, "int32_t", false),
        ],
        functions: vec![
            func("HAL_GPIO_Init", 0x0800_0100, 64),
            func("HAL_GPIO_WritePin", 0x0800_0200, 32),
            func("hal_adc_start", 0x0800_0300, 48),
            func("main", 0x0800_0000, 256),
            func("SysTick_Handler", 0x0800_1000, 16),
        ],
        text_base: 0x0800_0000,
        data_base: 0x2000_0000,
    }
}

// ─── Variable Prefix Search ─────────────────────────────

#[test]
fn empty_prefix_returns_all_variables() {
    let elf = test_elf();
    let results = elf.find_variables_by_prefix("");
    assert_eq!(results.len(), elf.variables.len());
}

#[test]
fn prefix_matches_beginning_of_name() {
    let elf = test_elf();
    let results = elf.find_variables_by_prefix("motor_s");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "motor_speed");
}

#[test]
fn prefix_search_is_case_insensitive() {
    let elf = test_elf();
    // Lowercase prefix matches uppercase variable name
    let results = elf.find_variables_by_prefix("adc");
    let names: Vec<&str> = results.iter().map(|v| v.name.as_str()).collect();
    assert!(names.contains(&"ADC_Buffer"));
    assert!(names.contains(&"adc_value"));
    assert!(names.contains(&"adc_calibration"));
}

#[test]
fn prefix_search_uppercase_prefix_matches_lowercase() {
    let elf = test_elf();
    let results = elf.find_variables_by_prefix("MOTOR");
    let names: Vec<&str> = results.iter().map(|v| v.name.as_str()).collect();
    assert!(names.contains(&"motor_speed"));
    assert!(names.contains(&"Motor_Dir"));
}

#[test]
fn no_matches_returns_empty() {
    let elf = test_elf();
    let results = elf.find_variables_by_prefix("xyz_nonexistent");
    assert!(results.is_empty());
}

#[test]
fn results_sorted_alphabetically() {
    let elf = test_elf();
    let results = elf.find_variables_by_prefix("adc");
    // Sorted by String::cmp (ASCII: uppercase before lowercase)
    let names: Vec<&str> = results.iter().map(|v| v.name.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn multiple_matches_all_returned() {
    let elf = test_elf();
    let results = elf.find_variables_by_prefix("adc");
    assert_eq!(results.len(), 3); // adc_value, ADC_Buffer, adc_calibration
}

#[test]
fn single_char_prefix() {
    let elf = test_elf();
    let results = elf.find_variables_by_prefix("t");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "temperature");
}

#[test]
fn full_name_as_prefix_returns_exact() {
    let elf = test_elf();
    let results = elf.find_variables_by_prefix("temperature");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "temperature");
}

#[test]
fn prefix_longer_than_any_name_returns_empty() {
    let elf = test_elf();
    let results = elf.find_variables_by_prefix("temperature_extra_long");
    assert!(results.is_empty());
}

// ─── Function Prefix Search ────────────────────────────

#[test]
fn function_prefix_matches() {
    let elf = test_elf();
    let results = elf.find_functions_by_prefix("HAL_GPIO");
    assert_eq!(results.len(), 2);
}

#[test]
fn function_prefix_case_insensitive() {
    let elf = test_elf();
    let results = elf.find_functions_by_prefix("hal");
    let names: Vec<&str> = results.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"HAL_GPIO_Init"));
    assert!(names.contains(&"HAL_GPIO_WritePin"));
    assert!(names.contains(&"hal_adc_start"));
    assert_eq!(results.len(), 3);
}

#[test]
fn function_prefix_sorted() {
    let elf = test_elf();
    let results = elf.find_functions_by_prefix("hal");
    let names: Vec<&str> = results.iter().map(|f| f.name.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn function_prefix_no_matches() {
    let elf = test_elf();
    let results = elf.find_functions_by_prefix("nonexistent");
    assert!(results.is_empty());
}

#[test]
fn function_empty_prefix_returns_all() {
    let elf = test_elf();
    let results = elf.find_functions_by_prefix("");
    assert_eq!(results.len(), elf.functions.len());
}

#[test]
fn function_full_name_prefix() {
    let elf = test_elf();
    let results = elf.find_functions_by_prefix("main");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "main");
}

// ─── list_variables ─────────────────────────────────────

#[test]
fn list_variables_returns_all() {
    let elf = test_elf();
    let vars = elf.list_variables();
    assert_eq!(vars.len(), 6);
}

#[test]
fn list_variables_preserves_insertion_order() {
    let elf = test_elf();
    let vars = elf.list_variables();
    assert_eq!(vars[0].name, "adc_value");
    assert_eq!(vars[1].name, "ADC_Buffer");
    assert_eq!(vars[5].name, "adc_calibration");
}

#[test]
fn list_variables_empty_elf() {
    let elf = ElfData {
        raw_data: vec![],
        variables: vec![],
        functions: vec![],
        text_base: 0,
        data_base: 0,
    };
    assert!(elf.list_variables().is_empty());
}

// ─── Keybinding Completion Routing ──────────────────────

#[test]
fn tab_with_completion_active_in_input_expression_accepts() {
    let cmd = map_key(
        &key(KeyCode::Tab),
        PaneId::Expressions,
        InputMode::InputExpression,
        true,
    );
    assert!(matches!(cmd, Some(Command::CompletionAccept)));
}

#[test]
fn tab_with_completion_active_in_input_command_accepts() {
    let cmd = map_key(
        &key(KeyCode::Tab),
        PaneId::Console,
        InputMode::InputCommand,
        true,
    );
    assert!(matches!(cmd, Some(Command::CompletionAccept)));
}

#[test]
fn tab_without_completion_in_input_mode_returns_none() {
    let cmd = map_key(
        &key(KeyCode::Tab),
        PaneId::Expressions,
        InputMode::InputExpression,
        false,
    );
    assert!(cmd.is_none());
}

#[test]
fn tab_in_normal_mode_returns_next_pane() {
    let cmd = map_key(
        &key(KeyCode::Tab),
        PaneId::Source,
        InputMode::Normal,
        false,
    );
    assert!(matches!(cmd, Some(Command::NextPane)));
}

#[test]
fn tab_in_normal_mode_ignores_completion_active() {
    // Normal mode doesn't look at completion_active
    let cmd = map_key(
        &key(KeyCode::Tab),
        PaneId::Source,
        InputMode::Normal,
        true,
    );
    assert!(matches!(cmd, Some(Command::NextPane)));
}

#[test]
fn up_in_input_mode_maps_to_history_up() {
    let cmd = map_key(
        &key(KeyCode::Up),
        PaneId::Expressions,
        InputMode::InputExpression,
        false,
    );
    assert!(matches!(cmd, Some(Command::InputHistoryUp)));
}

#[test]
fn down_in_input_mode_maps_to_history_down() {
    let cmd = map_key(
        &key(KeyCode::Down),
        PaneId::Expressions,
        InputMode::InputExpression,
        false,
    );
    assert!(matches!(cmd, Some(Command::InputHistoryDown)));
}
