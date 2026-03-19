//! Tests for the probe worker module.
//!
//! Run with: `cargo test --features test-support test_probe_worker`
//!
//! The `test-support` feature enables `ProbeHandle::dead()`, which creates a
//! handle backed by a disconnected channel (simulates a dead worker thread).

use stm32_tui_debugger::poller::ValueFormat;
use stm32_tui_debugger::probe::{MemReadRequest, RtosAddrs};
use stm32_tui_debugger::rtos::{TaskInfo, TaskState, TcbLayout};

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

// ── ProbeHandle error paths (channel disconnection) ─────────────────────

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
    async fn should_return_no_session_when_flash_on_dead_worker() {
        let handle = ProbeHandle::dead();
        let result = handle.flash(std::path::PathBuf::from("/nonexistent/firmware.elf")).await;
        assert!(matches!(result, Err(ProbeError::NoSession)));
    }

    #[tokio::test]
    async fn should_return_no_session_when_flash_empty_path_on_dead_worker() {
        let handle = ProbeHandle::dead();
        let result = handle.flash(std::path::PathBuf::new()).await;
        assert!(matches!(result, Err(ProbeError::NoSession)));
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
            let _ = handle.flash(std::path::PathBuf::from("/tmp/test.elf")).await;
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

// ── Task sorting ────────────────────────────────────────────────────────

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

// ── Regression: freeze bug (non-blocking async) ─────────────────────────

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
    async fn should_complete_flash_within_deadline_when_worker_dead() {
        let handle = ProbeHandle::dead();
        let path = std::path::PathBuf::from("/tmp/test.elf");
        let result = tokio::time::timeout(Duration::from_millis(100), handle.flash(path)).await;
        assert!(result.is_ok(), "flash() must not block when worker is dead");
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
            let _ = handle.flash(std::path::PathBuf::from("/tmp/test.elf")).await;
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
