use std::rc::Rc;

use addr2line::Context;
use gimli::RunTimeEndian;

use crate::error::SymbolError;
use super::types::SourceLocation;

type Reader = gimli::EndianReader<RunTimeEndian, Rc<[u8]>>;

pub struct DwarfResolver {
    context: Context<Reader>,
    /// Leaked allocation kept alive so `context` can borrow it with a 'static lifetime.
    _data: &'static [u8],
}

impl DwarfResolver {
    /// Build a DWARF resolver by leaking the ELF data so that
    /// `addr2line::Context` can hold a `'static` borrow.
    pub fn load(elf_data: &[u8]) -> Result<Self, SymbolError> {
        let leaked: &'static [u8] = Box::leak(elf_data.to_vec().into_boxed_slice());
        let object = object::File::parse(leaked)
            .map_err(|e| SymbolError::ElfLoadFailed(e.to_string()))?;
        let context =
            Context::new(&object).map_err(|e| SymbolError::DwarfParse(e.to_string()))?;
        Ok(Self {
            context,
            _data: leaked,
        })
    }

    /// Resolve an instruction address to a source file location.
    pub fn get_source_location(
        &self,
        addr: u64,
    ) -> Result<Option<SourceLocation>, SymbolError> {
        let loc = self
            .context
            .find_location(addr)
            .map_err(|e| SymbolError::DwarfParse(e.to_string()))?;

        Ok(loc.map(|l| SourceLocation {
            file: l.file.unwrap_or("<unknown>").to_string(),
            line: l.line.unwrap_or(0),
            column: l.column,
        }))
    }

    /// Resolve an instruction address to a function name.
    pub fn get_function_name(
        &self,
        addr: u64,
    ) -> Result<Option<String>, SymbolError> {
        let mut frames = self
            .context
            .find_frames(addr)
            .skip_all_loads()
            .map_err(|e| SymbolError::DwarfParse(e.to_string()))?;

        match frames.next().map_err(|e| SymbolError::DwarfParse(e.to_string()))? {
            Some(frame) => Ok(frame.function.map(|f| {
                f.demangle()
                    .map(|cow| cow.into_owned())
                    .unwrap_or_else(|raw| raw.to_string())
            })),
            None => Ok(None),
        }
    }
}
