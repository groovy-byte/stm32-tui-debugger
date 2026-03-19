use std::path::Path;
use std::time::Duration;

use probe_rs::{Permissions, Session};
use probe_rs::flashing;
use probe_rs::probe::list::Lister;
use tracing::{debug, error, info};

use crate::error::ProbeError;

/// Target execution state as observed by the debugger.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetState {
    Disconnected,
    Connected,
    Running,
    Halted,
    Error(String),
}

/// Manages a probe-rs debug session and exposes high-level target control.
pub struct DebugSession {
    session: Option<Session>,
    target_chip: String,
    state: TargetState,
}

impl DebugSession {
    /// Create a new disconnected session targeting the given chip (e.g. `"STM32F411CEUx"`).
    pub fn new(target_chip: &str) -> Self {
        Self {
            session: None,
            target_chip: target_chip.to_owned(),
            state: TargetState::Disconnected,
        }
    }

    /// Attach to the first available probe for the configured chip.
    pub fn connect(&mut self) -> Result<(), ProbeError> {
        info!(chip = %self.target_chip, "Connecting to target");

        // Try attaching under-reset first, then normal attach
        let perms = Permissions::new().allow_erase_all();

        let session = Session::auto_attach(&self.target_chip, perms)
            .or_else(|_| {
                info!("auto_attach failed, trying manual probe attach");
                let lister = Lister::new();
                let probes = lister.list_all();
                if probes.is_empty() {
                    return Err(probe_rs::Error::UnableToOpenProbe("No probes found"));
                }
                let probe = probes[0].open(&lister).map_err(|e| {
                    probe_rs::Error::Probe(e)
                })?;
                probe.attach(&self.target_chip, Permissions::new().allow_erase_all())
            })
            .map_err(|e| {
                let msg = format!("{e:#}");
                error!(error = %msg, "Connection failed");
                self.state = TargetState::Error(msg.clone());
                ProbeError::ConnectionFailed(msg)
            })?;

        self.session = Some(session);
        self.state = TargetState::Connected;
        info!("Successfully connected");
        Ok(())
    }

    /// Disconnect from the probe, dropping the inner session.
    pub fn disconnect(&mut self) {
        debug!("Disconnecting from target");
        self.session.take();
        self.state = TargetState::Disconnected;
    }

    /// Halt core 0 of the target.
    pub fn halt(&mut self) -> Result<(), ProbeError> {
        let session = self.session.as_mut().ok_or(ProbeError::NoSession)?;
        let mut core = session.core(0).map_err(|e| {
            ProbeError::ConnectionFailed(format!("Failed to access core 0: {e:#}"))
        })?;

        core.halt(Duration::from_millis(100)).map_err(|e| {
            ProbeError::ConnectionFailed(format!("Halt failed: {e:#}"))
        })?;

        self.state = TargetState::Halted;
        debug!("Core halted");
        Ok(())
    }

    /// Resume execution on core 0.
    pub fn resume(&mut self) -> Result<(), ProbeError> {
        let session = self.session.as_mut().ok_or(ProbeError::NoSession)?;
        let mut core = session.core(0).map_err(|e| {
            ProbeError::ConnectionFailed(format!("Failed to access core 0: {e:#}"))
        })?;

        core.run().map_err(|e| {
            ProbeError::ConnectionFailed(format!("Resume failed: {e:#}"))
        })?;

        self.state = TargetState::Running;
        debug!("Core resumed");
        Ok(())
    }

    /// Reset the target (halt after reset).
    pub fn reset(&mut self) -> Result<(), ProbeError> {
        let session = self.session.as_mut().ok_or(ProbeError::NoSession)?;
        let mut core = session.core(0).map_err(|e| {
            ProbeError::ConnectionFailed(format!("Failed to access core 0: {e:#}"))
        })?;

        core.reset_and_halt(Duration::from_millis(100)).map_err(|e| {
            ProbeError::ConnectionFailed(format!("Reset failed: {e:#}"))
        })?;

        self.state = TargetState::Halted;
        info!("Target reset and halted");
        Ok(())
    }

    /// Current target state.
    pub fn state(&self) -> &TargetState {
        &self.state
    }

    /// Returns `true` when an active session exists.
    pub fn is_connected(&self) -> bool {
        self.session.is_some()
    }

    /// Borrow core 0 from the active session.
    pub fn core(&mut self) -> Result<probe_rs::Core<'_>, ProbeError> {
        let session = self.session.as_mut().ok_or(ProbeError::NoSession)?;
        session.core(0).map_err(|e| {
            ProbeError::ConnectionFailed(format!("Failed to access core 0: {e:#}"))
        })
    }

    /// Flash an ELF file to the target, then reset.
    pub fn flash(&mut self, elf_path: &Path) -> Result<(), ProbeError> {
        let session = self.session.as_mut().ok_or(ProbeError::NoSession)?;
        info!(path = %elf_path.display(), "Flashing firmware");

        flashing::download_file(session, elf_path, flashing::Format::Elf)
            .map_err(|e| ProbeError::FlashFailed(format!("{e:#}")))?;

        // Reset and run after flashing
        let mut core = session.core(0).map_err(|e| {
            ProbeError::FlashFailed(format!("Post-flash core access failed: {e:#}"))
        })?;
        core.reset_and_halt(Duration::from_millis(100)).map_err(|e| {
            ProbeError::FlashFailed(format!("Post-flash reset failed: {e:#}"))
        })?;
        core.run().map_err(|e| {
            ProbeError::FlashFailed(format!("Post-flash resume failed: {e:#}"))
        })?;

        info!("Flash complete, target running");
        Ok(())
    }
}
