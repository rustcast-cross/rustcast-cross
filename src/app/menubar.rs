//! This has the menubar icon logic for the app

use global_hotkey::{hotkey::Code, hotkey::Modifiers};
use image::{DynamicImage, ImageReader};
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{
        AboutMetadataBuilder, Icon as Ico, Menu, MenuEvent, MenuItem, PredefinedMenuItem,
        accelerator::Accelerator,
    },
};

use crate::{
    app::{Message, tile::ExtSender},
    utils::open_settings,
};

use tokio::runtime::Runtime;

/// This create a new menubar icon for the app
pub fn menu_icon(hotkey: (Option<Modifiers>, Code), hotkey_id: u32, sender: ExtSender) -> TrayIcon {
    let builder = TrayIconBuilder::new();

    let image = get_image();
    let icon = Icon::from_rgba(image.as_bytes().to_vec(), image.width(), image.height()).unwrap();

    init_event_handler(sender, hotkey_id);

    let menu = Menu::with_items(&[
        &version_item(),
        &about_item(image),
        &PredefinedMenuItem::separator(),
        &refresh_item(),
        &open_item(hotkey),
        &PredefinedMenuItem::separator(),
        &open_settings_item(),
        &quit_item(),
    ])
    .unwrap();

    builder
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .build()
        .unwrap()
}

fn get_image() -> DynamicImage {
    let image_path = if cfg!(debug_assertions) {
        "docs/icon.png"
    } else {
        "../Resources/icon.png"
    };

    let image = ImageReader::open(image_path).unwrap().decode().unwrap();

    image
}

fn init_event_handler(sender: ExtSender, hotkey_id: u32) {
    let runtime = Runtime::new().unwrap();

    MenuEvent::set_event_handler(Some(move |x: MenuEvent| {
        let sender = sender.clone();
        let sender = sender.0.clone();
        match x.id().0.as_str() {
            "refresh_rustcast" => {
                runtime.spawn(async move {
                    sender.clone().try_send(Message::ReloadConfig).unwrap();
                });
            }
            "show_rustcast" => {
                runtime.spawn(async move {
                    sender
                        .clone()
                        .try_send(Message::KeyPressed(hotkey_id))
                        .unwrap();
                });
            }
            "open_preferences" => {
                open_settings();
            }
            _ => {}
        }
    }));
}

fn version_item() -> MenuItem {
    let version = "Version: ".to_string() + option_env!("APP_VERSION").unwrap_or("Unknown");
    MenuItem::new(version, false, None)
}

fn open_item(hotkey: (Option<Modifiers>, Code)) -> MenuItem {
    MenuItem::with_id(
        "show_rustcast",
        "Toggle RustCast",
        true,
        Some(Accelerator::new(hotkey.0, hotkey.1)),
    )
}

fn refresh_item() -> MenuItem {
    MenuItem::with_id(
        "refresh_rustcast",
        "Refresh RustCast",
        true,
        Some(Accelerator::new(
            Some(Modifiers::SUPER),
            global_hotkey::hotkey::Code::KeyR,
        )),
    )
}

fn open_settings_item() -> MenuItem {
    MenuItem::with_id(
        "open_preferences",
        "Open Preferences",
        true,
        Some(Accelerator::new(Some(Modifiers::SUPER), Code::Comma)),
    )
}

fn quit_item() -> PredefinedMenuItem {
    PredefinedMenuItem::quit(Some("Quit RustCast"))
}

fn about_item(image: DynamicImage) -> PredefinedMenuItem {
    let about_metadata_builder = AboutMetadataBuilder::new()
        .name(Some("RustCast"))
        .version(Some(
            option_env!("APP_VERSION").unwrap_or("Unknown Version"),
        ))
        .authors(Some(vec!["Unsecretised".to_string()]))
        .credits(Some("Unsecretised".to_string()))
        .icon(Ico::from_rgba(image.as_bytes().to_vec(), image.width(), image.height()).ok())
        .website(Some("https://rustcast.umangsurana.com"))
        .license(Some("MIT"))
        .build();

    PredefinedMenuItem::about(Some("About RustCast"), Some(about_metadata_builder))
}
