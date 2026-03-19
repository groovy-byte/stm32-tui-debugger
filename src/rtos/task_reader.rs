use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tracing::{debug, warn};

use crate::probe::worker::{ProbeHandle, RtosAddrs};
use crate::symbols::SymbolEngine;
use super::types::*;

const MAX_PRIORITIES: u32 = 56;

pub struct TaskReader {
    layout: TcbLayout,
    poll_rate_hz: u32,
}

impl TaskReader {
    pub fn new(poll_rate_hz: u32) -> Self {
        Self {
            layout: TcbLayout::default(),
            poll_rate_hz,
        }
    }

    pub fn with_layout(mut self, layout: TcbLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn resolve_addrs(symbols: &SymbolEngine) -> RtosAddrs {
        let current_tcb = symbols
            .resolve_variable("pxCurrentTCB")
            .map(|v| v.address)
            .unwrap_or(0);

        let num_tasks = symbols
            .resolve_variable("uxCurrentNumberOfTasks")
            .map(|v| v.address)
            .ok();

        let ready_lists = symbols
            .resolve_variable("pxReadyTasksLists")
            .map(|v| v.address)
            .ok();

        let delayed1 = symbols
            .resolve_variable("xDelayedTaskList1")
            .map(|v| v.address)
            .ok();

        let delayed2 = symbols
            .resolve_variable("xDelayedTaskList2")
            .map(|v| v.address)
            .ok();

        let suspended = symbols
            .resolve_variable("xSuspendedTaskList")
            .map(|v| v.address)
            .ok();

        RtosAddrs {
            current_tcb,
            num_tasks,
            ready_lists,
            delayed1,
            delayed2,
            suspended,
            max_priorities: MAX_PRIORITIES,
        }
    }

    pub async fn run(
        self,
        probe: ProbeHandle,
        addrs: RtosAddrs,
        tx: mpsc::Sender<RtosSnapshot>,
        mut shutdown: watch::Receiver<bool>,
    ) {
        let period = Duration::from_millis(1000 / self.poll_rate_hz.max(1) as u64);
        let mut ticker = tokio::time::interval(period);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let snapshot = match probe.read_rtos_snapshot(addrs.clone(), self.layout.clone()).await {
                        Ok(snap) => snap,
                        Err(e) => {
                            warn!("RTOS snapshot failed: {}", e);
                            RtosSnapshot {
                                current_task_addr: 0,
                                total_tasks: 0,
                                tasks: Vec::new(),
                                error: Some(e.to_string()),
                            }
                        }
                    };
                    let _ = tx.send(snapshot).await;
                }
                _ = shutdown.changed() => {
                    debug!("RTOS poller shutdown");
                    break;
                }
            }
        }
    }
}
