use stm32_tui_debugger::symbols::demangler;
use stm32_tui_debugger::symbols::types::*;

// ── Demangler ───────────────────────────────────────────────────────────

#[test]
fn should_demangle_cpp_function_name() {
    // _Z3foov → foo()
    let result = demangler::demangle("_Z3foov");
    assert_eq!(result, "foo()");
}

#[test]
fn should_demangle_cpp_function_with_int_param() {
    // _Z3bari → bar(int)
    let result = demangler::demangle("_Z3bari");
    assert_eq!(result, "bar(int)");
}

#[test]
fn should_demangle_namespaced_function() {
    // _ZN3foo3barEv → foo::bar()
    let result = demangler::demangle("_ZN3foo3barEv");
    assert_eq!(result, "foo::bar()");
}

#[test]
fn should_return_original_for_c_function_name() {
    let result = demangler::demangle("main");
    assert_eq!(result, "main");
}

#[test]
fn should_return_original_for_unmangled_name() {
    let result = demangler::demangle("HAL_GPIO_Init");
    assert_eq!(result, "HAL_GPIO_Init");
}

#[test]
fn should_detect_mangled_name_with_z_prefix() {
    assert!(demangler::is_mangled("_Z3foov"));
}

#[test]
fn should_detect_mangled_name_with_double_underscore_z() {
    assert!(demangler::is_mangled("__Z3foov"));
}

#[test]
fn should_not_detect_plain_c_name_as_mangled() {
    assert!(!demangler::is_mangled("main"));
}

#[test]
fn should_not_detect_hal_name_as_mangled() {
    assert!(!demangler::is_mangled("HAL_GPIO_Init"));
}

#[test]
fn should_not_detect_underscore_prefixed_name_as_mangled() {
    assert!(!demangler::is_mangled("_start"));
}

// ── Variable struct ─────────────────────────────────────────────────────

#[test]
fn should_create_struct_typekind() {
    let member = StructMember {
        name: "x".into(),
        offset: 0,
        type_info: TypeInfo {
            name: "int".into(),
            size: 4,
            kind: TypeKind::Primitive(PrimitiveType::I32),
        },
    };
    let tk = TypeKind::Struct {
        members: vec![member],
    };
    if let TypeKind::Struct { members } = tk {
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, "x");
    } else {
        panic!("Expected Struct variant");
    }
}

#[test]
fn should_create_array_typekind() {
    let elem = TypeInfo {
        name: "u8".into(),
        size: 1,
        kind: TypeKind::Primitive(PrimitiveType::U8),
    };
    let tk = TypeKind::Array {
        element_type: Box::new(elem),
        length: 10,
    };
    if let TypeKind::Array { element_type, length } = tk {
        assert_eq!(length, 10);
        assert_eq!(element_type.size, 1);
    } else {
        panic!("Expected Array variant");
    }
}

#[test]
fn should_create_pointer_typekind() {
    let target = TypeInfo {
        name: "u32".into(),
        size: 4,
        kind: TypeKind::Primitive(PrimitiveType::U32),
    };
    let tk = TypeKind::Pointer {
        target_type: Box::new(target),
        pointer_size: 4,
    };
    if let TypeKind::Pointer {
        target_type,
        pointer_size,
    } = tk
    {
        assert_eq!(pointer_size, 4);
        assert_eq!(target_type.name, "u32");
    } else {
        panic!("Expected Pointer variant");
    }
}

#[test]
fn should_create_enum_typekind() {
    let tk = TypeKind::Enum {
        variants: vec![("A".into(), 0), ("B".into(), 1), ("C".into(), 2)],
    };
    if let TypeKind::Enum { variants } = tk {
        assert_eq!(variants.len(), 3);
        assert_eq!(variants[0], ("A".into(), 0));
    } else {
        panic!("Expected Enum variant");
    }
}

#[test]
fn should_create_typedef_typekind() {
    let underlying = TypeInfo {
        name: "u32".into(),
        size: 4,
        kind: TypeKind::Primitive(PrimitiveType::U32),
    };
    let tk = TypeKind::Typedef {
        underlying: Box::new(underlying),
    };
    if let TypeKind::Typedef { underlying } = tk {
        assert_eq!(underlying.name, "u32");
    } else {
        panic!("Expected Typedef variant");
    }
}

#[test]
fn should_create_unknown_typekind() {
    let tk = TypeKind::Unknown;
    assert!(matches!(tk, TypeKind::Unknown));
}

// ── TypeInfo ────────────────────────────────────────────────────────────

#[test]
fn should_create_typeinfo() {
    let ti = TypeInfo {
        name: "float".into(),
        size: 4,
        kind: TypeKind::Primitive(PrimitiveType::F32),
    };
    assert_eq!(ti.name, "float");
    assert_eq!(ti.size, 4);
}

// ── SourceLocation ──────────────────────────────────────────────────────

// ── FunctionInfo ────────────────────────────────────────────────────────

#[test]
fn should_create_function_info() {
    let fi = FunctionInfo {
        name: "HAL_Init".into(),
        address: 0x0800_0100,
        size: 64,
        source: Some(SourceLocation {
            file: "hal.c".into(),
            line: 10,
            column: None,
        }),
    };
    assert_eq!(fi.name, "HAL_Init");
    assert_eq!(fi.address, 0x0800_0100);
    assert_eq!(fi.size, 64);
    assert!(fi.source.is_some());
}
