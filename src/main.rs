mod app;
mod commands;
mod config;
mod macos;
mod utils;

use crate::{app::Tile, config::Config, utils::to_key_code};

use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    {
        macos::set_activation_policy_accessory();
    }

    let file_path = std::env::var("HOME").unwrap() + "/.config/rustcast/config.toml";
    let config: Config = match std::fs::read_to_string(&file_path) {
        Ok(a) => toml::from_str(&a).unwrap(),
        Err(_) => Config::default(),
    };
    std::fs::write(
        &file_path,
        toml::to_string(&config).unwrap_or_else(|x| x.to_string()),
    )
    .unwrap();
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
