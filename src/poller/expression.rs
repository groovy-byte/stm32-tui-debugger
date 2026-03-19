use crate::error::PollerError;
use crate::symbols::SymbolEngine;
use super::types::{Expression, ValueFormat};

impl Expression {
    pub fn new(name: &str) -> Self {
        Self {
            id: name.to_string(),
            name: name.to_string(),
            address: None,
            size: None,
            format: ValueFormat::Auto,
        }
    }

    pub fn with_format(mut self, format: ValueFormat) -> Self {
        self.format = format;
        self
    }

    /// Resolve the expression against the symbol table.
    pub fn resolve(&mut self, symbols: &SymbolEngine) -> Result<(), PollerError> {
        let var = symbols
            .resolve_variable(&self.name)
            .map_err(|e| PollerError::ExpressionParseFailed(e.to_string()))?;
        self.address = Some(var.address);
        self.size = Some(var.size);
        Ok(())
    }

    pub fn is_resolved(&self) -> bool {
        self.address.is_some() && self.size.is_some()
    }
}
