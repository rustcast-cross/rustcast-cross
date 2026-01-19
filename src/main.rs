mod app;
mod calculator;
mod clipboard;
mod commands;
mod config;
mod utils;

mod cross_platform;

use std::fs::File;

// import from utils
use crate::utils::{create_config_file_if_not_exists, get_config_file_path, get_config_installation_dir, get_log_dir, read_config_file};

use crate::app::tile::Tile;

use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{HotKey, Modifiers},
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    {
        macos::set_activation_policy_accessory();
    }

    let file_path = get_config_file_path();
    let config = read_config_file(&file_path).unwrap();
    create_config_file_if_not_exists(&file_path, &config).unwrap();

    {
        let log_path = get_log_dir() + "/log.log";

        create_config_file_if_not_exists(&log_path, &config).unwrap();
        
        let file = File::create(&log_path)
            .expect("Failed to create logfile");

        let log_file    = tracing_subscriber::fmt::layer().with_ansi(false).with_writer(file);
        let console_out = tracing_subscriber::fmt::layer().with_filter(LevelFilter::INFO);

        let subscriber = tracing_subscriber::registry().with(log_file).with(console_out);

        tracing::subscriber::set_global_default(subscriber).expect("Error initing tracing");
        tracing::info!("Log file at: {}", &log_path);
    }

    let manager = GlobalHotKeyManager::new().unwrap();

    let modifier = Modifiers::from_name(&config.toggle_mod);

    let key = config.toggle_key;

    let show_hide = HotKey::new(modifier, key);

    // Hotkeys are stored as a vec so that hyperkey support can be added later
    let hotkeys = vec![show_hide];

    let result = manager
        .register_all(&hotkeys);

    if let Err(global_hotkey::Error::AlreadyRegistered(key)) = result {
        if key == show_hide {
            // It probably should give up here.
            panic!("Couldn't register the key to open ({})", key)
        }
        else { tracing::warn!("Couldn't register hotkey {}", key) }
    }
    else if let Err(e) = result {
        tracing::error!("{}", e.to_string())
    }

    tracing::info!("Starting.");

    iced::daemon(
        move || Tile::new((modifier, key), show_hide.id(), &config),
        Tile::update,
        Tile::view,
    )
    .subscription(Tile::subscription)
    .theme(Tile::theme)
    .run()
}
