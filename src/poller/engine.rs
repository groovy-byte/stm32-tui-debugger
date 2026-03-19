use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, warn};

use crate::probe::worker::{ProbeHandle, MemReadRequest};
use crate::symbols::SymbolEngine;
use super::types::{DisplayValue, Expression, PollResult, ValueFormat};

pub struct PollerEngine {
    expressions: Vec<Expression>,
    last_values: HashMap<String, DisplayValue>,
    poll_rate_hz: u32,
}

impl PollerEngine {
    pub fn new(poll_rate_hz: u32) -> Self {
        Self {
            expressions: Vec::new(),
            last_values: HashMap::new(),
            poll_rate_hz,
        }
    }

    pub fn add_expression(&mut self, expr: Expression) {
        self.expressions.push(expr);
    }

    pub fn remove_expression(&mut self, id: &str) {
        self.expressions.retain(|e| e.id != id);
        self.last_values.remove(id);
    }

    pub fn list_expressions(&self) -> &[Expression] {
        &self.expressions
    }

    /// Pre-resolve all expression addresses using the symbol engine.
    /// Call this before `run()` so the `SymbolEngine` (which is `!Send`)
    /// doesn't need to cross a spawn boundary.
    pub fn resolve_symbols(&mut self, symbols: &SymbolEngine) {
        for expr in &mut self.expressions {
            if let Err(e) = expr.resolve(symbols) {
                warn!(expr = %expr.name, error = %e, "Failed to resolve expression");
            }
        }
    }

    /// Run the polling loop. Sends `PollResult`s through the channel until
    /// shutdown is signalled.
    pub async fn run(
        mut self,
        probe: ProbeHandle,
        tx: mpsc::Sender<PollResult>,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        let period = Duration::from_millis(1000 / self.poll_rate_hz.max(1) as u64);
        let mut ticker = interval(period);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.poll_once(&probe, &tx).await;
                }
                _ = shutdown.changed() => {
                    debug!("Poller shutdown requested");
                    break;
                }
            }
        }
    }

    async fn poll_once(
        &mut self,
        probe: &ProbeHandle,
        tx: &mpsc::Sender<PollResult>,
    ) {
        let requests: Vec<MemReadRequest> = self
            .expressions
            .iter()
            .filter_map(|expr| {
                let addr = expr.address?;
                let size = expr.size?;
                Some(MemReadRequest {
                    id: expr.id.clone(),
                    addr,
                    size,
                    format: expr.format,
                })
            })
            .collect();

        if requests.is_empty() {
            return;
        }

        let readings = probe.read_memory_batch(requests).await;

        let now = Instant::now();
        for (id, format, result) in readings {
            let (value, error) = match result {
                Ok(bytes) => (decode_bytes(&bytes, format), None),
                Err(e) => (DisplayValue::Error(e.clone()), Some(e)),
            };

            let changed = self
                .last_values
                .get(&id)
                .map_or(true, |prev| *prev != value);

            if error.is_none() {
                self.last_values.insert(id.clone(), value.clone());
            }

            let result = PollResult {
                expr_id: id,
                value,
                changed,
                timestamp: now,
                error,
            };

            if tx.send(result).await.is_err() {
                debug!("Poll result receiver dropped, stopping");
                return;
            }
        }
    }
}

fn decode_bytes(bytes: &[u8], format: ValueFormat) -> DisplayValue {
    match bytes.len() {
        1 => DisplayValue::Unsigned(bytes[0] as u64),
        2 => {
            let val = u16::from_le_bytes([bytes[0], bytes[1]]);
            DisplayValue::Unsigned(val as u64)
        }
        4 => {
            let val = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            match format {
                ValueFormat::Float => DisplayValue::Float(f32::from_bits(val) as f64),
                _ => DisplayValue::Unsigned(val as u64),
            }
        }
        8 => {
            let raw: [u8; 8] = bytes[0..8].try_into().unwrap();
            let val = u64::from_le_bytes(raw);
            match format {
                ValueFormat::Float => DisplayValue::Float(f64::from_bits(val)),
                _ => DisplayValue::Unsigned(val),
            }
        }
        _ => DisplayValue::Bytes(bytes.to_vec()),
    }
}
