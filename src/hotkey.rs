//! Multi-platform handling of hotkeys

#[cfg(not(target_os = "linux"))]
use {
    crate::config::Config,
    global_hotkey::{self, GlobalHotKeyManager, hotkey::HotKey},
};

/// Initialises the hotkey manager.
///
/// **IMPORTANT:** If the hotkey manager returned dies, it will stop sending out events.
#[cfg(not(target_os = "linux"))]
pub fn init_hotkey_manager(config: &Config) -> (GlobalHotKeyManager, HotKey) {
    let manager = GlobalHotKeyManager::new().unwrap();

    let show_hide = config.toggle_hotkey.parse().unwrap();
    tracing::debug!(target: "init", "Show/hide hotkey: {:?}", show_hide);

    let mut hotkeys = vec![show_hide];

    if let Some(show_clipboard) = &config.clipboard_hotkey
        && let Some(cb_page_hk) = show_clipboard.parse().ok()
    {
        hotkeys.push(cb_page_hk);
    }

    let result = manager.register_all(&hotkeys);

    if let Err(global_hotkey::Error::AlreadyRegistered(key)) = result {
        if key == show_hide {
            // It probably should give up here.
            panic!("Couldn't register the key to open ({key})")
        } else {
            tracing::warn!(target: "init", "Couldn't register hotkey {}", key);
        }
    } else if let Err(e) = result {
        tracing::error!("{}", e.to_string());
    }

    (manager, show_hide)
}

/// Initialises sockets (for linux)
#[cfg(target_os = "linux")]
pub fn init_socket() {
    // All this code is by kybe, idk what it does very specifically

    // error handling should really be improved soon (tm)
    use crate::platform::linux::SOCKET_PATH;
    use std::fs;
    use std::os::unix::net::UnixListener;
    use std::{io::Write, os::unix::net::UnixStream};
    use tracing::info;

    if UnixListener::bind(SOCKET_PATH).is_err() {
        match UnixStream::connect(SOCKET_PATH) {
            Ok(mut stream) => {
                use std::env;

                let clipboard = env::args().any(|arg| arg.trim() == "--cphist");
                let cmd = if clipboard { "clipboard" } else { "toggle" };
                info!("socket sending: {cmd}");
                let _ = stream.write_all(cmd.as_bytes());
                std::process::exit(0);
            }
            Err(_) => {
                let _ = fs::remove_file(SOCKET_PATH);
            }
        }
    }
}
