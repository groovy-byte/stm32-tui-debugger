use std::path::PathBuf;

use clap::Parser;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub target_chip: String,
    pub elf_path: PathBuf,
    pub source_root: PathBuf,
    pub svd_cache_dir: PathBuf,
    #[serde(default = "default_poll_rate")]
    pub poll_rate_hz: u32,
    #[serde(default)]
    pub rtt_enabled: bool,
    #[serde(default)]
    pub rtt_channel: u32,
}

fn default_poll_rate() -> u32 {
    20
}

fn default_svd_cache_dir() -> PathBuf {
    ProjectDirs::from("", "", "stm32-tui-debugger")
        .map(|dirs| dirs.cache_dir().join("svd"))
        .unwrap_or_else(|| PathBuf::from(".cache/svd"))
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn load(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(AppError::Io)?;
        let mut config: Config =
            toml::from_str(&contents).map_err(|e| AppError::Config(e.to_string()))?;

        // Expand ~ in svd_cache_dir
        if let Some(s) = config.svd_cache_dir.to_str() {
            if s.starts_with("~/") {
                if let Some(home) = std::env::var_os("HOME") {
                    config.svd_cache_dir = PathBuf::from(home).join(&s[2..]);
                }
            }
        }

        Ok(config)
    }

    /// Build a `Config` from CLI arguments, optionally merging with a TOML file.
    pub fn from_cli(cli: &Cli) -> Result<Self> {
        if let Some(ref config_path) = cli.config {
            let mut config = Self::load(config_path)?;
            // CLI args take precedence over file values.
            config.target_chip = cli.chip.clone();
            config.elf_path = cli.elf.clone();
            config.poll_rate_hz = cli.poll_rate;
            Ok(config)
        } else {
            Ok(Self::default_for_chip(&cli.chip, &cli.elf))
        }
    }

    /// Produce a `Config` with sensible defaults for a given chip and ELF path.
    pub fn default_for_chip(chip: &str, elf_path: &PathBuf) -> Self {
        let source_root = elf_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            target_chip: chip.to_string(),
            elf_path: elf_path.clone(),
            source_root,
            svd_cache_dir: default_svd_cache_dir(),
            poll_rate_hz: default_poll_rate(),
            rtt_enabled: false,
            rtt_channel: 0,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "stm32-tui-debugger")]
#[command(about = "Terminal debugger for STM32 C/C++ projects")]
pub struct Cli {
    /// Path to firmware ELF file
    #[arg(short, long)]
    pub elf: PathBuf,

    /// Target chip name (e.g., STM32F407VG)
    #[arg(short, long)]
    pub chip: String,

    /// Path to config TOML file (optional)
    #[arg(short = 'C', long)]
    pub config: Option<PathBuf>,

    /// Poll rate in Hz
    #[arg(long, default_value = "20")]
    pub poll_rate: u32,
}
