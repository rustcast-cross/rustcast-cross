//! This module handles the logic for the tile, AKA rustcast's main window
mod elm;
mod update;

use crate::app::apps::App;
use crate::app::{Message, Page};
use crate::clipboard::ClipBoardContentType;
use crate::commands::Function;
use crate::config::Config;

use arboard::Clipboard;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};

use iced::futures::SinkExt;
use iced::window;
use iced::{
    Element, Subscription, Task, Theme, futures,
    keyboard::{self, key::Named},
    stream,
};

use objc2::rc::Retained;
use objc2_app_kit::NSRunningApplication;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use std::fs;
use std::time::Duration;

/// This is the base window, and its a "Tile"
/// Its fields are:
/// - Theme ([`iced::Theme`])
/// - Query (String)
/// - Query Lowercase (String, but lowercase)
/// - Previous Query Lowercase (String)
/// - Results (Vec<[`App`]>) the results of the search
/// - Options (Vec<[`App`]>) the options to search through
/// - Visible (bool) whether the window is visible or not
/// - Focused (bool) whether the window is focused or not
/// - Frontmost ([`Option<Retained<NSRunningApplication>>`]) the frontmost application before the window was opened
/// - Config ([`Config`]) the app's config
/// - Open Hotkey ID (`u32`) the id of the hotkey that opens the window
/// - Clipboard Content (`Vec<`[`ClipBoardContentType`]`>`) all of the cliboard contents
/// - Page ([`Page`]) the current page of the window (main or clipboard history)
#[derive(Debug, Clone)]
pub struct Tile {
    theme: iced::Theme,
    query: String,
    query_lc: String,
    prev_query_lc: String,
    results: Vec<App>,
    options: Vec<App>,
    visible: bool,
    focused: bool,
    frontmost: Option<Retained<NSRunningApplication>>,
    config: Config,
    open_hotkey_id: u32,
    clipboard_content: Vec<ClipBoardContentType>,
    page: Page,
}

impl Tile {
    /// Initialise the base window
    pub fn new(keybind_id: u32, config: &Config) -> (Self, Task<Message>) {
        elm::new(keybind_id, config)
    }

    /// This handles the iced's updates, which have all the variants of [Message]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        update::handle_update(self, message)
    }

    /// This is the view of the window. It handles the rendering of the window
    ///
    /// The rendering of the window size (the resizing of the window) is handled by the
    /// [`Tile::update`] function.
    pub fn view(&self, wid: window::Id) -> Element<'_, Message> {
        elm::view(self, wid)
    }

    /// This returns the theme of the window
    pub fn theme(&self, _: window::Id) -> Option<Theme> {
        Some(self.theme.clone())
    }

    /// This handles the subscriptions of the window
    ///
    /// The subscriptions are:
    /// - Hotkeys
    /// - Hot reloading
    /// - Clipboard history
    /// - Window close events
    /// - Keypresses (escape to close the window)
    /// - Window focus changes
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            Subscription::run(handle_hotkeys),
            Subscription::run(handle_hot_reloading),
            Subscription::run(handle_clipboard_history),
            window::close_events().map(Message::HideWindow),
            keyboard::listen().filter_map(|event| {
                if let keyboard::Event::KeyPressed { key, .. } = event {
                    match key {
                        keyboard::Key::Named(Named::Escape) => Some(Message::KeyPressed(65598)),
                        _ => None,
                    }
                } else {
                    None
                }
            }),
            window::events()
                .with(self.focused)
                .filter_map(|(focused, (wid, event))| match event {
                    window::Event::Unfocused => {
                        if focused {
                            Some(Message::WindowFocusChanged(wid, false))
                        } else {
                            None
                        }
                    }
                    window::Event::Focused => Some(Message::WindowFocusChanged(wid, true)),
                    _ => None,
                }),
        ])
    }

    /// Handles the search query changed event.
    ///
    /// This is separate from the `update` function because it has a decent amount of logic, and
    /// should be separated out to make it easier to test. This function is called by the `update`
    /// function to handle the search query changed event.
    pub fn handle_search_query_changed(&mut self) {
        let filter_vec: &Vec<App> = if self.query_lc.starts_with(&self.prev_query_lc) {
            self.prev_query_lc = self.query_lc.to_owned();
            &self.results
        } else {
            &self.options
        };

        let query = self.query_lc.clone();

        let mut exact: Vec<App> = filter_vec
            .par_iter()
            .filter(|x| match &x.open_command {
                Function::RunShellCommand(_, _) => x
                    .name_lc
                    .starts_with(query.split_once(" ").unwrap_or((&query, "")).0),
                _ => x.name_lc == query,
            })
            .cloned()
            .collect();

        let mut prefix: Vec<App> = filter_vec
            .par_iter()
            .filter(|x| match x.open_command {
                Function::RunShellCommand(_, _) => false,
                _ => x.name_lc != query && x.name_lc.starts_with(&query),
            })
            .cloned()
            .collect();

        exact.append(&mut prefix);
        self.results = exact;
    }

    /// Gets the frontmost application to focus later.
    pub fn capture_frontmost(&mut self) {
        use objc2_app_kit::NSWorkspace;

        let ws = NSWorkspace::sharedWorkspace();
        self.frontmost = ws.frontmostApplication();
    }

    /// Restores the frontmost application.
    #[allow(deprecated)]
    pub fn restore_frontmost(&mut self) {
        use objc2_app_kit::NSApplicationActivationOptions;

        if let Some(app) = self.frontmost.take() {
            app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
        }
    }
}

/// This is the subscription function that handles hot reloading of the config
fn handle_hot_reloading() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let content = fs::read_to_string(
            std::env::var("HOME").unwrap_or("".to_owned()) + "/.config/rustcast/config.toml",
        )
        .unwrap_or("".to_string());
        loop {
            let current_content = fs::read_to_string(
                std::env::var("HOME").unwrap_or("".to_owned()) + "/.config/rustcast/config.toml",
            )
            .unwrap_or("".to_string());

            if current_content != content {
                output.send(Message::ReloadConfig).await.unwrap();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}

/// This is the subscription function that handles hotkeys for hiding / showing the window
fn handle_hotkeys() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv()
                && event.state == HotKeyState::Pressed
            {
                output.try_send(Message::KeyPressed(event.id)).unwrap();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}

/// This is the subscription function that handles the change in clipboard history
fn handle_clipboard_history() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let mut clipboard = Clipboard::new().unwrap();
        let mut prev_byte_rep: Option<ClipBoardContentType> = None;

        loop {
            let byte_rep = if let Ok(a) = clipboard.get_image() {
                Some(ClipBoardContentType::Image(a))
            } else if let Ok(a) = clipboard.get_text() {
                Some(ClipBoardContentType::Text(a))
            } else {
                None
            };

            if byte_rep != prev_byte_rep
                && let Some(content) = &byte_rep
            {
                output
                    .send(Message::ClipboardHistory(content.to_owned()))
                    .await
                    .ok();
                prev_byte_rep = byte_rep;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}
