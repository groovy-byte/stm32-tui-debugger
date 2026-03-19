use object::{Object, ObjectSection, ObjectSymbol, SymbolKind};
use std::fs;
use std::path::Path;

use crate::error::SymbolError;
use super::types::{FunctionInfo, Variable};

pub struct ElfData {
    pub raw_data: Vec<u8>,
    pub variables: Vec<Variable>,
    pub functions: Vec<FunctionInfo>,
    pub text_base: u32,
    pub data_base: u32,
}

impl ElfData {
    /// Load and parse an ELF file from disk.
    pub fn load(path: &Path) -> Result<Self, SymbolError> {
        let data = fs::read(path)
            .map_err(|e| SymbolError::ElfLoadFailed(format!("{}: {}", path.display(), e)))?;
        Self::load_from_bytes(&data)
    }

    /// Parse an ELF from a byte slice already in memory.
    pub fn load_from_bytes(data: &[u8]) -> Result<Self, SymbolError> {
        let object = object::File::parse(data)
            .map_err(|e| SymbolError::ElfLoadFailed(e.to_string()))?;

        let mut variables = Vec::new();
        let mut functions = Vec::new();

        for symbol in object.symbols() {
            let name = match symbol.name() {
                Ok(n) if !n.is_empty() => n.to_string(),
                _ => continue,
            };

            let address = symbol.address() as u32;
            let size = symbol.size();

            match symbol.kind() {
                SymbolKind::Data => {
                    variables.push(Variable {
                        name,
                        address,
                        size: size as usize,
                        type_name: String::new(),
                        is_global: symbol.is_global(),
                    });
                }
                SymbolKind::Text => {
                    functions.push(FunctionInfo {
                        name,
                        address,
                        size: size as u32,
                        source: None,
                    });
                }
                _ => {}
            }
        }

        let text_base = object
            .section_by_name(".text")
            .map(|s| s.address() as u32)
            .unwrap_or(0);

        let data_base = object
            .section_by_name(".data")
            .map(|s| s.address() as u32)
            .unwrap_or(0);

        Ok(Self {
            raw_data: data.to_vec(),
            variables,
            functions,
            text_base,
            data_base,
        })
    }

    pub fn find_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.iter().find(|v| v.name == name)
    }

    pub fn find_function(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn find_function_at(&self, addr: u32) -> Option<&FunctionInfo> {
        self.functions
            .iter()
            .find(|f| addr >= f.address && addr < f.address + f.size)
    }

    pub fn find_variables_by_prefix(&self, prefix: &str) -> Vec<&Variable> {
        let prefix_lower = prefix.to_ascii_lowercase();
        let mut matches: Vec<&Variable> = self
            .variables
            .iter()
            .filter(|v| v.name.to_ascii_lowercase().starts_with(&prefix_lower))
            .collect();
        matches.sort_by(|a, b| a.name.cmp(&b.name));
        matches
    }

    pub fn find_functions_by_prefix(&self, prefix: &str) -> Vec<&FunctionInfo> {
        let prefix_lower = prefix.to_ascii_lowercase();
        let mut matches: Vec<&FunctionInfo> = self
            .functions
            .iter()
            .filter(|f| f.name.to_ascii_lowercase().starts_with(&prefix_lower))
            .collect();
        matches.sort_by(|a, b| a.name.cmp(&b.name));
        matches
    }

    pub fn list_variables(&self) -> &[Variable] {
        &self.variables
    }
}
