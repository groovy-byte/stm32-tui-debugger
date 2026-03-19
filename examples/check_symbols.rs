use object::{Object, ObjectSymbol, SymbolKind};
use std::env;
use std::fs;

fn main() {
    let path = env::args().nth(1).expect("Usage: check_symbols <ELF>");
    let data = fs::read(&path).unwrap();
    let obj = object::File::parse(&*data).unwrap();

    let mut data_syms: Vec<String> = Vec::new();
    let mut text_count: u32 = 0;
    let mut unknown_syms: Vec<String> = Vec::new();
    let mut other_count: u32 = 0;

    for sym in obj.symbols() {
        let name = match sym.name() {
            Ok(n) if !n.is_empty() => n.to_string(),
            _ => continue,
        };
        match sym.kind() {
            SymbolKind::Data => data_syms.push(name),
            SymbolKind::Text => text_count += 1,
            SymbolKind::Unknown => unknown_syms.push(name),
            _ => other_count += 1,
        }
    }

    println!("Data (variables): {}", data_syms.len());
    println!("Text (functions): {}", text_count);
    println!("Unknown:          {}", unknown_syms.len());
    println!("Other:            {}", other_count);

    println!("\n=== Data symbols ===");
    for s in &data_syms {
        println!("  {}", s);
    }

    if !unknown_syms.is_empty() {
        println!("\n=== Unknown symbols (first 30) ===");
        for s in unknown_syms.iter().take(30) {
            println!("  {}", s);
        }
    }

    // Check specific targets
    println!("\n=== Specific symbol lookup ===");
    let targets = [
        "hi2c3", "htim2", "tests_run", "tests_passed",
        "pxCurrentTCB", "g_sensor_data", "current_resolution",
        "huart6", "servo_offset",
    ];
    for t in &targets {
        let found = obj.symbols().find(|s| s.name().map(|n| n == *t).unwrap_or(false));
        match found {
            Some(s) => println!(
                "  {:20} kind={:?}  addr=0x{:08x}  size={}  global={}",
                t,
                s.kind(),
                s.address(),
                s.size(),
                s.is_global()
            ),
            None => println!("  {:20} NOT FOUND", t),
        }
    }
}
