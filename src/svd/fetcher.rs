use std::path::PathBuf;

use tracing::{debug, info, warn};

use crate::error::SvdError;

pub struct SvdFetcher {
    cache_dir: PathBuf,
}

impl SvdFetcher {
    pub fn new(cache_dir: PathBuf) -> Self {
        SvdFetcher { cache_dir }
    }

    /// Get SVD file path — returns cached version or downloads it.
    pub fn get_svd_path(&self, chip_name: &str) -> Result<PathBuf, SvdError> {
        let filename = Self::chip_to_svd_filename(chip_name);
        let cached_path = self.cache_dir.join(&filename);

        if cached_path.exists() {
            debug!(path = %cached_path.display(), "Using cached SVD file");
            return Ok(cached_path);
        }

        info!(chip = chip_name, "SVD file not cached, downloading");
        self.download_svd(chip_name)
    }

    /// Map a chip name to the expected SVD filename.
    ///
    /// The convention used by stm32-rs is:
    ///   STM32F407VG → stm32f407.svd
    ///   STM32H563ZI → stm32h563.svd
    ///
    /// We lowercase the name, keep `stm32` + family letter + numeric part,
    /// and drop the trailing pin/package suffix.
    fn chip_to_svd_filename(chip_name: &str) -> String {
        let lower = chip_name.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();

        let mut stem = String::new();
        let mut i = 0;

        // Copy the "stm32" prefix.
        while i < chars.len() && i < 5 {
            stem.push(chars[i]);
            i += 1;
        }

        // Copy the family letter (e.g. 'f', 'h', 'l', 'g', 'u', 'w').
        if i < chars.len() && chars[i].is_alphabetic() {
            stem.push(chars[i]);
            i += 1;
        }

        // Copy the numeric model identifier (typically 3-4 digits).
        while i < chars.len() && chars[i].is_ascii_digit() {
            stem.push(chars[i]);
            i += 1;
        }

        format!("{}.svd", stem)
    }

    fn download_svd(&self, chip_name: &str) -> Result<PathBuf, SvdError> {
        let filename = Self::chip_to_svd_filename(chip_name);
        let stem = filename.trim_end_matches(".svd");

        // stm32-rs keeps the SVD files named by family on the master branch.
        let urls = [
            format!(
                "https://raw.githubusercontent.com/stm32-rs/stm32-rs/master/svd/{}.svd",
                stem
            ),
            format!(
                "https://raw.githubusercontent.com/stm32-rs/stm32-rs/main/svd/{}.svd",
                stem
            ),
        ];

        std::fs::create_dir_all(&self.cache_dir).map_err(|e| {
            SvdError::FetchFailed(format!("Failed to create cache directory: {}", e))
        })?;

        let mut last_err = String::new();
        for url in &urls {
            debug!(url, "Attempting SVD download");
            match reqwest::blocking::get(url) {
                Ok(resp) if resp.status().is_success() => {
                    let body = resp.text().map_err(|e| {
                        SvdError::FetchFailed(format!("Failed to read response body: {}", e))
                    })?;

                    let dest = self.cache_dir.join(&filename);
                    std::fs::write(&dest, &body).map_err(|e| {
                        SvdError::FetchFailed(format!("Failed to write SVD to cache: {}", e))
                    })?;

                    info!(path = %dest.display(), "SVD file cached");
                    return Ok(dest);
                }
                Ok(resp) => {
                    last_err = format!("HTTP {} from {}", resp.status(), url);
                    warn!(url, status = %resp.status(), "Download attempt failed");
                }
                Err(e) => {
                    last_err = format!("{}: {}", url, e);
                    warn!(url, error = %e, "Download attempt failed");
                }
            }
        }

        Err(SvdError::FetchFailed(format!(
            "Could not download SVD for '{}': {}",
            chip_name, last_err
        )))
    }

    /// Check whether the SVD for a given chip is already cached locally.
    pub fn is_cached(&self, chip_name: &str) -> bool {
        let filename = Self::chip_to_svd_filename(chip_name);
        self.cache_dir.join(filename).exists()
    }

    /// Remove all cached SVD files.
    pub fn clear_cache(&self) -> Result<(), SvdError> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir).map_err(|e| {
                SvdError::FetchFailed(format!("Failed to clear SVD cache: {}", e))
            })?;
        }
        Ok(())
    }
}
