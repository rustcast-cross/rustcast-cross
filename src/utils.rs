//! This has all the utility functions that rustcast uses

use std::{
    io,
    path::{Path, PathBuf},
};

#[cfg(target_os = "macos")]
use {objc2_app_kit::NSWorkspace, objc2_foundation::NSURL};

#[cfg(target_os = "linux")]
use crate::cross_platform::linux::get_installed_linux_apps;

#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::process::Command;

pub fn get_config_installation_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        std::env::var("LOCALAPPDATA").unwrap().into()
    } else {
        std::env::var("HOME").unwrap().into()
    }
}

pub fn get_config_file_path() -> PathBuf {
    let home = get_config_installation_dir();

    if cfg!(target_os = "windows") {
        home.join("rustcast/config.toml")
    } else {
        home.join(".config/rustcast/config.toml")
    }
}

use crate::config::Config;

pub fn read_config_file(file_path: &Path) -> anyhow::Result<Config> {
    match std::fs::read_to_string(file_path) {
        Ok(a) => Ok(toml::from_str(&a)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let cfg = Config::default();
            std::fs::write(
                file_path,
                toml::to_string(&cfg).unwrap_or_else(|x| x.to_string()),
            )?;
            Ok(cfg)
        }
        Err(e) => Err(e.into()),
    }
}

// TODO: this should also work with args
pub fn open_application(path: impl AsRef<Path>) {
    let path = path.as_ref();

    #[cfg(target_os = "windows")]
    {
        println!("Opening application: {}", path.display());

        Command::new("powershell")
            .arg(format!("Start-Process '{}'", path.display()))
            .status()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        NSWorkspace::new().openURL(&NSURL::fileURLWithPath(
            &objc2_foundation::NSString::from_str(&path.to_string_lossy()),
        ));
    }

    #[cfg(target_os = "linux")]
    {
        Command::new(path).status().ok();
    }
    #[cfg(target_os = "linux")]
    {
        Command::new(path).status().ok();
    }
}

/// Converts a slice of BGRA data to RGBA using SIMD
///
/// Stolen from <https://stackoverflow.com/a/78190249>/
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn bgra_to_rgba(data: &mut [u8]) {
    use std::arch::x86_64::__m128i;
    use std::arch::x86_64::_mm_loadu_si128;
    use std::arch::x86_64::_mm_setr_epi8;
    use std::arch::x86_64::_mm_storeu_si128;

    #[cfg(target_arch = "x86")]
    use std::arch::x86::_mm_shuffle_epi8;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::_mm_shuffle_epi8;
    //
    // The shuffle mask for converting BGRA -> RGBA
    let mask: __m128i = unsafe {
        _mm_setr_epi8(
            2, 1, 0, 3, // First pixel
            6, 5, 4, 7, // Second pixel
            10, 9, 8, 11, // Third pixel
            14, 13, 12, 15, // Fourth pixel
        )
    };
    // For each 16-byte chunk in your data
    #[allow(clippy::cast_ptr_alignment)] // It's never actually wrong
    for chunk in data.chunks_exact_mut(16) {
        let mut vector = unsafe { _mm_loadu_si128(chunk.as_ptr().cast::<__m128i>()) };
        vector = unsafe { _mm_shuffle_epi8(vector, mask) };
        unsafe { _mm_storeu_si128(chunk.as_mut_ptr().cast::<__m128i>(), vector) };
    }
}

// Fallback for non x86/x86_64 devices (not like that'll ever be used, but why not)
/// Converts a slice of BGRA data to RGBA
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn bgra_to_rgba(data: &mut [u8]) {
    for i in (0..data.len()).step_by(4) {
        let r = data[i + 2];

        data[i + 2] = data[i];
        data[i] = r;
    }
}
