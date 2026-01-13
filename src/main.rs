mod app;
mod calculator;
mod clipboard;
mod commands;
mod config;

#[cfg(target_os = "macos")]
mod haptics;

mod macos;
mod utils;
mod windows;

// import from utils
use crate::utils::{create_config_file_if_not_exists, get_config_file_path, read_config_file};

use crate::app::tile::Tile;

use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{HotKey, Modifiers},
};

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    {
        macos::set_activation_policy_accessory();
    }

    let file_path = get_config_file_path();
    let config = read_config_file(&file_path).unwrap();
    create_config_file_if_not_exists(&file_path, &config).unwrap();

    let manager = GlobalHotKeyManager::new().unwrap();

    let modifier = Modifiers::from_name(&config.toggle_mod);

    let key = config.toggle_key;

    let show_hide = HotKey::new(modifier, key);

    // Hotkeys are stored as a vec so that hyperkey support can be added later
    let hotkeys = vec![show_hide];

    manager
        .register_all(&hotkeys)
        .expect("Unable to register hotkey");

    println!("Starting");

    iced::daemon(
        move || Tile::new((modifier, key), show_hide.id(), &config),
        Tile::update,
        Tile::view,
    )
    .subscription(Tile::subscription)
    .theme(Tile::theme)
    .run()
}
