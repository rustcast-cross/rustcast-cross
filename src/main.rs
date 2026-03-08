mod app;
mod app_finding;
mod commands;
mod config;
mod functions;
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
use crate::utils::{get_config_file_path, get_config_installation_dir };

use crate::app::tile::{self, Tile};
use logging::preinit_logger;

#[cfg(not(target_os = "linux"))]
use crate::hotkey::init_hotkey_manager;

#[cfg(target_os = "linux")]
use crate::hotkey::init_socket;

/// Convenience function to open and parse the config file
fn read_config_file(file_path: &Path) -> anyhow::Result<Config> {
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
