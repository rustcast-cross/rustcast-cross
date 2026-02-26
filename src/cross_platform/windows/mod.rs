use std::path::PathBuf;

use lnk::{Encoding, encoding::WINDOWS_1252};
use windows::{Win32::{Globalization::GetACP, System::Com::CoTaskMemFree, UI::{Shell::{FOLDERID_LocalAppData, FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86, KF_FLAG_DEFAULT, SHGetKnownFolderPath}, WindowsAndMessaging::GetCursorPos}}, core::GUID};

pub mod appicon;

#[allow(clippy::cast_precision_loss)]
pub fn open_on_focused_monitor() -> iced::Point {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
    };

    use crate::app::{DEFAULT_WINDOW_HEIGHT, WINDOW_WIDTH};
    let mut point = POINT { x: 0, y: 0 };

    #[allow(clippy::cast_possible_truncation)]
    let mut monitor_info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    let _cursor = unsafe { GetCursorPos(&raw mut point) };
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    let _monitor_infos = unsafe { GetMonitorInfoW(monitor, &raw mut monitor_info) };

    let monitor_width = monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
    let monitor_height = monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;
    let window_width = WINDOW_WIDTH;
    let window_height = DEFAULT_WINDOW_HEIGHT;

    let x = monitor_info.rcMonitor.left as f32 + (monitor_width as f32 - window_width) / 2.0;
    let y = monitor_info.rcMonitor.top as f32 + (monitor_height as f32 - window_height) / 2.0;

    iced::Point { x, y }
}

/// Wrapper over `GetACP` that defaults to `WINDOWS_1252` if the ACP isn't found
pub fn get_acp() -> Encoding {
    #[allow(clippy::cast_possible_truncation)]
    unsafe { codepage::to_encoding(GetACP() as u16) }.unwrap_or_else(|| {
        tracing::warn!(
            "ACP not found, falling back to WINDOWS_1252 as the default shortcut encoding"
        );
        WINDOWS_1252
    })
}


/// Wrapper around `SHGetKnownFolderPath` to get paths to known folders
fn get_windows_path(folder_id: &GUID) -> Option<PathBuf> {
    unsafe {
        let folder = SHGetKnownFolderPath(folder_id, KF_FLAG_DEFAULT, None);
        if let Ok(folder) = folder {
            let path = folder.to_string().ok()?;
            CoTaskMemFree(Some(folder.0.cast()));
            Some(path.into())
        } else {
            None
        }
    }
}

/// Returns the set of known paths
pub fn get_known_paths() -> Vec<PathBuf> {
    let paths = vec![
        get_windows_path(&FOLDERID_ProgramFiles).unwrap_or_default(),
        get_windows_path(&FOLDERID_ProgramFilesX86).unwrap_or_default(),
        (get_windows_path(&FOLDERID_LocalAppData)
            .unwrap_or_default()
            .join("Programs")),
    ];
    paths
}