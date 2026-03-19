use crate::error::ProbeError;
use crate::probe::MemoryAccess;

/// FreeRTOS List_t offsets (standard 32-bit)
const LIST_NUM_ITEMS: u32 = 0;
const LIST_END_NEXT: u32 = 12; // xListEnd.pxNext → first real item

/// ListItem_t offsets
const ITEM_NEXT: u32 = 4;
const ITEM_OWNER: u32 = 12; // pvOwner → TCB pointer

/// Walk a FreeRTOS List_t at `list_addr`, returning the TCB addresses
/// of all tasks in the list.
pub fn walk_list(
    core: &mut probe_rs::Core<'_>,
    list_addr: u32,
) -> Result<Vec<u32>, ProbeError> {
    let num_items = MemoryAccess::read_u32(core, list_addr + LIST_NUM_ITEMS)?;
    if num_items == 0 {
        return Ok(Vec::new());
    }

    let mut tcb_addrs = Vec::with_capacity(num_items as usize);

    // Address of xListEnd within List_t (offset 8)
    let list_end_addr = list_addr + 8;
    // Get pointer to first real list item (xListEnd.pxNext)
    let mut current_item = MemoryAccess::read_u32(core, list_addr + LIST_END_NEXT)?;

    // Walk the circular linked list (safety cap at 64 tasks)
    for _ in 0..num_items.min(64) {
        if current_item == 0 || current_item == list_end_addr {
            break;
        }

        // Read pvOwner (TCB pointer) from this ListItem_t
        let tcb_addr = MemoryAccess::read_u32(core, current_item + ITEM_OWNER)?;
        if tcb_addr != 0 {
            tcb_addrs.push(tcb_addr);
        }

        // Follow pxNext
        current_item = MemoryAccess::read_u32(core, current_item + ITEM_NEXT)?;
    }

    Ok(tcb_addrs)
}
