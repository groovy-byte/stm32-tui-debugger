use cpp_demangle::Symbol;

/// Attempt to demangle a C++ (Itanium ABI) symbol name.
/// Returns the original name unchanged if it is not mangled.
pub fn demangle(mangled: &str) -> String {
    match Symbol::new(mangled.as_bytes()) {
        Ok(sym) => sym.to_string(),
        Err(_) => mangled.to_string(),
    }
}

/// Check whether a name looks like an Itanium-ABI mangled C++ symbol.
pub fn is_mangled(name: &str) -> bool {
    name.starts_with("_Z") || name.starts_with("__Z")
}
