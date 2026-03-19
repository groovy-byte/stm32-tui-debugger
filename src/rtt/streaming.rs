use std::collections::VecDeque;
use tokio::sync::mpsc;
use tracing::debug;

pub struct RttBuffer {
    lines: VecDeque<String>,
    max_lines: usize,
}

impl RttBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
        }
    }

    pub fn push_line(&mut self, line: String) {
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines.iter()
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

/// RTT log task placeholder — will be fully implemented when probe-rs RTT API
/// is integrated. For now it simply keeps the task alive until shutdown.
pub async fn rtt_log_task(
    _tx: mpsc::Sender<String>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    debug!("RTT log task started (stub — waiting for probe-rs RTT integration)");
    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                debug!("RTT task shutdown");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // When RTT is connected, this will read from probe-rs RTT channels
                // and forward lines through tx
            }
        }
    }
}
