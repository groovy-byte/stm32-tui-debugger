use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    event::DisableMouseCapture,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::{mpsc, watch};

use stm32_tui_debugger::{
    config::{Cli, Config},
    poller::{PollerEngine, PollResult},
    probe::ProbeHandle,
    rtos::RtosSnapshot,
    svd::{PeripheralRegistry, SvdDevice, SvdFetcher},
    symbols::SymbolEngine,
    tui::{AppEvent, Command, EventReader, PaneId, TuiState},
};

#[tokio::main]
async fn main() -> Result<()> {
    // ── Parse CLI & build config ──────────────────────────────────────
    let cli = Cli::parse();
    let config = Config::from_cli(&cli)?;

    // ── Setup terminal ────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_result = run_app(&mut terminal, config).await;

    // ── Restore terminal (unconditional) ──────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(ref e) = app_result {
        eprintln!("STM32 TUI Debugger exited with error: {e:#}");
    } else {
        println!("STM32 TUI Debugger exited.");
    }

    app_result
}

/// Core application logic, separated so that terminal cleanup always runs
/// even if an error occurs.
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: Config,
) -> Result<()> {
    // ── TUI state ─────────────────────────────────────────────────────
    let mut tui_state = TuiState::new();
    tui_state.console_state.lines.push_back(format!(
        "[INFO] STM32 TUI Debugger starting — target: {}",
        config.target_chip
    ));

    // ── Load symbols (optional) ───────────────────────────────────────
    let symbols = if config.elf_path.exists() {
        match SymbolEngine::load(&config.elf_path) {
            Ok(engine) => {
                let var_count = engine.elf.variables.len();
                let func_count = engine.elf.functions.len();
                tui_state.console_state.lines.push_back(format!(
                    "[INFO] Loaded ELF: {var_count} variables, {func_count} functions"
                ));
                Some(Arc::new(engine))
            }
            Err(e) => {
                tui_state
                    .console_state
                    .lines
                    .push_back(format!("[WARN] Failed to load ELF: {e}"));
                None
            }
        }
    } else {
        tui_state.console_state.lines.push_back(format!(
            "[WARN] ELF file not found: {}",
            config.elf_path.display()
        ));
        None
    };

    // ── Connect probe (optional) ──────────────────────────────────────
    let (probe, probe_status) = ProbeHandle::spawn(&config.target_chip);
    tui_state.console_state.lines.push_back(probe_status);

    // ── Load SVD (optional) ───────────────────────────────────────────
    let _svd_registry = {
        let fetcher = SvdFetcher::new(config.svd_cache_dir.clone());
        match fetcher
            .get_svd_path(&config.target_chip)
            .and_then(|path| SvdDevice::parse_file(&path))
        {
            Ok(device) => {
                let registry = PeripheralRegistry::from_device(device);
                tui_state.peripheral_state.peripherals = registry.list_peripherals();
                tui_state.console_state.lines.push_back(format!(
                    "[INFO] Loaded SVD: {} peripherals",
                    tui_state.peripheral_state.peripherals.len()
                ));
                Some(Arc::new(registry))
            }
            Err(e) => {
                tui_state
                    .console_state
                    .lines
                    .push_back(format!("[WARN] SVD load failed: {e}"));
                None
            }
        }
    };

    // ── Shutdown signal ───────────────────────────────────────────────
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // ── Spawn poller task ─────────────────────────────────────────────
    let (poll_tx, mut poll_rx) = mpsc::channel::<PollResult>(100);
    if let Some(ref syms) = symbols {
        let mut poller = PollerEngine::new(config.poll_rate_hz);
        poller.resolve_symbols(syms);
        let probe_clone = probe.clone();
        let shutdown_rx_poller = shutdown_rx.clone();
        tokio::spawn(async move {
            poller
                .run(probe_clone, poll_tx, shutdown_rx_poller)
                .await;
        });
    }

    // ── Spawn RTT task ────────────────────────────────────────────────
    let (rtt_tx, mut rtt_rx) = mpsc::channel::<String>(200);
    {
        let shutdown_rx_rtt = shutdown_rx.clone();
        tokio::spawn(async move {
            stm32_tui_debugger::rtt::streaming::rtt_log_task(rtt_tx, shutdown_rx_rtt).await;
        });
    }

    // ── Spawn RTOS task reader ─────────────────────────────────────────
    let (rtos_tx, mut rtos_rx) = mpsc::channel::<RtosSnapshot>(50);
    if let Some(ref syms) = symbols {
        let task_reader =
            stm32_tui_debugger::rtos::TaskReader::new(config.poll_rate_hz.min(5));
        let rtos_addrs = stm32_tui_debugger::rtos::TaskReader::resolve_addrs(syms);
        let probe_clone = probe.clone();
        let shutdown_rx_rtos = shutdown_rx.clone();
        tokio::spawn(async move {
            task_reader
                .run(probe_clone, rtos_addrs, rtos_tx, shutdown_rx_rtos)
                .await;
        });
    }

    // ── Main event loop ───────────────────────────────────────────────
    loop {
        // Draw current state
        terminal.draw(|f| stm32_tui_debugger::tui::app::draw(f, &tui_state))?;

        // Multiplex input, poller results, and RTT lines
        tokio::select! {
            // User input (crossterm poll is blocking, so offload to a thread)
            event = tokio::task::spawn_blocking(|| {
                EventReader::poll(Duration::from_millis(16))
            }) => {
                if let Ok(Ok(Some(app_event))) = event {
                    match app_event {
                        AppEvent::Key(key_event) => {
                            let cmd = stm32_tui_debugger::tui::keybindings::map_key(
                                &key_event,
                                tui_state.focused,
                            );
                            if let Some(cmd) = cmd {
                                if handle_command(cmd, &mut tui_state, &probe).await {
                                    break; // Quit requested
                                }
                            }
                        }
                        AppEvent::Resize(_, _) => {} // ratatui handles resize
                        AppEvent::Tick => {}
                    }
                }
            }

            // Poller results → update expression entries
            Some(result) = poll_rx.recv() => {
                let value_str = format!("{}", result.value);
                if let Some(entry) = tui_state
                    .expression_state
                    .entries
                    .iter_mut()
                    .find(|e| e.name == result.expr_id)
                {
                    entry.value = value_str;
                    entry.changed = result.changed;
                }
                if let Some(err) = result.error {
                    tui_state
                        .console_state
                        .lines
                        .push_back(format!("[ERROR] {}: {err}", result.expr_id));
                }
            }

            // RTT log lines → console
            Some(line) = rtt_rx.recv() => {
                tui_state.console_state.lines.push_back(line);
                while tui_state.console_state.lines.len() > tui_state.console_state.max_lines {
                    tui_state.console_state.lines.pop_front();
                }
            }

            // RTOS snapshots → tasks pane
            Some(snapshot) = rtos_rx.recv() => {
                tui_state.tasks_state.rtos_detected = !snapshot.tasks.is_empty();
                tui_state.tasks_state.tasks = snapshot.tasks.iter().map(|t| {
                    stm32_tui_debugger::tui::state::TaskEntry {
                        name: t.name.clone(),
                        tcb_address: t.tcb_address,
                        priority: t.priority,
                        state: t.state.to_string(),
                        stack_top: t.stack_top,
                        stack_base: t.stack_base,
                    }
                }).collect();
                if let Some(err) = snapshot.error {
                    tui_state.console_state.lines.push_back(format!("[ERROR] RTOS: {err}"));
                }
            }
        }

        if !tui_state.running {
            break;
        }
    }

    // Signal all background tasks to stop
    let _ = shutdown_tx.send(true);

    probe.shutdown();

    Ok(())
}

/// Dispatch a `Command`, returning `true` when the app should quit.
async fn handle_command(
    cmd: Command,
    state: &mut TuiState,
    probe: &ProbeHandle,
) -> bool {
    match cmd {
        Command::Quit => return true,

        Command::NextPane => state.focused = state.focused.next(),
        Command::PrevPane => state.focused = state.focused.prev(),

        Command::ScrollUp => scroll_up(state),
        Command::ScrollDown => scroll_down(state),
        Command::PageUp => {
            for _ in 0..10 {
                scroll_up(state);
            }
        }
        Command::PageDown => {
            for _ in 0..10 {
                scroll_down(state);
            }
        }

        Command::HaltTarget => {
            match probe.halt().await {
                Ok(()) => state
                    .console_state
                    .lines
                    .push_back("[INFO] Target halted".into()),
                Err(e) => state
                    .console_state
                    .lines
                    .push_back(format!("[ERROR] Halt failed: {e}")),
            }
        }
        Command::ResumeTarget => {
            match probe.resume().await {
                Ok(()) => state
                    .console_state
                    .lines
                    .push_back("[INFO] Target resumed".into()),
                Err(e) => state
                    .console_state
                    .lines
                    .push_back(format!("[ERROR] Resume failed: {e}")),
            }
        }
        Command::ResetTarget => {
            match probe.reset().await {
                Ok(()) => state
                    .console_state
                    .lines
                    .push_back("[INFO] Target reset".into()),
                Err(e) => state
                    .console_state
                    .lines
                    .push_back(format!("[ERROR] Reset failed: {e}")),
            }
        }

        // Remaining commands are stubs for now
        Command::Select
        | Command::Back
        | Command::ToggleExpand
        | Command::AddExpression
        | Command::RemoveExpression
        | Command::StepOver
        | Command::StepInto => {}

        Command::RefreshTasks => {
            state.status_message = Some("RTOS tasks refreshing...".into());
        }
        Command::InspectTask => {
            if let Some(idx) = state.tasks_state.selected {
                if let Some(task) = state.tasks_state.tasks.get(idx) {
                    state.console_state.lines.push_back(format!(
                        "[INFO] Task '{}' \u{2014} TCB: 0x{:08x}, Priority: {}, State: {}, Stack: 0x{:08x}..0x{:08x}",
                        task.name, task.tcb_address, task.priority, task.state, task.stack_base, task.stack_top
                    ));
                }
            }
        }
    }
    false
}

fn scroll_up(state: &mut TuiState) {
    match state.focused {
        PaneId::Source => {
            state.source_state.scroll_offset =
                state.source_state.scroll_offset.saturating_sub(1);
        }
        PaneId::Peripherals => {
            if let Some(ref mut sel) = state.peripheral_state.selected_peripheral {
                *sel = sel.saturating_sub(1);
            } else if !state.peripheral_state.peripherals.is_empty() {
                state.peripheral_state.selected_peripheral = Some(0);
            }
        }
        PaneId::Tasks => {
            if let Some(ref mut sel) = state.tasks_state.selected {
                *sel = sel.saturating_sub(1);
            } else if !state.tasks_state.tasks.is_empty() {
                state.tasks_state.selected = Some(0);
            }
        }
        PaneId::Expressions => {
            if let Some(ref mut sel) = state.expression_state.selected {
                *sel = sel.saturating_sub(1);
            }
        }
        PaneId::Console => {
            state.console_state.auto_scroll = false;
            state.console_state.scroll_offset =
                state.console_state.scroll_offset.saturating_sub(1);
        }
    }
}

fn scroll_down(state: &mut TuiState) {
    match state.focused {
        PaneId::Source => {
            if state.source_state.scroll_offset + 1 < state.source_state.lines.len() {
                state.source_state.scroll_offset += 1;
            }
        }
        PaneId::Peripherals => {
            let max = state
                .peripheral_state
                .peripherals
                .len()
                .saturating_sub(1);
            if let Some(ref mut sel) = state.peripheral_state.selected_peripheral {
                if *sel < max {
                    *sel += 1;
                }
            } else if !state.peripheral_state.peripherals.is_empty() {
                state.peripheral_state.selected_peripheral = Some(0);
            }
        }
        PaneId::Tasks => {
            let max = state
                .tasks_state
                .tasks
                .len()
                .saturating_sub(1);
            if let Some(ref mut sel) = state.tasks_state.selected {
                if *sel < max {
                    *sel += 1;
                }
            } else if !state.tasks_state.tasks.is_empty() {
                state.tasks_state.selected = Some(0);
            }
        }
        PaneId::Expressions => {
            let max = state
                .expression_state
                .entries
                .len()
                .saturating_sub(1);
            if let Some(ref mut sel) = state.expression_state.selected {
                if *sel < max {
                    *sel += 1;
                }
            } else if !state.expression_state.entries.is_empty() {
                state.expression_state.selected = Some(0);
            }
        }
        PaneId::Console => {
            state.console_state.scroll_offset += 1;
        }
    }
}
