use crate::error::ProbeError;
use crate::probe::MemoryAccess;
use super::types::{TcbLayout, TaskInfo, TaskState};

/// Parse a TCB struct from raw memory at the given address.
pub fn read_tcb(
    core: &mut probe_rs::Core<'_>,
    tcb_addr: u32,
    layout: &TcbLayout,
    state: TaskState,
) -> Result<TaskInfo, ProbeError> {
    let top_of_stack = MemoryAccess::read_u32(core, tcb_addr + layout.top_of_stack as u32)?;
    let priority = MemoryAccess::read_u32(core, tcb_addr + layout.priority as u32)?;
    let stack_base = MemoryAccess::read_u32(core, tcb_addr + layout.stack_base as u32)?;

    // Read task name (null-terminated string)
    let name_bytes = MemoryAccess::read_u8(
        core,
        tcb_addr + layout.task_name as u32,
        layout.task_name_len,
    )?;
    let name = String::from_utf8_lossy(
        &name_bytes[..name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len())],
    )
    .to_string();

    Ok(TaskInfo {
        name,
        tcb_address: tcb_addr,
        priority,
        state,
        stack_top: top_of_stack,
        stack_base,
        stack_high_water: None,
    })
}
