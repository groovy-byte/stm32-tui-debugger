use probe_rs::MemoryInterface;
use tracing::debug;

use crate::error::ProbeError;

/// Stateless helper for typed memory reads and writes through a `probe_rs::Core`.
pub struct MemoryAccess;

impl MemoryAccess {
    /// Read `len` bytes starting at `addr`.
    pub fn read_u8(
        core: &mut probe_rs::Core<'_>,
        addr: u32,
        len: usize,
    ) -> Result<Vec<u8>, ProbeError> {
        debug!(addr = format_args!("0x{addr:08x}"), len, "read_u8");
        let mut buf = vec![0u8; len];
        core.read_8(addr as u64, &mut buf).map_err(|e| ProbeError::MemoryReadFailed {
            addr,
            reason: format!("{e:#}"),
        })?;
        Ok(buf)
    }

    /// Read a single 32-bit word at `addr`.
    pub fn read_u32(core: &mut probe_rs::Core<'_>, addr: u32) -> Result<u32, ProbeError> {
        debug!(addr = format_args!("0x{addr:08x}"), "read_u32");
        core.read_word_32(addr as u64).map_err(|e| ProbeError::MemoryReadFailed {
            addr,
            reason: format!("{e:#}"),
        })
    }

    /// Read `count` consecutive 32-bit words starting at `addr`.
    pub fn read_block_u32(
        core: &mut probe_rs::Core<'_>,
        addr: u32,
        count: usize,
    ) -> Result<Vec<u32>, ProbeError> {
        debug!(addr = format_args!("0x{addr:08x}"), count, "read_block_u32");
        let mut buf = vec![0u32; count];
        core.read_32(addr as u64, &mut buf).map_err(|e| ProbeError::MemoryReadFailed {
            addr,
            reason: format!("{e:#}"),
        })?;
        Ok(buf)
    }

    /// Write `data` bytes starting at `addr`.
    pub fn write_u8(
        core: &mut probe_rs::Core<'_>,
        addr: u32,
        data: &[u8],
    ) -> Result<(), ProbeError> {
        debug!(addr = format_args!("0x{addr:08x}"), len = data.len(), "write_u8");
        core.write_8(addr as u64, data).map_err(|e| ProbeError::MemoryWriteFailed {
            addr,
            reason: format!("{e:#}"),
        })
    }

    /// Write a single 32-bit word at `addr`.
    pub fn write_u32(
        core: &mut probe_rs::Core<'_>,
        addr: u32,
        value: u32,
    ) -> Result<(), ProbeError> {
        debug!(addr = format_args!("0x{addr:08x}"), value, "write_u32");
        core.write_word_32(addr as u64, value).map_err(|e| ProbeError::MemoryWriteFailed {
            addr,
            reason: format!("{e:#}"),
        })
    }
}
