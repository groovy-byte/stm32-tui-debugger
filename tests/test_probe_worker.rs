//! Tests for the probe worker module.
//!
//! Run with: `cargo test --features test-support test_probe_worker`
//!
//! The `test-support` feature enables `ProbeHandle::dead()`, which creates a
//! handle backed by a disconnected channel (simulates a dead worker thread).

use stm32_tui_debugger::poller::ValueFormat;
use stm32_tui_debugger::probe::{MemReadRequest, RtosAddrs};
use stm32_tui_debugger::rtos::{RtosSnapshot, TaskInfo, TaskState, TcbLayout};

// ── Helpers ─────────────────────────────────────────────────────────────

fn make_task(name: &str, priority: u32, state: TaskState) -> TaskInfo {
    TaskInfo {
        name: name.to_string(),
        tcb_address: 0x2000_0100,
        priority,
        state,
        stack_top: 0x2000_1000,
        stack_base: 0x2000_0000,
        stack_high_water: None,
    }
}

fn sample_rtos_addrs() -> RtosAddrs {
    RtosAddrs {
        current_tcb: 0x2000_0000,
        num_tasks: Some(0x2000_0004),
        ready_lists: Some(0x2000_0100),
        delayed1: Some(0x2000_0200),
        delayed2: Some(0x2000_0300),
        suspended: Some(0x2000_0400),
        max_priorities: 56,
    }
}

// ── 1. RtosAddrs construction and defaults ──────────────────────────────

#[test]
fn should_populate_all_rtos_addrs_fields() {
    let addrs = sample_rtos_addrs();
    assert_eq!(addrs.current_tcb, 0x2000_0000);
    assert_eq!(addrs.num_tasks, Some(0x2000_0004));
    assert_eq!(addrs.ready_lists, Some(0x2000_0100));
    assert_eq!(addrs.delayed1, Some(0x2000_0200));
    assert_eq!(addrs.delayed2, Some(0x2000_0300));
    assert_eq!(addrs.suspended, Some(0x2000_0400));
    assert_eq!(addrs.max_priorities, 56);
}

#[test]
fn should_allow_optional_fields_as_none() {
    let addrs = RtosAddrs {
        current_tcb: 0x2000_0000,
        num_tasks: None,
        ready_lists: None,
        delayed1: None,
        delayed2: None,
        suspended: None,
        max_priorities: 5,
    };
    assert!(addrs.num_tasks.is_none());
    assert!(addrs.ready_lists.is_none());
    assert!(addrs.delayed1.is_none());
    assert!(addrs.delayed2.is_none());
    assert!(addrs.suspended.is_none());
}

#[test]
fn should_accept_zero_max_priorities() {
    let addrs = RtosAddrs {
        current_tcb: 0x2000_0000,
        num_tasks: None,
        ready_lists: None,
        delayed1: None,
        delayed2: None,
        suspended: None,
        max_priorities: 0,
    };
    assert_eq!(addrs.max_priorities, 0);
}

#[test]
fn should_accept_one_max_priority() {
    let addrs = RtosAddrs {
        current_tcb: 0x2000_0000,
        num_tasks: None,
        ready_lists: None,
        delayed1: None,
        delayed2: None,
        suspended: None,
        max_priorities: 1,
    };
    assert_eq!(addrs.max_priorities, 1);
}

#[test]
fn should_accept_typical_freertos_max_priorities() {
    let addrs = RtosAddrs {
        current_tcb: 0x2000_0000,
        num_tasks: None,
        ready_lists: None,
        delayed1: None,
        delayed2: None,
        suspended: None,
        max_priorities: 56,
    };
    assert_eq!(addrs.max_priorities, 56);
}

#[test]
fn should_clone_rtos_addrs() {
    let addrs = sample_rtos_addrs();
    let cloned = addrs.clone();
    assert_eq!(cloned.current_tcb, addrs.current_tcb);
    assert_eq!(cloned.max_priorities, addrs.max_priorities);
    assert_eq!(cloned.num_tasks, addrs.num_tasks);
}

#[test]
fn should_debug_format_rtos_addrs() {
    let addrs = sample_rtos_addrs();
    let debug = format!("{addrs:?}");
    assert!(debug.contains("RtosAddrs"));
    assert!(debug.contains("current_tcb"));
}

// ── 2. MemReadRequest construction ──────────────────────────────────────

#[test]
fn should_create_mem_read_request_with_all_fields() {
    let req = MemReadRequest {
        id: "var1".to_string(),
        addr: 0x0800_0000,
        size: 4,
        format: ValueFormat::Hex,
    };
    assert_eq!(req.id, "var1");
    assert_eq!(req.addr, 0x0800_0000);
    assert_eq!(req.size, 4);
    assert_eq!(req.format, ValueFormat::Hex);
}

#[test]
fn should_create_zero_size_mem_read_request() {
    let req = MemReadRequest {
        id: "empty".to_string(),
        addr: 0x2000_0000,
        size: 0,
        format: ValueFormat::Auto,
    };
    assert_eq!(req.size, 0);
}

#[test]
fn should_create_mem_read_request_with_decimal_format() {
    let req = MemReadRequest {
        id: "counter".to_string(),
        addr: 0x2000_1000,
        size: 4,
        format: ValueFormat::Decimal,
    };
    assert_eq!(req.format, ValueFormat::Decimal);
}

#[test]
fn should_create_mem_read_request_with_binary_format() {
    let req = MemReadRequest {
        id: "flags".to_string(),
        addr: 0x4000_0000,
        size: 1,
        format: ValueFormat::Binary,
    };
    assert_eq!(req.format, ValueFormat::Binary);
}

#[test]
fn should_create_mem_read_request_with_float_format() {
    let req = MemReadRequest {
        id: "temperature".to_string(),
        addr: 0x2000_2000,
        size: 4,
        format: ValueFormat::Float,
    };
    assert_eq!(req.format, ValueFormat::Float);
}

#[test]
fn should_create_mem_read_request_with_auto_format() {
    let req = MemReadRequest {
        id: "raw".to_string(),
        addr: 0x1000,
        size: 16,
        format: ValueFormat::Auto,
    };
    assert_eq!(req.format, ValueFormat::Auto);
}

#[test]
fn should_clone_mem_read_request() {
    let req = MemReadRequest {
        id: "x".to_string(),
        addr: 0x1000,
        size: 8,
        format: ValueFormat::Hex,
    };
    let cloned = req.clone();
    assert_eq!(cloned.id, req.id);
    assert_eq!(cloned.addr, req.addr);
    assert_eq!(cloned.size, req.size);
    assert_eq!(cloned.format, req.format);
}

#[test]
fn should_debug_format_mem_read_request() {
    let req = MemReadRequest {
        id: "test".to_string(),
        addr: 0x1234,
        size: 2,
        format: ValueFormat::Auto,
    };
    let debug = format!("{req:?}");
    assert!(debug.contains("MemReadRequest"));
    assert!(debug.contains("test"));
}

// ── 3. ProbeHandle error paths (channel disconnection) ──────────────────

#[cfg(feature = "test-support")]
mod probe_handle_error_paths {
    use super::*;
    use stm32_tui_debugger::error::ProbeError;
    use stm32_tui_debugger::probe::ProbeHandle;

    #[tokio::test]
    async fn should_return_no_session_when_halt_on_dead_worker() {
        let handle = ProbeHandle::dead();
        let result = handle.halt().await;
        assert!(matches!(result, Err(ProbeError::NoSession)));
    }

    #[tokio::test]
    async fn should_return_no_session_when_resume_on_dead_worker() {
        let handle = ProbeHandle::dead();
        let result = handle.resume().await;
        assert!(matches!(result, Err(ProbeError::NoSession)));
    }

    #[tokio::test]
    async fn should_return_no_session_when_reset_on_dead_worker() {
        let handle = ProbeHandle::dead();
        let result = handle.reset().await;
        assert!(matches!(result, Err(ProbeError::NoSession)));
    }

    #[tokio::test]
    async fn should_return_empty_vec_when_read_memory_batch_on_dead_worker() {
        let handle = ProbeHandle::dead();
        let requests = vec![MemReadRequest {
            id: "x".to_string(),
            addr: 0x2000_0000,
            size: 4,
            format: ValueFormat::Hex,
        }];
        let result = handle.read_memory_batch(requests).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn should_return_no_session_when_read_rtos_snapshot_on_dead_worker() {
        let handle = ProbeHandle::dead();
        let addrs = sample_rtos_addrs();
        let layout = TcbLayout::default();
        let result = handle.read_rtos_snapshot(addrs, layout).await;
        assert!(matches!(result, Err(ProbeError::NoSession)));
    }

    #[tokio::test]
    async fn should_not_panic_when_shutdown_on_dead_worker() {
        let handle = ProbeHandle::dead();
        handle.shutdown();
    }

    #[tokio::test]
    async fn should_return_empty_vec_when_batch_is_empty_on_dead_worker() {
        let handle = ProbeHandle::dead();
        let result = handle.read_memory_batch(vec![]).await;
        assert!(result.is_empty());
    }
}

// ── 4. ProbeHandle concurrent access ────────────────────────────────────

#[cfg(feature = "test-support")]
mod probe_handle_concurrent {
    use stm32_tui_debugger::error::ProbeError;
    use stm32_tui_debugger::probe::ProbeHandle;

    #[tokio::test]
    async fn should_allow_cloned_handles_to_send_commands() {
        let handle = ProbeHandle::dead();
        let clone1 = handle.clone();
        let clone2 = handle.clone();

        assert!(matches!(clone1.halt().await, Err(ProbeError::NoSession)));
        assert!(matches!(clone2.resume().await, Err(ProbeError::NoSession)));
    }

    #[tokio::test]
    async fn should_handle_rapid_sequential_commands_without_deadlock() {
        let handle = ProbeHandle::dead();
        for _ in 0..100 {
            let _ = handle.halt().await;
            let _ = handle.resume().await;
            let _ = handle.reset().await;
        }
    }

    #[tokio::test]
    async fn should_handle_multiple_shutdowns_without_panic() {
        let handle = ProbeHandle::dead();
        handle.shutdown();
        handle.shutdown();
        handle.shutdown();
    }
}

// ── 5. Integration with existing RTOS types ─────────────────────────────

#[test]
fn should_create_empty_rtos_snapshot() {
    let snap = RtosSnapshot {
        current_task_addr: 0,
        total_tasks: 0,
        tasks: Vec::new(),
        error: None,
    };
    assert!(snap.tasks.is_empty());
    assert_eq!(snap.total_tasks, 0);
    assert!(snap.error.is_none());
}

#[test]
fn should_sort_tasks_by_priority_descending_then_name() {
    let mut tasks = vec![
        make_task("Comm", 3, TaskState::Blocked),
        make_task("Main", 5, TaskState::Running),
        make_task("Idle", 0, TaskState::Ready),
        make_task("Audio", 3, TaskState::Ready),
    ];
    tasks.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.name.cmp(&b.name)));

    assert_eq!(tasks[0].name, "Main");
    assert_eq!(tasks[1].name, "Audio");
    assert_eq!(tasks[2].name, "Comm");
    assert_eq!(tasks[3].name, "Idle");
}

#[test]
fn should_match_worker_sort_order_for_snapshot_tasks() {
    let mut tasks = vec![
        make_task("B_task", 2, TaskState::Ready),
        make_task("A_task", 2, TaskState::Blocked),
        make_task("High", 10, TaskState::Running),
    ];
    // This is the exact sort used in do_rtos_snapshot
    tasks.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.name.cmp(&b.name)));

    assert_eq!(tasks[0].name, "High");
    assert_eq!(tasks[1].name, "A_task");
    assert_eq!(tasks[2].name, "B_task");
}

#[test]
fn should_default_tcb_layout_to_cortex_m_offsets() {
    let layout = TcbLayout::default();
    assert_eq!(layout.top_of_stack, 0);
    assert_eq!(layout.state_list_item, 4);
    assert_eq!(layout.event_list_item, 24);
    assert_eq!(layout.priority, 44);
    assert_eq!(layout.stack_base, 48);
    assert_eq!(layout.task_name, 52);
    assert_eq!(layout.task_name_len, 16);
}

#[test]
fn should_default_value_format_to_auto() {
    assert_eq!(ValueFormat::default(), ValueFormat::Auto);
}

#[test]
fn should_create_snapshot_with_error_message() {
    let snap = RtosSnapshot {
        current_task_addr: 0,
        total_tasks: 0,
        tasks: Vec::new(),
        error: Some("pxCurrentTCB not found".to_string()),
    };
    assert_eq!(snap.error.as_deref(), Some("pxCurrentTCB not found"));
}

#[test]
fn should_clone_rtos_snapshot() {
    let snap = RtosSnapshot {
        current_task_addr: 0x2000_0100,
        total_tasks: 2,
        tasks: vec![
            make_task("A", 5, TaskState::Running),
            make_task("B", 1, TaskState::Ready),
        ],
        error: None,
    };
    let cloned = snap.clone();
    assert_eq!(cloned.total_tasks, 2);
    assert_eq!(cloned.tasks.len(), 2);
    assert_eq!(cloned.current_task_addr, 0x2000_0100);
}

// ── 6. Regression: freeze bug (non-blocking async) ──────────────────────

#[cfg(feature = "test-support")]
mod freeze_regression {
    use super::*;
    use stm32_tui_debugger::probe::ProbeHandle;
    use std::time::Duration;

    #[tokio::test]
    async fn should_complete_halt_within_deadline_when_worker_dead() {
        let handle = ProbeHandle::dead();
        let result = tokio::time::timeout(Duration::from_millis(100), handle.halt()).await;
        assert!(result.is_ok(), "halt() must not block when worker is dead");
    }

    #[tokio::test]
    async fn should_complete_resume_within_deadline_when_worker_dead() {
        let handle = ProbeHandle::dead();
        let result = tokio::time::timeout(Duration::from_millis(100), handle.resume()).await;
        assert!(result.is_ok(), "resume() must not block when worker is dead");
    }

    #[tokio::test]
    async fn should_complete_reset_within_deadline_when_worker_dead() {
        let handle = ProbeHandle::dead();
        let result = tokio::time::timeout(Duration::from_millis(100), handle.reset()).await;
        assert!(result.is_ok(), "reset() must not block when worker is dead");
    }

    #[tokio::test]
    async fn should_complete_read_memory_batch_within_deadline_when_worker_dead() {
        let handle = ProbeHandle::dead();
        let result =
            tokio::time::timeout(Duration::from_millis(100), handle.read_memory_batch(vec![]))
                .await;
        assert!(
            result.is_ok(),
            "read_memory_batch() must not block when worker is dead"
        );
    }

    #[tokio::test]
    async fn should_complete_read_rtos_snapshot_within_deadline_when_worker_dead() {
        let handle = ProbeHandle::dead();
        let addrs = sample_rtos_addrs();
        let layout = TcbLayout::default();
        let result =
            tokio::time::timeout(Duration::from_millis(100), handle.read_rtos_snapshot(addrs, layout))
                .await;
        assert!(
            result.is_ok(),
            "read_rtos_snapshot() must not block when worker is dead"
        );
    }

    #[tokio::test]
    async fn should_complete_all_commands_within_combined_deadline_when_worker_dead() {
        let handle = ProbeHandle::dead();
        let result = tokio::time::timeout(Duration::from_millis(200), async {
            let _ = handle.halt().await;
            let _ = handle.resume().await;
            let _ = handle.reset().await;
            let _ = handle.read_memory_batch(vec![]).await;
            let _ = handle
                .read_rtos_snapshot(sample_rtos_addrs(), TcbLayout::default())
                .await;
            handle.shutdown();
        })
        .await;
        assert!(
            result.is_ok(),
            "all commands must complete without blocking"
        );
    }
}
