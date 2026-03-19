pub mod types;
pub mod elf_loader;
pub mod dwarf_resolver;
pub mod demangler;

pub use types::*;
pub use elf_loader::ElfData;
pub use dwarf_resolver::DwarfResolver;

use std::path::Path;
use crate::error::SymbolError;

/// High-level symbol engine combining ELF + DWARF.
pub struct SymbolEngine {
    pub elf: ElfData,
    pub dwarf: DwarfResolver,
}

impl SymbolEngine {
    /// Load an ELF file and initialise both the symbol table and DWARF resolver.
    pub fn load(elf_path: &Path) -> Result<Self, SymbolError> {
        let raw = std::fs::read(elf_path)
            .map_err(|e| SymbolError::ElfLoadFailed(e.to_string()))?;
        let elf = ElfData::load_from_bytes(&raw)?;
        let dwarf = DwarfResolver::load(&raw)?;
        Ok(Self { elf, dwarf })
    }

    pub fn resolve_variable(&self, name: &str) -> Result<&Variable, SymbolError> {
        self.elf
            .find_variable(name)
            .ok_or_else(|| SymbolError::VariableNotFound(name.to_string()))
    }

    pub fn get_source_location(
        &self,
        addr: u32,
    ) -> Result<Option<SourceLocation>, SymbolError> {
        self.dwarf.get_source_location(addr as u64)
    }

    pub fn demangle_name(&self, name: &str) -> String {
        demangler::demangle(name)
    }
}
