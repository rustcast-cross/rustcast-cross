//! This handles the update logic for the tile (AKA rustcast's main window)
use std::cmp::min;
use std::fs;
use std::time::Duration;

use iced::Task;
use iced::widget::operation;
use iced::window;

use crate::app::DEFAULT_WINDOW_HEIGHT;
use crate::app::RUSTCAST_DESC_NAME;
use crate::app::WINDOW_WIDTH;
use crate::app::apps::App;
use crate::app::default_settings;
use crate::calculator::Expression;
use crate::commands::Function;
use crate::{
    app::{Message, Page, tile::Tile},
    macos::focus_this_app,
};

pub fn handle_update(tile: &mut Tile, message: Message) -> Task<Message> {
    match message {
        Message::OpenWindow => {
            tile.capture_frontmost();
            focus_this_app();
            tile.focused = true;
            Task::none()
        }

        Message::SearchQueryChanged(input, id) => {
            tile.query_lc = input.trim().to_lowercase();
            tile.query = input;
            let prev_size = tile.results.len();
            if tile.query_lc.is_empty() && tile.page == Page::Main {
                tile.results = vec![];
                return window::resize(
                    id,
                    iced::Size {
                        width: WINDOW_WIDTH,
                        height: DEFAULT_WINDOW_HEIGHT,
                    },
                );
            } else if tile.query_lc == "randomvar" {
                let rand_num = rand::random_range(0..100);
                tile.results = vec![App {
                    open_command: Function::RandomVar(rand_num),
                    desc: "Easter egg".to_string(),
                    icons: None,
                    name: rand_num.to_string(),
                    name_lc: String::new(),
                }];
                return window::resize(
                    id,
                    iced::Size {
                        width: WINDOW_WIDTH,
                        height: 55. + DEFAULT_WINDOW_HEIGHT,
                    },
                );
            } else if tile.query_lc.ends_with("?") {
                tile.results = vec![App {
                    open_command: Function::GoogleSearch(tile.query.clone()),
                    icons: None,
                    desc: "Search".to_string(),
                    name: format!("Search for: {}", tile.query),
                    name_lc: String::new(),
                }];
                return window::resize(
                    id,
                    iced::Size::new(WINDOW_WIDTH, 55. + DEFAULT_WINDOW_HEIGHT),
                );
            } else if tile.query_lc == "cbhist" {
                tile.page = Page::ClipboardHistory
            } else if tile.query_lc == "main" {
                tile.page = Page::Main
            }

            tile.handle_search_query_changed();

            if tile.results.is_empty()
                && let Some(res) = Expression::from_str(&tile.query)
            {
                tile.results.push(App {
                    open_command: Function::Calculate(res),
                    desc: RUSTCAST_DESC_NAME.to_string(),
                    icons: None,
                    name: res.eval().to_string(),
                    name_lc: "".to_string(),
                });
            }
            let new_length = tile.results.len();

            let max_elem = min(5, new_length);

            if prev_size != new_length && tile.page == Page::Main {
                std::thread::sleep(Duration::from_millis(30));

                window::resize(
                    id,
                    iced::Size {
                        width: WINDOW_WIDTH,
                        height: ((max_elem * 55) + DEFAULT_WINDOW_HEIGHT as usize) as f32,
                    },
                )
            } else if tile.page == Page::ClipboardHistory {
                let element_count = min(tile.clipboard_content.len(), 5);
                window::resize(
                    id,
                    iced::Size {
                        width: WINDOW_WIDTH,
                        height: ((element_count * 55) + DEFAULT_WINDOW_HEIGHT as usize) as f32,
                    },
                )
            } else {
                Task::none()
            }
        }

        Message::ClearSearchQuery => {
            tile.query_lc = String::new();
            tile.query = String::new();
            Task::none()
        }

        Message::ReloadConfig => {
            tile.config = toml::from_str(
                &fs::read_to_string(
                    std::env::var("HOME").unwrap_or("".to_owned())
                        + "/.config/rustcast/config.toml",
                )
                .unwrap_or("".to_owned()),
            )
            .unwrap();

            Task::none()
        }

        Message::KeyPressed(hk_id) => {
            if hk_id == tile.open_hotkey_id {
                tile.visible = !tile.visible;
                if tile.visible {
                    Task::chain(
                        window::open(default_settings())
                            .1
                            .map(|_| Message::OpenWindow),
                        operation::focus("query"),
                    )
                } else {
                    let to_close = window::latest().map(|x| x.unwrap());
                    Task::batch([
                        to_close.map(Message::HideWindow),
                        Task::done(if tile.config.buffer_rules.clone().clear_on_hide {
                            Message::ClearSearchQuery
                        } else {
                            Message::_Nothing
                        }),
                    ])
                }
            } else {
                Task::none()
            }
        }

        Message::RunFunction(command) => {
            command.execute(&tile.config, &tile.query);

            if tile.config.buffer_rules.clear_on_enter {
                window::latest()
                    .map(|x| x.unwrap())
                    .map(Message::HideWindow)
                    .chain(Task::done(Message::ClearSearchQuery))
            } else {
                Task::none()
            }
        }

        Message::HideWindow(a) => {
            tile.restore_frontmost();
            tile.visible = false;
            tile.focused = false;
            tile.page = Page::Main;
            Task::batch([window::close(a), Task::done(Message::ClearSearchResults)])
        }
        Message::ClearSearchResults => {
            tile.results = vec![];
            Task::none()
        }
        Message::WindowFocusChanged(wid, focused) => {
            tile.focused = focused;
            if !focused {
                Task::done(Message::HideWindow(wid)).chain(Task::done(Message::ClearSearchQuery))
            } else {
                Task::none()
            }
        }

        Message::ClipboardHistory(clip_content) => {
            tile.clipboard_content.insert(0, clip_content);
            Task::none()
        }

        Message::_Nothing => Task::none(),
    }
}
