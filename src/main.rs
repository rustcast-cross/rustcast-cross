mod app;
mod calculator;
mod clipboard;
mod commands;
mod config;
mod icon;
mod styles;
mod unit_conversion;
mod utils;

mod cross_platform;
mod preinit_logger;

use std::env::temp_dir;
use std::fs::{File, create_dir_all};
use std::io;

use crate::config::{Config, Logger};
// import from utils
use crate::utils::{get_config_file_path, get_config_installation_dir, read_config_file};

use crate::app::tile::{self, Tile};

#[cfg(not(target_os = "linux"))]
use global_hotkey::GlobalHotKeyManager;
use tracing::instrument::WithSubscriber;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

#[cfg(target_os = "linux")]
const SOCKET_PATH: &str = "/tmp/rustcast.sock";

fn load_config() -> Result<Config, anyhow::Error> {
    let file_path = get_config_file_path();
    let config = read_config_file(&file_path);
    if let Err(e) = config {
        // Tracing isn't inited yet
        preinit_logger::error(&format!("Error parsing config: {e}"));
        return Err(e.context("Failure to parse config"))
    }

    config
}

fn init_loggers(config: &Config) {
    let loggers: Vec<_> = config.log
        .iter()
        .filter_map(|(k, config)| {
            let v = match config {
                Logger::Stdout { level, use_ansi, env_filter } => 
                    Some(tracing_subscriber::fmt::layer()
                        .with_ansi(*use_ansi)
                        .with_filter(LevelFilter::from_level(*level))
                        .boxed()),
                Logger::File { level, path, use_ansi, env_filter } => {
                    let file = File::create(path)
                        .inspect_err(|e|
                            preinit_logger::error(&format!(
                                "Failed to create log at path {} with error {e:?}, skipping", path.display()))
                        )
                        .ok()?;

                    Some(tracing_subscriber::fmt::layer()
                        .with_ansi(*use_ansi)
                        .with_writer(file)
                        .with_filter(LevelFilter::from_level(*level))
                        .boxed()
                    )
                }
            };

            preinit_logger::info(&format!("Inited logger {k}"));

            v
        })
        .collect();

    let registry = tracing_subscriber::registry().with(loggers);
    
    if let Err(e) = tracing::subscriber::set_global_default(registry) {
        preinit_logger::error(&format!("Error starting tracing loggers: {e:?}"));
        std::process::exit(-1); // Can't think of another sane thing to do here that wouldn't leave
                                // the app with no logs at all
    }

    tracing::info!("Inited loggers in config");
}

fn load_config() -> Result<Config, anyhow::Error> {
    let file_path = get_config_file_path();
    let config = read_config_file(&file_path);
    if let Err(e) = config {
        // Tracing isn't inited yet
        preinit_logger::error(&format!("Error parsing config: {e}"));
        return Err(e.context("Failure to parse config"))
    }

    config
}

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    cross_platform::macos::set_activation_policy_accessory();

    let config_dir = get_config_installation_dir();
    let config = if let Err(e) = std::fs::metadata(config_dir.join("rustcast/")) {
        if e.kind() == io::ErrorKind::NotFound {
            let result = create_dir_all(config_dir.join("rustcast/"));

            if let Err(e) = result {
                preinit_logger::error(&format!("{e}"));
                std::process::exit(1);
            }
            } else {
                if result.is_err() {
                    preinit_logger::error(&format!("Error creating dirs: {e}"));
                }
            }
        };

    let file_path = get_config_file_path();

    let config = match read_config_file(&file_path) {
        Err(e) => {
            preinit_logger::warn("Failed to load config; using default config");
            Config::default()
        }
        Ok(config) => {
            load_config()
                .inspect_err(|e| preinit_logger::error(&format!("Failed to load config: {e:?}")))
                .unwrap_or_default();
            
            init_loggers(&config);
            config
        }
    };

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
