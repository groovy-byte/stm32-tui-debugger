//! Dedicated probe worker thread — all USB I/O runs here, never on the async
//! runtime.  The rest of the application communicates through [`ProbeHandle`],
//! which uses a dual-channel pattern: control commands (halt/resume/reset) are
//! always processed before read commands, so the UI never waits behind a long
//! RTOS snapshot scan.

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tokio::sync::oneshot;
use tracing::debug;

use crate::error::ProbeError;
use crate::poller::ValueFormat;
use crate::probe::{DebugSession, MemoryAccess};
use crate::rtos::{list_walker, tcb, RtosSnapshot, TaskState, TcbLayout};

// ── Public types ──────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct RtosAddrs {
    pub current_tcb: u32,
    pub num_tasks: Option<u32>,
    pub ready_lists: Option<u32>,
    pub delayed1: Option<u32>,
    pub delayed2: Option<u32>,
    pub suspended: Option<u32>,
    pub max_priorities: u32,
}

#[derive(Clone, Debug)]
pub struct MemReadRequest {
    pub id: String,
    pub addr: u32,
    pub size: usize,
    pub format: ValueFormat,
}

pub type MemReadResult = (String, ValueFormat, Result<Vec<u8>, String>);

// ── Internal command enums (split by priority) ────────────────────────────

enum CtrlCmd {
    Halt(oneshot::Sender<Result<(), ProbeError>>),
    Resume(oneshot::Sender<Result<(), ProbeError>>),
    Reset(oneshot::Sender<Result<(), ProbeError>>),
    Flash(PathBuf, oneshot::Sender<Result<(), ProbeError>>),
    Shutdown,
}

enum ReadCmd {
    MemoryBatch {
        requests: Vec<MemReadRequest>,
        reply: oneshot::Sender<Vec<MemReadResult>>,
    },
    RtosSnapshot {
        addrs: RtosAddrs,
        layout: TcbLayout,
        reply: oneshot::Sender<Result<RtosSnapshot, ProbeError>>,
    },
}

// ── ProbeHandle (async, cloneable, Send) ──────────────────────────────────

#[derive(Clone)]
pub struct ProbeHandle {
    ctrl_tx: mpsc::Sender<CtrlCmd>,
    read_tx: mpsc::Sender<ReadCmd>,
}

impl ProbeHandle {
    pub fn spawn(chip: &str) -> (Self, String) {
        let (ctrl_tx, ctrl_rx) = mpsc::channel();
        let (read_tx, read_rx) = mpsc::channel();
        let (init_tx, init_rx) = std::sync::mpsc::channel();
        let chip = chip.to_string();

        thread::Builder::new()
            .name("probe-worker".into())
            .spawn(move || worker_main(&chip, ctrl_rx, read_rx, init_tx))
            .expect("failed to spawn probe-worker thread");

        let status = init_rx
            .recv()
            .unwrap_or_else(|_| "Probe thread crashed on startup".into());

        (ProbeHandle { ctrl_tx, read_tx }, status)
    }

    pub async fn halt(&self) -> Result<(), ProbeError> {
        let (tx, rx) = oneshot::channel();
        self.ctrl_tx
            .send(CtrlCmd::Halt(tx))
            .map_err(|_| ProbeError::NoSession)?;
        rx.await.map_err(|_| ProbeError::NoSession)?
    }

    pub async fn resume(&self) -> Result<(), ProbeError> {
        let (tx, rx) = oneshot::channel();
        self.ctrl_tx
            .send(CtrlCmd::Resume(tx))
            .map_err(|_| ProbeError::NoSession)?;
        rx.await.map_err(|_| ProbeError::NoSession)?
    }

    pub async fn reset(&self) -> Result<(), ProbeError> {
        let (tx, rx) = oneshot::channel();
        self.ctrl_tx
            .send(CtrlCmd::Reset(tx))
            .map_err(|_| ProbeError::NoSession)?;
        rx.await.map_err(|_| ProbeError::NoSession)?
    }

    pub async fn flash(&self, elf_path: PathBuf) -> Result<(), ProbeError> {
        let (tx, rx) = oneshot::channel();
        self.ctrl_tx
            .send(CtrlCmd::Flash(elf_path, tx))
            .map_err(|_| ProbeError::NoSession)?;
        rx.await.map_err(|_| ProbeError::NoSession)?
    }

    pub async fn read_memory_batch(&self, requests: Vec<MemReadRequest>) -> Vec<MemReadResult> {
        let (tx, rx) = oneshot::channel();
        if self
            .read_tx
            .send(ReadCmd::MemoryBatch {
                requests,
                reply: tx,
            })
            .is_err()
        {
            return Vec::new();
        }
        rx.await.unwrap_or_default()
    }

    pub async fn read_rtos_snapshot(
        &self,
        addrs: RtosAddrs,
        layout: TcbLayout,
    ) -> Result<RtosSnapshot, ProbeError> {
        let (tx, rx) = oneshot::channel();
        self.read_tx
            .send(ReadCmd::RtosSnapshot {
                addrs,
                layout,
                reply: tx,
            })
            .map_err(|_| ProbeError::NoSession)?;
        rx.await.map_err(|_| ProbeError::NoSession)?
    }

    pub fn shutdown(&self) {
        let _ = self.ctrl_tx.send(CtrlCmd::Shutdown);
    }
}

// ── Worker thread ─────────────────────────────────────────────────────────

fn worker_main(
    chip: &str,
    ctrl_rx: mpsc::Receiver<CtrlCmd>,
    read_rx: mpsc::Receiver<ReadCmd>,
    init_tx: mpsc::Sender<String>,
) {
    let mut session = DebugSession::new(chip);
    let status = match session.connect() {
        Ok(()) => "[INFO] Probe connected".into(),
        Err(e) => format!("[WARN] Probe not connected: {e}"),
    };
    let _ = init_tx.send(status);

    loop {
        // 1. Always drain control commands first (high priority, non-blocking)
        loop {
            match ctrl_rx.try_recv() {
                Ok(CtrlCmd::Halt(reply)) => {
                    let _ = reply.send(session.halt());
                }
                Ok(CtrlCmd::Resume(reply)) => {
                    let _ = reply.send(session.resume());
                }
                Ok(CtrlCmd::Reset(reply)) => {
                    let _ = reply.send(session.reset());
                }
                Ok(CtrlCmd::Flash(path, reply)) => {
                    let _ = reply.send(session.flash(&path));
                }
                Ok(CtrlCmd::Shutdown) => {
                    debug!("Probe worker shutting down");
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => return,
            }
        }

        // 2. Process one read command (with short timeout so we re-check ctrl)
        match read_rx.recv_timeout(Duration::from_millis(10)) {
            Ok(ReadCmd::MemoryBatch { requests, reply }) => {
                let results = do_memory_batch(&mut session, requests);
                let _ = reply.send(results);
            }
            Ok(ReadCmd::RtosSnapshot {
                addrs,
                layout,
                reply,
            }) => {
                let result = do_rtos_snapshot(&mut session, &addrs, &layout);
                let _ = reply.send(result);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

// ── Probe-thread helpers (synchronous, blocking USB I/O is fine here) ────

fn do_memory_batch(
    session: &mut DebugSession,
    requests: Vec<MemReadRequest>,
) -> Vec<MemReadResult> {
    let mut core = match session.core() {
        Ok(c) => c,
        Err(_) => {
            return requests
                .into_iter()
                .map(|r| (r.id, r.format, Err("No active session".into())))
                .collect();
        }
    };

    requests
        .into_iter()
        .map(|r| {
            let result = MemoryAccess::read_u8(&mut core, r.addr, r.size)
                .map_err(|e| e.to_string());
            (r.id, r.format, result)
        })
        .collect()
}

const LIST_T_SIZE: u32 = 20;

fn do_rtos_snapshot(
    session: &mut DebugSession,
    addrs: &RtosAddrs,
    layout: &TcbLayout,
) -> Result<RtosSnapshot, ProbeError> {
    let mut core = session.core()?;

    // 1. pxCurrentTCB
    let current_tcb_addr = MemoryAccess::read_u32(&mut core, addrs.current_tcb)?;

    // 2. uxCurrentNumberOfTasks
    let total_tasks = addrs
        .num_tasks
        .and_then(|a| MemoryAccess::read_u32(&mut core, a).ok())
        .unwrap_or(0);

    let mut tasks = Vec::new();

    // 3. Current task
    if current_tcb_addr != 0 {
        if let Ok(t) = tcb::read_tcb(&mut core, current_tcb_addr, layout, TaskState::Running) {
            tasks.push(t);
        }
    }

    // 4. Ready lists — early exit once we've found all tasks
    if let Some(ready_base) = addrs.ready_lists {
        for pri in 0..addrs.max_priorities {
            if total_tasks > 0 && tasks.len() as u32 >= total_tasks {
                break;
            }
            let list_addr = ready_base + pri * LIST_T_SIZE;
            match list_walker::walk_list(&mut core, list_addr) {
                Ok(addrs) => {
                    for addr in addrs {
                        if addr == current_tcb_addr {
                            continue;
                        }
                        if let Ok(t) = tcb::read_tcb(&mut core, addr, layout, TaskState::Ready) {
                            tasks.push(t);
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }

    // 5. Delayed lists
    for delayed_addr in [addrs.delayed1, addrs.delayed2].iter().flatten() {
        if let Ok(tcb_addrs) = list_walker::walk_list(&mut core, *delayed_addr) {
            for addr in tcb_addrs {
                if tasks.iter().any(|t| t.tcb_address == addr) {
                    continue;
                }
                if let Ok(t) = tcb::read_tcb(&mut core, addr, layout, TaskState::Blocked) {
                    tasks.push(t);
                }
            }
        }
    }

    // 6. Suspended list
    if let Some(susp) = addrs.suspended {
        if let Ok(tcb_addrs) = list_walker::walk_list(&mut core, susp) {
            for addr in tcb_addrs {
                if tasks.iter().any(|t| t.tcb_address == addr) {
                    continue;
                }
                if let Ok(t) = tcb::read_tcb(&mut core, addr, layout, TaskState::Suspended) {
                    tasks.push(t);
                }
            }
        }
    }

    tasks.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.name.cmp(&b.name)));

    Ok(RtosSnapshot {
        current_task_addr: current_tcb_addr,
        total_tasks,
        tasks,
        error: None,
    })
}

// ── Test helpers ──────────────────────────────────────────────────────────

#[cfg(any(test, feature = "test-support"))]
impl ProbeHandle {
    /// Create a handle whose worker is already dead (for testing error paths).
    pub fn dead() -> Self {
        let (ctrl_tx, _ctrl_rx) = std::sync::mpsc::channel();
        let (read_tx, _read_rx) = std::sync::mpsc::channel();
        ProbeHandle { ctrl_tx, read_tx }
    }
}
