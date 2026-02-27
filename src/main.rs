mod app;
mod app_finding;
mod calculator;
mod clipboard;
mod commands;
mod config;
mod cross_platform;
mod icon;
mod logging;
mod styles;
mod unit_conversion;
mod utils;

use std::fs::create_dir_all;
use std::io;
use std::path::Path;

use crate::config::Config;
use crate::logging::init::init_loggers;
// import from utils
use crate::utils::{get_config_file_path, get_config_installation_dir, read_config_file};

use crate::app::tile::{self, Tile};
use logging::preinit_logger;

#[cfg(not(target_os = "linux"))]
use global_hotkey::GlobalHotKeyManager;
use tracing_subscriber::EnvFilter;

#[cfg(target_os = "linux")]
const SOCKET_PATH: &str = "/tmp/rustcast.sock";

fn parse_cfg_file(path: impl AsRef<Path>) -> anyhow::Result<Config> {
    let config = read_config_file(path.as_ref());
    if let Err(e) = config {
        return Err(e.context("Failure to parse config"));
    }

    config
}

fn load_config() -> Config {
    let config_dir = get_config_installation_dir().join("rustcast/");

    if let Err(e) = std::fs::metadata(&config_dir) {
        if e.kind() == io::ErrorKind::NotFound {
            preinit_logger::info(&format!("Config dir at {}", &config_dir.display()));
            let result = create_dir_all(config_dir);

            if let Err(e) = result {
                preinit_logger::error(&format!("Error creating config dirs: {e}"));
            }
        } else {
            preinit_logger::error(&format!("Error getting config dir: {e}"));
        }

        preinit_logger::warn("Errors opening config dir, using default cfg");
        return Config::default();
    }

    let file_path = get_config_file_path();

    match read_config_file(&file_path) {
        Err(e) => {
            preinit_logger::warn(&format!(
                "Failed to load config with error {e}; using default config"
            ));
            Config::default()
        }
        Ok(config) => {
            parse_cfg_file(file_path)
                .inspect_err(|e| {
                    preinit_logger::warn(&format!(
                        "Failed to load config with error {e}; using default config"
                    ));
                })
                .unwrap_or_default();

            init_loggers(&config);
            config
        }
    }
}

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    cross_platform::macos::set_activation_policy_accessory();

    let config = load_config();

    #[cfg(target_os = "linux")]
    {
        // error handling should really be improved soon (tm)
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

    #[cfg(not(target_os = "linux"))]
    let show_hide_bind = {
        let manager = GlobalHotKeyManager::new().unwrap();

        let show_hide = config.toggle_hotkey.parse().unwrap();

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
                tracing::warn!("Couldn't register hotkey {}", key);
            }
        } else if let Err(e) = result {
            tracing::error!("{}", e.to_string());
        }

        show_hide
    };

    tracing::info!("Starting.");

    iced::daemon(
        move || {
            tile::elm::new(
                #[cfg(not(target_os = "linux"))]
                show_hide_bind,
                &config,
            )
        },
        tile::update::handle_update,
        tile::elm::view,
    )
    .subscription(Tile::subscription)
    .theme(Tile::theme)
    .run()
}
