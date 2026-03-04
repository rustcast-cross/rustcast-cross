mod app;
mod app_finding;
mod calculator;
mod clipboard;
mod commands;
mod config;
mod hotkey;
mod icon;
mod logging;
mod platform;
mod styles;
mod unit_conversion;
mod utils;

use std::fs::create_dir_all;
use std::io;
use std::path::Path;

use crate::config::Config;
use crate::logging::init::init_loggers;
use crate::utils::{get_config_file_path, get_config_installation_dir, read_config_file};

use crate::app::tile::{self, Tile};
use logging::preinit_logger;

#[cfg(not(target_os = "linux"))]
use crate::hotkey::init_hotkey_manager;

#[cfg(target_os = "linux")]
use crate::hotkey::init_socket;

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
    platform::macos::set_activation_policy_accessory();

    let config = load_config();
    tracing::debug!(target: "init", "Loaded config: {config:#?}");

    #[cfg(not(target_os = "linux"))]
    let (_manager, show_hide_bind) = init_hotkey_manager(&config);

    #[cfg(target_os = "linux")]
    init_socket();

    tracing::info!(target: "init", "Starting rustcast");

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
