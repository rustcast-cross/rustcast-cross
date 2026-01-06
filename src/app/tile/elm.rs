//! This module handles the logic for the new and view functions according to the elm
//! architecture. If the subscription function becomes too large, it should be moved to this file
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::text::LineHeight;
use iced::widget::{Column, Scrollable, space};
use iced::window;
use iced::{Element, Task};
use iced::{Length::Fill, widget::text_input};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

pub fn default_app_paths() -> Vec<String> {
    let user_local_path = std::env::var("HOME").unwrap() + "/Applications/";

    let paths = vec![
        "/Applications/".to_string(),
        user_local_path,
        "/System/Applications/".to_string(),
        "/System/Applications/Utilities/".to_string(),
    ];

    paths
}

use crate::app::apps::AppCommand;
use crate::{
    app::{Message, Page, apps::App, default_settings, tile::Tile},
    config::Config,
    macos::{self, transform_process_to_ui_element},
    utils::get_installed_apps,
};

/// Initialise the base window
pub fn new(keybind_id: u32, config: &Config) -> (Tile, Task<Message>) {
    let (id, open) = window::open(default_settings());

    let open = open.discard().chain(window::run(id, |handle| {
        macos::macos_window_config(&handle.window_handle().expect("Unable to get window handle"));
        // should work now that we have a window
        transform_process_to_ui_element();
    }));

    let store_icons = config.theme.show_icons;

    let paths = default_app_paths();

    let mut options: Vec<App> = paths
        .par_iter()
        .map(|path| get_installed_apps(path, store_icons))
        .flatten()
        .collect();

    options.extend(config.shells.iter().map(|x| x.to_app()));
    options.extend(App::basic_apps());
    options.par_sort_by_key(|x| x.name.len());

    (
        Tile {
            query: String::new(),
            query_lc: String::new(),
            prev_query_lc: String::new(),
            results: vec![],
            options,
            visible: true,
            frontmost: None,
            focused: false,
            config: config.clone(),
            theme: config.theme.to_owned().into(),
            open_hotkey_id: keybind_id,
            clipboard_content: vec![],
            page: Page::Main,
        },
        Task::batch([open.map(|_| Message::OpenWindow)]),
    )
}

pub fn view(tile: &Tile, wid: window::Id) -> Element<'_, Message> {
    if tile.visible {
        let title_input = text_input(tile.config.placeholder.as_str(), &tile.query)
            .on_input(move |a| Message::SearchQueryChanged(a, wid))
            .on_paste(move |a| Message::SearchQueryChanged(a, wid))
            .on_submit_maybe({
                if !tile.results.is_empty() {
                    match tile.results.first().unwrap().to_owned().open_command {
                        AppCommand::Function(func) => Some(Message::RunFunction(func)),
                        AppCommand::Message(msg) => Some(msg),
                        AppCommand::Display => None,
                    }
                } else {
                    None
                }
            })
            .id("query")
            .width(Fill)
            .line_height(LineHeight::Relative(1.5))
            .padding(20);

        let scrollbar_direction = if tile.config.theme.show_scroll_bar {
            Direction::Vertical(Scrollbar::new().width(2).scroller_width(2))
        } else {
            Direction::Vertical(Scrollbar::hidden())
        };
        match tile.page {
            Page::Main => {
                let mut search_results = Column::new();
                for result in &tile.results {
                    search_results = search_results.push(result.render(&tile.config.theme));
                }

                let scrollable = Scrollable::with_direction(search_results, scrollbar_direction);
                Column::new().push(title_input).push(scrollable).into()
            }
            Page::ClipboardHistory => {
                let mut clipboard_history = Column::new();
                for result in &tile.clipboard_content {
                    clipboard_history = clipboard_history.push(result.render_clipboard_item());
                }
                let scrollable = Scrollable::with_direction(clipboard_history, scrollbar_direction);
                Column::new().push(title_input).push(scrollable).into()
            }
        }
    } else {
        space().into()
    }
}
