use std::path::PathBuf;
use stm32_tui_debugger::config::Config;

#[test]
fn should_set_target_chip() {
    let cfg = Config::default_for_chip("STM32F407VG", &PathBuf::from("firmware.elf"));
    assert_eq!(cfg.target_chip, "STM32F407VG");
}

#[test]
fn should_set_elf_path() {
    let cfg = Config::default_for_chip("STM32F407VG", &PathBuf::from("firmware.elf"));
    assert_eq!(cfg.elf_path, PathBuf::from("firmware.elf"));
}

#[test]
fn should_default_poll_rate_to_20() {
    let cfg = Config::default_for_chip("STM32F407VG", &PathBuf::from("firmware.elf"));
    assert_eq!(cfg.poll_rate_hz, 20);
}

#[test]
fn should_default_rtt_disabled() {
    let cfg = Config::default_for_chip("STM32F407VG", &PathBuf::from("firmware.elf"));
    assert!(!cfg.rtt_enabled);
}

#[test]
fn should_default_rtt_channel_to_zero() {
    let cfg = Config::default_for_chip("STM32F407VG", &PathBuf::from("firmware.elf"));
    assert_eq!(cfg.rtt_channel, 0);
}

#[test]
fn should_have_svd_cache_dir() {
    let cfg = Config::default_for_chip("STM32F407VG", &PathBuf::from("firmware.elf"));
    // The svd_cache_dir should be non-empty (either XDG-based or fallback)
    assert!(!cfg.svd_cache_dir.as_os_str().is_empty());
}

#[test]
fn should_derive_source_root_from_elf_parent() {
    let cfg = Config::default_for_chip("STM32H563ZI", &PathBuf::from("/home/dev/project/build/fw.elf"));
    assert_eq!(cfg.source_root, PathBuf::from("/home/dev/project/build"));
}

#[test]
fn should_fallback_source_root_to_dot_for_bare_filename() {
    let cfg = Config::default_for_chip("STM32F407VG", &PathBuf::from("firmware.elf"));
    // A bare filename has no parent directory, so source_root should be "."
    assert_eq!(cfg.source_root, PathBuf::from(""));
}

#[test]
fn should_load_invalid_config_file_returns_error() {
    let result = Config::load(&PathBuf::from("/nonexistent/config.toml"));
    assert!(result.is_err());
}
