mod app;
mod calculator;
mod clipboard;
mod commands;
mod config;
mod macos;
mod utils;

use std::path::Path;

use crate::{app::tile::Tile, config::Config, utils::to_key_code};

use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    {
        macos::set_activation_policy_accessory();
    }

    let home = std::env::var("HOME").unwrap();

    let file_path = home.clone() + "/.config/rustcast/config.toml";
    if !Path::new(&file_path).exists() {
        std::fs::create_dir_all(home + "/.config/rustcast").unwrap();
        std::fs::write(
            &file_path,
            toml::to_string(&Config::default()).unwrap_or_else(|x| x.to_string()),
        )
        .unwrap();
    }
    let config: Config = match std::fs::read_to_string(&file_path) {
        Ok(a) => toml::from_str(&a).unwrap_or(Config::default()),
        Err(_) => Config::default(),
    };

    let manager = GlobalHotKeyManager::new().unwrap();

    let show_hide = HotKey::new(
        Some(Modifiers::from_name(&config.toggle_mod).unwrap_or(Modifiers::ALT)),
        to_key_code(&config.toggle_key).unwrap_or(Code::Space),
    );

    manager
        .register_all(&[show_hide])
        .expect("Unable to register hotkey");

    iced::daemon(
        move || Tile::new(show_hide.id(), &config),
        Tile::update,
        Tile::view,
    )
    .subscription(Tile::subscription)
    .theme(Tile::theme)
    .run()
}
