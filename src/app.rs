//! Main logic for the app
use crate::clipboard::ClipBoardContentType;
use crate::commands::Function;

pub mod apps;
pub mod menubar;
pub mod tile;

use iced::window::{self, Id, Settings};

/// The default window width
pub const WINDOW_WIDTH: f32 = 500.;

/// The default window height
pub const DEFAULT_WINDOW_HEIGHT: f32 = 65.;

/// The rustcast descriptor name to be put for all rustcast commands
pub const RUSTCAST_DESC_NAME: &str = "RustCast";

/// The different pages that rustcast can have / has
#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Main,
    ClipboardHistory,
}

/// The message type that iced uses for actions that can do something
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    OpenWindow,
    SearchQueryChanged(String, Id),
    KeyPressed(u32),
    HideWindow(Id),
    RunFunction(Function),
    ReturnFocus,
    ClearSearchResults,
    WindowFocusChanged(Id, bool),
    ClearSearchQuery,
    ReloadConfig,
    SwitchToPage(Page),
    ClipboardHistory(ClipBoardContentType),
}

/// The window settings for rustcast
pub fn default_settings() -> Settings {
    Settings {
        resizable: false,
        decorations: false,
        minimizable: false,
        level: window::Level::AlwaysOnTop,
        transparent: true,
        blur: true,
        size: iced::Size {
            width: WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
        },
        ..Default::default()
    }
}
