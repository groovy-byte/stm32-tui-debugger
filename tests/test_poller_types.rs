use std::time::Instant;
use stm32_tui_debugger::poller::{DisplayValue, Expression, PollResult, ValueFormat};

// ── ValueFormat ─────────────────────────────────────────────────────────

// ── Expression ──────────────────────────────────────────────────────────

#[test]
fn should_have_no_address_initially() {
    let expr = Expression::new("variable");
    assert!(expr.address.is_none());
}

#[test]
fn should_have_no_size_initially() {
    let expr = Expression::new("variable");
    assert!(expr.size.is_none());
}

#[test]
fn should_default_format_to_auto() {
    let expr = Expression::new("test");
    assert_eq!(expr.format, ValueFormat::Auto);
}

#[test]
fn should_chain_with_format() {
    let expr = Expression::new("test").with_format(ValueFormat::Hex);
    assert_eq!(expr.format, ValueFormat::Hex);
}

#[test]
fn should_preserve_name_after_with_format() {
    let expr = Expression::new("counter").with_format(ValueFormat::Binary);
    assert_eq!(expr.name, "counter");
    assert_eq!(expr.format, ValueFormat::Binary);
}

// ── DisplayValue Display formatting ─────────────────────────────────────

#[test]
fn should_display_integer() {
    let v = DisplayValue::Integer(-42);
    assert_eq!(format!("{v}"), "-42");
}

#[test]
fn should_display_unsigned() {
    let v = DisplayValue::Unsigned(255);
    assert_eq!(format!("{v}"), "255");
}

#[test]
fn should_display_float() {
    let v = DisplayValue::Float(3.14);
    assert_eq!(format!("{v}"), "3.140000");
}

#[test]
fn should_display_float_zero() {
    let v = DisplayValue::Float(0.0);
    assert_eq!(format!("{v}"), "0.000000");
}

#[test]
fn should_display_bytes() {
    let v = DisplayValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]);
    assert_eq!(format!("{v}"), "[de ad be ef]");
}

#[test]
fn should_display_empty_bytes() {
    let v = DisplayValue::Bytes(vec![]);
    assert_eq!(format!("{v}"), "[]");
}

#[test]
fn should_display_text() {
    let v = DisplayValue::Text("hello".into());
    assert_eq!(format!("{v}"), "hello");
}

#[test]
fn should_display_struct() {
    let v = DisplayValue::Struct(vec![
        ("x".into(), DisplayValue::Integer(1)),
        ("y".into(), DisplayValue::Integer(2)),
    ]);
    assert_eq!(format!("{v}"), "{ x: 1, y: 2 }");
}

#[test]
fn should_display_single_member_struct() {
    let v = DisplayValue::Struct(vec![("val".into(), DisplayValue::Unsigned(42))]);
    assert_eq!(format!("{v}"), "{ val: 42 }");
}

#[test]
fn should_display_empty_struct() {
    let v = DisplayValue::Struct(vec![]);
    assert_eq!(format!("{v}"), "{  }");
}

#[test]
fn should_display_array() {
    let v = DisplayValue::Array(vec![
        DisplayValue::Integer(1),
        DisplayValue::Integer(2),
        DisplayValue::Integer(3),
    ]);
    assert_eq!(format!("{v}"), "[1, 2, 3]");
}

#[test]
fn should_display_empty_array() {
    let v = DisplayValue::Array(vec![]);
    assert_eq!(format!("{v}"), "[]");
}

#[test]
fn should_display_error() {
    let v = DisplayValue::Error("read timeout".into());
    assert_eq!(format!("{v}"), "<error: read timeout>");
}

// ── DisplayValue equality ───────────────────────────────────────────────

// ── Nested DisplayValue ─────────────────────────────────────────────────

#[test]
fn should_display_nested_struct_in_array() {
    let v = DisplayValue::Array(vec![DisplayValue::Struct(vec![(
        "id".into(),
        DisplayValue::Unsigned(1),
    )])]);
    assert_eq!(format!("{v}"), "[{ id: 1 }]");
}

// ── PollResult ──────────────────────────────────────────────────────────

#[test]
fn should_create_poll_result() {
    let result = PollResult {
        expr_id: "my_var".into(),
        value: DisplayValue::Integer(100),
        changed: true,
        timestamp: Instant::now(),
        error: None,
    };
    assert_eq!(result.expr_id, "my_var");
    assert!(result.changed);
    assert!(result.error.is_none());
}

#[test]
fn should_create_poll_result_with_error() {
    let result = PollResult {
        expr_id: "bad_var".into(),
        value: DisplayValue::Error("failed".into()),
        changed: false,
        timestamp: Instant::now(),
        error: Some("read failed".into()),
    };
    assert!(!result.changed);
    assert_eq!(result.error.as_deref(), Some("read failed"));
}
