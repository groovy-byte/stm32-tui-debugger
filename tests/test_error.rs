use stm32_tui_debugger::error::*;

// ── Display formatting ──────────────────────────────────────────────────

#[test]
fn should_display_probe_connection_failed() {
    let e = ProbeError::ConnectionFailed("USB timeout".into());
    assert_eq!(e.to_string(), "Failed to connect to probe: USB timeout");
}

#[test]
fn should_display_probe_no_session() {
    let e = ProbeError::NoSession;
    assert_eq!(e.to_string(), "No active session");
}

#[test]
fn should_display_probe_target_running() {
    let e = ProbeError::TargetRunning;
    assert_eq!(e.to_string(), "Target not halted");
}

#[test]
fn should_display_memory_read_failed_with_hex_addr() {
    let e = ProbeError::MemoryReadFailed {
        addr: 0x2000_0100,
        reason: "bus fault".into(),
    };
    assert_eq!(
        e.to_string(),
        "Memory read failed at 0x20000100: bus fault"
    );
}

#[test]
fn should_display_memory_write_failed_with_hex_addr() {
    let e = ProbeError::MemoryWriteFailed {
        addr: 0x4002_0000,
        reason: "access denied".into(),
    };
    assert_eq!(
        e.to_string(),
        "Memory write failed at 0x40020000: access denied"
    );
}

#[test]
fn should_display_register_access_failed() {
    let e = ProbeError::RegisterAccessFailed("PC not readable".into());
    assert_eq!(e.to_string(), "Register access failed: PC not readable");
}

#[test]
fn should_display_flash_failed() {
    let e = ProbeError::FlashFailed("erase timeout".into());
    assert_eq!(e.to_string(), "Flash failed: erase timeout");
}

#[test]
fn should_display_flash_failed_with_detailed_message() {
    let e = ProbeError::FlashFailed("sector 3 at 0x0800C000: verify mismatch".into());
    assert_eq!(
        e.to_string(),
        "Flash failed: sector 3 at 0x0800C000: verify mismatch"
    );
}

#[test]
fn should_convert_flash_failed_to_app_error() {
    let probe = ProbeError::FlashFailed("write error".into());
    let app: AppError = probe.into();
    assert_eq!(app.to_string(), "Probe error: Flash failed: write error");
}

// ── Symbol errors ───────────────────────────────────────────────────────

#[test]
fn should_display_elf_load_failed() {
    let e = SymbolError::ElfLoadFailed("not an ELF".into());
    assert_eq!(e.to_string(), "ELF load failed: not an ELF");
}

#[test]
fn should_display_variable_not_found() {
    let e = SymbolError::VariableNotFound("my_var".into());
    assert_eq!(e.to_string(), "Variable not found: my_var");
}

#[test]
fn should_display_type_resolution_failed() {
    let e = SymbolError::TypeResolutionFailed("uint32_t".into());
    assert_eq!(e.to_string(), "Type resolution failed for: uint32_t");
}

#[test]
fn should_display_dwarf_parse_error() {
    let e = SymbolError::DwarfParse("bad CU header".into());
    assert_eq!(e.to_string(), "DWARF parsing error: bad CU header");
}

#[test]
fn should_display_demangle_failed() {
    let e = SymbolError::DemangleFailed("invalid mangled name".into());
    assert_eq!(e.to_string(), "Demangling failed: invalid mangled name");
}

// ── SVD errors ──────────────────────────────────────────────────────────

#[test]
fn should_display_svd_fetch_failed() {
    let e = SvdError::FetchFailed("network error".into());
    assert_eq!(e.to_string(), "SVD fetch failed: network error");
}

#[test]
fn should_display_svd_parse_error() {
    let e = SvdError::ParseError("malformed XML".into());
    assert_eq!(e.to_string(), "SVD parse error: malformed XML");
}

#[test]
fn should_display_peripheral_not_found() {
    let e = SvdError::PeripheralNotFound("SPI3".into());
    assert_eq!(e.to_string(), "Peripheral not found: SPI3");
}

#[test]
fn should_display_register_not_found() {
    let e = SvdError::RegisterNotFound("CR1".into());
    assert_eq!(e.to_string(), "Register not found: CR1");
}

// ── Poller errors ───────────────────────────────────────────────────────

#[test]
fn should_display_expression_parse_failed() {
    let e = PollerError::ExpressionParseFailed("bad token".into());
    assert_eq!(e.to_string(), "Expression parse failed: bad token");
}

#[test]
fn should_display_evaluation_failed() {
    let e = PollerError::EvaluationFailed("div by zero".into());
    assert_eq!(e.to_string(), "Evaluation failed: div by zero");
}

#[test]
fn should_display_type_mismatch() {
    let e = PollerError::TypeMismatch("expected u32".into());
    assert_eq!(e.to_string(), "Type mismatch: expected u32");
}

// ── TUI errors ──────────────────────────────────────────────────────────

#[test]
fn should_display_terminal_setup_failed() {
    let e = TuiError::TerminalSetupFailed("no TTY".into());
    assert_eq!(e.to_string(), "Terminal setup failed: no TTY");
}

#[test]
fn should_display_render_failed() {
    let e = TuiError::RenderFailed("buffer overflow".into());
    assert_eq!(e.to_string(), "Rendering error: buffer overflow");
}

// ── From conversions into AppError ──────────────────────────────────────

#[test]
fn should_convert_probe_error_to_app_error() {
    let probe = ProbeError::NoSession;
    let app: AppError = probe.into();
    assert_eq!(app.to_string(), "Probe error: No active session");
}

#[test]
fn should_convert_symbol_error_to_app_error() {
    let sym = SymbolError::VariableNotFound("x".into());
    let app: AppError = sym.into();
    assert_eq!(app.to_string(), "Symbol error: Variable not found: x");
}

#[test]
fn should_convert_svd_error_to_app_error() {
    let svd = SvdError::PeripheralNotFound("GPIOB".into());
    let app: AppError = svd.into();
    assert_eq!(app.to_string(), "SVD error: Peripheral not found: GPIOB");
}

#[test]
fn should_convert_poller_error_to_app_error() {
    let poller = PollerError::TypeMismatch("u8 vs u32".into());
    let app: AppError = poller.into();
    assert_eq!(app.to_string(), "Poller error: Type mismatch: u8 vs u32");
}

#[test]
fn should_convert_tui_error_to_app_error() {
    let tui = TuiError::RenderFailed("oops".into());
    let app: AppError = tui.into();
    assert_eq!(app.to_string(), "TUI error: Rendering error: oops");
}

#[test]
fn should_convert_io_error_to_app_error() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let app: AppError = io.into();
    assert!(app.to_string().contains("file missing"));
}

#[test]
fn should_display_config_error() {
    let app = AppError::Config("bad TOML".into());
    assert_eq!(app.to_string(), "Config error: bad TOML");
}

// ── Memory address edge cases ───────────────────────────────────────────

#[test]
fn should_format_zero_address() {
    let e = ProbeError::MemoryReadFailed {
        addr: 0x0000_0000,
        reason: "null ptr".into(),
    };
    assert_eq!(e.to_string(), "Memory read failed at 0x00000000: null ptr");
}

#[test]
fn should_format_max_address() {
    let e = ProbeError::MemoryReadFailed {
        addr: 0xFFFF_FFFF,
        reason: "out of range".into(),
    };
    assert_eq!(
        e.to_string(),
        "Memory read failed at 0xffffffff: out of range"
    );
}
