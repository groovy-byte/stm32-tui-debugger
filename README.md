# stm32-tui-debugger

A terminal-based debugger for STM32 microcontrollers, written in Rust.

<!-- Badges: uncomment when published
[![Crates.io](https://img.shields.io/crates/v/stm32-tui-debugger)](https://crates.io/crates/stm32-tui-debugger)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
-->

## Overview

`stm32-tui-debugger` provides live, non-intrusive inspection of STM32
firmware directly from the terminal. It reads ELF/DWARF symbols from your
C/C++ build, fetches SVD peripheral descriptions from
[stm32-rs](https://github.com/stm32-rs/stm32-rs), and polls target memory
over ST-Link — all without halting the CPU.

**Who is this for?** Embedded developers who prefer a terminal workflow, want
faster iteration than a full IDE, or need to monitor firmware on a headless
machine.

### Key Capabilities

- Five-pane TUI: source viewer, peripheral tree, FreeRTOS tasks, live expressions, console/RTT
- ELF/DWARF variable and function resolution via `gimli` + `addr2line`
- SVD register decoding with automatic download and caching
- Async memory polling at configurable rates (default 20 Hz)
- C++ Itanium-ABI name demangling
- FreeRTOS task monitoring with auto-detection and color-coded state display
- RTT log streaming infrastructure (probe-rs integration stub)
- Typed error hierarchy with `thiserror`

## TUI Layout

```
┌─────────────────────────────────┬────────────────────────┐
│                                 │                        │
│       Source Viewer              │   Peripherals (SVD)    │
│       (line numbers,            │   (expandable tree)    │
│        current-line highlight)  │                        │
│                                 ├────────────────────────┤
│                                 │                        │
│                                 │   Tasks (FreeRTOS)     │
│                                 │   (color-coded states) │
│                                 │                        │
├─────────────────────────────────┼────────────────────────┤
│                                 │                        │
│       Live Expressions          │   Console / RTT        │
│       (name, value, Δ)          │   (color-coded logs)   │
│                                 │                        │
├─────────────────────────────────┴────────────────────────┤
│ Tab: switch pane │ F5: resume │ F6: halt │ F7: reset │ q │
└──────────────────────────────────────────────────────────┘
```

## Features

| Feature | Status | Notes |
|:--------|:------:|:------|
| Multi-pane TUI (source, peripherals, tasks, expressions, console) | ✅ | `ratatui` + `crossterm` |
| ELF/DWARF symbol resolution | ✅ | `gimli`, `addr2line`, `object` |
| SVD peripheral register decoding | ✅ | `svd-parser`, auto-fetch from stm32-rs |
| Live memory polling with change detection | ✅ | Async `PollerEngine`, configurable Hz |
| C++ name demangling | ✅ | `cpp_demangle` (Itanium ABI) |
| RTT log streaming (infrastructure) | ✅ | Buffer + async task stub |
| Probe session management (connect/halt/resume/reset) | ✅ | `probe-rs` |
| FreeRTOS task monitoring | ✅ | Auto-detect via `pxCurrentTCB`, color-coded states, 2–5 Hz polling |
| DAP/GDB source-level debugging | 📋 | Planned |
| ~~FreeRTOS task view~~ | ~~📋~~ | ~~Planned~~ — see above |
| Breakpoint management | 📋 | Planned |
| C struct / bitfield live decoding | 📋 | Types defined, decoder planned |

## Requirements

- **Rust** 1.70+ (edition 2021)
- **ST-Link** v2 or v3 debug probe
- **Target** STM32 board with SWD connection
- **Firmware ELF** compiled with debug symbols (`-g`)

## Installation

```sh
# Build from source
cargo build --release

# Or install into ~/.cargo/bin
cargo install --path .
```

## Usage

```sh
# Minimal — chip name + ELF path are required
stm32-tui-debugger --chip STM32F407VG --elf target/firmware.elf

# Custom poll rate (Hz)
stm32-tui-debugger -c STM32H563ZI -e build/app.elf --poll-rate 50

# With a TOML config file (CLI flags override file values)
stm32-tui-debugger -c STM32F411CEUx -e firmware.elf -C debugger.toml
```

### CLI Flags

| Flag | Short | Description | Default |
|:-----|:-----:|:------------|:--------|
| `--elf <PATH>` | `-e` | Path to firmware ELF file | *required* |
| `--chip <NAME>` | `-c` | Target chip (e.g. `STM32F407VG`) | *required* |
| `--config <PATH>` | `-C` | TOML configuration file | none |
| `--poll-rate <HZ>` | | Memory poll frequency | `20` |

## Configuration

Create a TOML file for persistent settings:

```toml
target_chip = "STM32F407VG"
elf_path = "build/firmware.elf"
source_root = "src/"
svd_cache_dir = "~/.cache/stm32-tui-debugger/svd"
poll_rate_hz = 30
rtt_enabled = false
rtt_channel = 0
```

Fields `poll_rate_hz`, `rtt_enabled`, and `rtt_channel` are optional and fall
back to defaults when omitted. CLI flags for `--chip`, `--elf`, and
`--poll-rate` always override the file values.

SVD files are automatically downloaded from
`stm32-rs/stm32-rs` on first use and cached in `svd_cache_dir`.

## Keybindings

### Global

| Key | Action |
|:----|:-------|
| `q` / `Ctrl-c` | Quit |
| `Tab` | Next pane |
| `Shift-Tab` | Previous pane |
| `F5` | Resume target |
| `F6` | Halt target |
| `F7` | Reset target (halt after reset) |
| `F10` | Step over (planned) |
| `F11` | Step into (planned) |

### Navigation (all panes)

| Key | Action |
|:----|:-------|
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |
| `PageUp` | Page up |
| `PageDown` | Page down |
| `Enter` | Select |
| `Esc` | Back |

### Pane-Specific

| Key | Pane | Action |
|:----|:-----|:-------|
| `Space` | Peripherals | Toggle register expand |
| `a` | Expressions | Add expression |
| `d` | Expressions | Remove expression |
| `r` | Tasks | Refresh task list |
| `i` / `Enter` | Tasks | Inspect selected task |

## FreeRTOS Support

The Tasks pane provides live FreeRTOS kernel awareness — no agent or target-side
code required.

**Auto-detection.** On startup the debugger looks for the `pxCurrentTCB` symbol
in the firmware ELF. If found, FreeRTOS monitoring activates automatically;
otherwise the pane shows a "not detected" message.

**What you see.** Each task row displays:

| Column | Description |
|:-------|:------------|
| Name | Task name (up to 16 chars from TCB) |
| Pri | Numeric priority |
| State | Running, Ready, Blocked, Suspended, or Deleted |
| Stack Top | Current stack pointer (`0x…`) |
| Stack Base | Stack base address (`0x…`) |

States are **color-coded**: green (Running), cyan (Ready), yellow (Blocked),
dark-gray (Suspended), red (Deleted).

**How it works.** A background RTOS poller reads target memory at 2–5 Hz,
walking the FreeRTOS internal linked lists (`pxReadyTasksLists`,
`xDelayedTaskList1/2`, `xSuspendedTaskList`) and parsing each TCB structure.
The low poll rate keeps probe overhead minimal — each snapshot requires many
individual memory reads.

**Configurable TCB layout.** The default byte offsets match a standard
Cortex-M FreeRTOS build without MPU support. If your FreeRTOS configuration
changes the TCB struct layout (e.g. MPU-enabled, additional hooks), you can
supply custom offsets via a TOML config section or the `TcbLayout` API.

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full design document.

```
main.rs ──► config ──► probe/session ──► probe/memory
                  │         │
                  ├── symbols (ELF + DWARF + demangler)
                  ├── svd (fetcher → parser → registry)
                  ├── poller (engine ◄── expressions)
                  ├── rtos (task reader ◄── TCB/list types)
                  ├── rtt (streaming buffer)
                  └── tui (app, state, keybindings, panes)
```

## Development

```sh
# Build
cargo build

# Run tests
cargo test

# Check without building
cargo check

# Format
cargo fmt

# Lint
cargo clippy
```

### Project Structure

```
src/
├── main.rs          Entry point (Tokio async main)
├── lib.rs           Module declarations
├── config.rs        CLI (clap) + TOML config
├── error.rs         Typed error hierarchy
├── probe/           ST-Link session & memory access
├── symbols/         ELF loader, DWARF resolver, demangler
├── svd/             SVD fetch, parse, peripheral registry
├── poller/          Async expression polling engine
├── rtos/            FreeRTOS task monitoring
├── rtt/             RTT log buffer + streaming task
└── tui/             Ratatui application
    ├── app.rs       Layout & rendering
    ├── state.rs     Pane state models
    ├── event.rs     Crossterm event reader
    ├── keybindings.rs  Key → Command mapping
    ├── panes/       Per-pane draw functions
    └── widgets/     Custom widgets (placeholder)
```

## License

MIT — see [LICENSE](LICENSE) for details.
