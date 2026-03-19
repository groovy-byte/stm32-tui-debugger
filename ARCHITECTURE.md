# Architecture

This document describes the architecture of `stm32-tui-debugger`: the module
responsibilities, data flow, async task model, and key design decisions.

## System Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                         Terminal (crossterm)                      │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    TUI Layer (ratatui)                      │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐   │  │
│  │  │ Source Pane   │ │ Periph Pane  │ │ Tasks Pane       │   │  │
│  │  └──────────────┘ └──────────────┘ └──────────────────┘   │  │
│  │  ┌──────────────┐ ┌──────────────────────────────────┐    │  │
│  │  │ Expression   │ │ Console / RTT Pane               │    │  │
│  │  │ Pane         │ │                                  │    │  │
│  │  └──────────────┘ └──────────────────────────────────┘    │  │
│  └───────────────────────┬────────────────────────────────────┘  │
│                          │ AppEvent / Command                    │
│  ┌───────────────────────▼────────────────────────────────────┐  │
│  │                   Application Core                         │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐ │  │
│  │  │ Config   │  │ Symbols  │  │ SVD      │  │ Poller    │ │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └───────────┘ │  │
│  │  ┌──────────┐                                            │  │
│  │  │ RTOS     │                                            │  │
│  │  └──────────┘                                            │  │
│  └───────────────────────┬────────────────────────────────────┘  │
│                          │                                       │
│  ┌───────────────────────▼────────────────────────────────────┐  │
│  │              probe-rs (DebugSession + MemoryAccess)         │  │
│  └───────────────────────┬────────────────────────────────────┘  │
│                          │ SWD / JTAG                            │
└──────────────────────────┼───────────────────────────────────────┘
                           │
                   ┌───────▼───────┐
                   │   ST-Link     │
                   │   v2 / v3     │
                   └───────┬───────┘
                           │
                   ┌───────▼───────┐
                   │  STM32 Target │
                   └───────────────┘
```

## Module Responsibilities

| Module | Purpose | Key Types |
|:-------|:--------|:----------|
| `config` | CLI argument parsing (clap) and TOML config loading | `Cli`, `Config` |
| `error` | Typed error hierarchy with per-module variants | `AppError`, `ProbeError`, `SymbolError`, `SvdError`, `PollerError`, `TuiError` |
| `probe::session` | Manages a `probe_rs::Session` — connect, halt, resume, reset | `DebugSession`, `TargetState` |
| `probe::memory` | Stateless helpers for typed memory reads/writes through a `probe_rs::Core` | `MemoryAccess` |
| `symbols::elf_loader` | Parses ELF via the `object` crate; extracts variables, functions, section bases | `ElfData` |
| `symbols::dwarf_resolver` | Wraps `addr2line::Context` for address → source location and function name lookup | `DwarfResolver` |
| `symbols::demangler` | C++ Itanium-ABI demangling via `cpp_demangle` | `demangle()`, `is_mangled()` |
| `symbols::types` | Shared type definitions for symbol information | `Variable`, `SourceLocation`, `TypeInfo`, `TypeKind`, `FunctionInfo` |
| `symbols` (mod) | High-level facade combining ELF + DWARF | `SymbolEngine` |
| `svd::fetcher` | Downloads SVD files from `stm32-rs/stm32-rs` on GitHub and caches them locally | `SvdFetcher` |
| `svd::parser` | Parses SVD XML into an in-memory device model using `svd-parser` | `SvdDevice`, `PeripheralInfo` |
| `svd::registry` | Queryable index of peripherals/registers with bitfield decoding | `PeripheralRegistry`, `Register`, `BitField`, `RegisterValue`, `AccessType` |
| `poller::types` | Data types for expressions, display values, and poll results | `Expression`, `ValueFormat`, `PollResult`, `DisplayValue` |
| `poller::expression` | Expression construction, resolution against the symbol table | `Expression::resolve()` |
| `poller::engine` | Async polling loop — reads memory at a fixed rate and sends results via channel | `PollerEngine` |
| `rtt::streaming` | Ring-buffer for RTT log lines + async task stub | `RttBuffer`, `rtt_log_task()` |
| `rtos::types` | FreeRTOS data types — TCB layout, task state enum, snapshot | `TcbLayout`, `TaskState`, `TaskInfo`, `RtosSnapshot` |
| `rtos::task_reader` | Reads FreeRTOS task lists from target memory via ELF symbols | `TaskReader` |
| `rtos::tcb` | Parses individual Task Control Block structures | `read_tcb()` |
| `rtos::list_walker` | Walks FreeRTOS `List_t` linked lists in target memory | `walk_list()` |
| `tui::state` | State models for each pane and the overall TUI | `TuiState`, `PaneId`, `SourceState`, `PeripheralState`, `TasksState`, `ExpressionState`, `ConsoleState` |
| `tui::app` | Layout definition and top-level `draw()` function | `draw()` |
| `tui::event` | Crossterm event polling adapter | `AppEvent`, `EventReader` |
| `tui::keybindings` | Maps `KeyEvent` + focused pane to a `Command` enum | `Command`, `map_key()` |
| `tui::panes::*` | Per-pane rendering (source, peripherals, tasks, expressions, console) | `draw()` functions |
| `tui::widgets` | Placeholder for custom ratatui widgets | — |

## Async Task Architecture

The application runs on the **Tokio** runtime (`#[tokio::main]`). The task
topology is:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tokio Runtime                            │
│                                                                 │
│  ┌──────────────────┐          watch::channel (shutdown)        │
│  │   Main Task       │──────────────────────┐                   │
│  │  (TUI event loop) │                      │                   │
│  └────────┬─────────┘                       │                   │
│           │ spawns                           ▼                   │
│  ┌────────▼─────────┐  ┌───────────────┐  ┌───────────────┐    │
│  │  Poller Task      │  │  RTOS Poller  │  │  RTT Task     │    │
│  │  (PollerEngine    │  │  (TaskReader  │  │  (rtt_log_    │    │
│  │   ::run)          │  │   ::run)      │  │   task)       │    │
│  └────────┬─────────┘  └───────┬───────┘  └───────┬───────┘    │
│           │ mpsc::Sender       │ mpsc::Sender      │ mpsc::     │
│           │ <PollResult>       │ <RtosSnapshot>    │ Sender     │
│           │                    │                   │ <String>   │
│           ▼                    ▼                   ▼            │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                   TuiState (main task)                   │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  Shared state:                                                  │
│    Arc<Mutex<DebugSession>>  ← poller + RTOS poller borrow core │
│    Arc<SymbolEngine>         ← read-only after init             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Task Descriptions

| Task | Lifetime | Communication |
|:-----|:---------|:-------------|
| **Main** | Entire application | Owns `TuiState`; receives from poller, RTOS poller, and RTT via `mpsc` channels; sends shutdown via `watch` |
| **Poller** | Spawned after connect | Holds `Arc<Mutex<DebugSession>>` for memory reads; sends `PollResult` to main |
| **RTOS Poller** | Spawned after connect (if FreeRTOS detected) | Holds `Arc<Mutex<DebugSession>>` + `Arc<SymbolEngine>`; sends `RtosSnapshot` to main at 2–5 Hz |
| **RTT** | Spawned when `rtt_enabled` | Reads from probe-rs RTT channels (stub); sends log lines to main |

All tasks observe a shared `tokio::sync::watch<bool>` for graceful shutdown.

## Data Flow

### Startup

```
CLI args ──► Config::from_cli()
               ├── DebugSession::new(chip)
               ├── DebugSession::connect()
               ├── SymbolEngine::load(elf)
               ├── SvdFetcher::get_svd_path(chip) ──► SvdDevice::parse_file()
               │                                        └── PeripheralRegistry::from_device()
               └── TuiState::new()
```

### Polling Cycle (per tick)

```
PollerEngine::poll_once()
  │
  ├── session.lock()            acquire Mutex
  ├── session.core()            borrow probe_rs::Core
  ├── for each Expression:
  │     MemoryAccess::read_u8(core, addr, size)
  ├── drop(core), drop(guard)   release Mutex
  │
  └── for each reading:
        decode_bytes(raw, format) ──► DisplayValue
        compare with last_values  ──► changed flag
        tx.send(PollResult)       ──► main task
```

### Key Input

```
EventReader::poll()
  ──► AppEvent::Key(event)
  ──► keybindings::map_key(event, focused_pane)
  ──► Command enum
  ──► match Command {
        Quit         → shutdown
        NextPane     → state.focused = focused.next()
        HaltTarget   → session.halt()
        ScrollUp     → pane_state.scroll_offset -= 1
        AddExpression→ prompt + poller.add_expression()
        ...
      }
```

### SVD Register Read

```
PeripheralRegistry::get_register("GPIOA", "ODR")
  ──► Register { address: 0x4002_0014, width: 32, fields: [...] }

MemoryAccess::read_u32(core, 0x4002_0014)
  ──► raw_value: u32

PeripheralRegistry::decode_register(register, raw_value)
  ──► RegisterValue { field_values: [("ODR0", 1), ("ODR1", 0), ...] }
```

## Error Handling Strategy

Errors are modelled as a two-level hierarchy using `thiserror`:

```
AppError                          (top-level, in error.rs)
├── Probe(ProbeError)             connection, memory r/w, register access
├── Symbol(SymbolError)           ELF load, DWARF parse, variable lookup
├── Svd(SvdError)                 fetch, parse, peripheral/register lookup
├── Poller(PollerError)           expression resolution, evaluation
├── Tui(TuiError)                 terminal setup, rendering
├── Config(String)                TOML deserialization
└── Io(std::io::Error)            filesystem operations
```

Each module defines its own error enum with descriptive variants (e.g.
`ProbeError::MemoryReadFailed { addr, reason }`). This enables callers to
pattern-match on specific failure modes while `AppError` provides a single
`Result<T>` type for the application boundary.

`anyhow::Result` is used at the binary crate level (`main.rs`) for top-level
error reporting with backtraces.

## FreeRTOS Kernel Awareness

The `rtos` module implements non-intrusive FreeRTOS monitoring by reading
kernel data structures directly from target memory — no target-side agent or
instrumentation is needed.

### TCB Discovery via ELF Symbols

FreeRTOS stores its scheduler state in well-known global variables. The
`TaskReader` resolves these ELF symbols at startup via `SymbolEngine`:

| Symbol | Purpose |
|:-------|:--------|
| `pxCurrentTCB` | Pointer to the currently running task's TCB |
| `uxCurrentNumberOfTasks` | Total task count (validation) |
| `pxReadyTasksLists` | Array of `List_t` — one per priority (up to 56) |
| `xDelayedTaskList1` / `xDelayedTaskList2` | Delayed/blocked task lists |
| `xSuspendedTaskList` | Suspended task list |

If `pxCurrentTCB` is not found in the ELF, FreeRTOS is considered absent and
the Tasks pane shows a "not detected" message.

### Walking `List_t` Linked Lists

FreeRTOS maintains tasks in circular doubly-linked lists (`List_t`). Each
`List_t` is 20 bytes (5 × `u32`) on Cortex-M: item count, index pointer, and
the list-end sentinel. `list_walker::walk_list()` follows `pxNext` pointers
from the sentinel, reading each `ListItem_t` and extracting the TCB owner
pointer, until it loops back to the sentinel or exceeds the item count.

### TCB Memory Layout

Task Control Block fields are read at configurable byte offsets (`TcbLayout`).
The defaults match a standard Cortex-M FreeRTOS build **without** MPU support:

| Field | Offset | Size | Description |
|:------|-------:|-----:|:------------|
| `top_of_stack` | 0 | 4 | Current stack pointer |
| `state_list_item` | 4 | 20 | State list link (`ListItem_t`) |
| `event_list_item` | 24 | 20 | Event list link |
| `priority` | 44 | 4 | Task priority |
| `stack_base` | 48 | 4 | Stack base pointer |
| `task_name` | 52 | 16 | Null-terminated name |

MPU-enabled or hook-extended builds shift these offsets; supply a custom
`TcbLayout` via config or the builder API (`TaskReader::with_layout()`).

### Why Polling is Capped at 5 Hz

Each RTOS snapshot requires many individual memory reads: one for
`pxCurrentTCB`, one per priority level for the ready lists (up to 56), plus
list walks and per-task TCB reads. At 10+ tasks this easily exceeds 100
reads per snapshot. Polling faster than 5 Hz would monopolise the probe
mutex and starve expression polling and user commands. The 2–5 Hz range
provides responsive task-state updates without noticeable impact on the
rest of the debugger.

## Design Decisions

### Why probe-rs over OpenOCD / pyOCD

| Concern | probe-rs | OpenOCD |
|:--------|:---------|:--------|
| Language integration | Native Rust crate — links directly | Requires socket/pipe IPC |
| Non-intrusive reads | `core.read_word_32()` while target runs | Requires GDB RSP overhead |
| Startup latency | Single process, no daemon | Separate server process |
| Chip database | Built-in target DB, auto-download | Manual config files |

probe-rs gives us a single-process architecture with direct USB access to the
ST-Link. No external daemon, no GDB protocol overhead, no config files.

### Why ratatui over cursive / tui-rs / egui

- **ratatui** is the actively maintained fork of tui-rs and the de-facto
  standard for Rust TUI applications.
- Immediate-mode rendering model pairs naturally with the async poll loop —
  redraw the entire frame on every tick.
- Composable layout system (`Layout`, `Constraint`) makes the five-pane
  split straightforward.
- Large widget ecosystem (tables, lists, paragraphs, sparklines).

### Why gimli + addr2line over goblin / other DWARF parsers

- `gimli` is the most complete DWARF parser in the Rust ecosystem (used by
  `rustc` itself).
- `addr2line` provides a high-level `Context` API on top of gimli that maps
  addresses to source locations and function names in a single call.
- The `object` crate handles ELF container parsing and is designed to feed
  directly into gimli.

### Arc\<Mutex\<DebugSession\>\> vs Channels for Probe Access

The probe's `Session` and `Core` objects are **not** `Send` across all
operations (core borrows are lifetime-scoped to the session). We wrap
`DebugSession` in `Arc<Mutex<…>>` so the poller task can acquire the lock,
borrow a `Core`, perform a batch of reads, and release the lock — all within
a single lock scope. This keeps probe access serialised without requiring a
dedicated "probe actor" task.

`SymbolEngine` is wrapped in `Arc<…>` (no Mutex) because it is read-only
after initial ELF loading.

### Synchronous probe-rs Calls Inside Async

`probe-rs` performs blocking USB I/O. Rather than spawning a dedicated
`tokio::task::spawn_blocking` for every read, the poller acquires the mutex
and performs all reads synchronously while holding the lock. This batches
USB transactions and avoids per-read task-spawn overhead. The lock hold time
is bounded because each poll cycle reads a fixed set of expressions.

The poller runs on a `tokio::time::interval` tick, and the mutex ensures the
main task (which may also need probe access for halt/resume/reset commands)
does not conflict with the polling reads.

### SVD Auto-Fetch and Caching

SVD files are downloaded on demand from the `stm32-rs/stm32-rs` GitHub
repository. The chip name is mapped to an SVD filename by a deterministic
rule:

```
STM32F407VG → stm32f407.svd
STM32H563ZI → stm32h563.svd
```

The fetcher tries both `master` and `main` branches and caches the result
in `svd_cache_dir` (default: platform cache directory via the `directories`
crate). Subsequent runs use the cached file without network access.

### Immediate-Mode Rendering

The TUI uses an immediate-mode rendering model: every frame, the entire
terminal is redrawn from `TuiState`. There is no retained widget tree or
diff engine. This simplifies state management — the pane `draw()` functions
are pure functions of `(Frame, Rect, PaneState, focused)` with no side
effects.

Ratatui's double-buffering ensures only changed cells are written to the
terminal, keeping actual I/O minimal despite full redraws.
