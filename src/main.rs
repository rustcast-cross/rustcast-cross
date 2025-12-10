mod components;
mod macos;
mod update;

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use iced::Fill;
use iced::futures;
use iced::stream;
use iced::widget::text::LineHeight;
use iced::widget::{Column, operation, space, text_input};
use iced::window::{self, Id, Settings};
use iced::{Element, Subscription, Task, Theme};

// ------ Main function --------

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    {
        macos::set_activation_policy_accessory();
    }

    let manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(Some(Modifiers::ALT), Code::Space);
    manager
        .register_all(&[hotkey])
        .expect("Unable to register hotkey");

    iced::daemon(Tile::new, Tile::update, Tile::view)
        .subscription(Tile::subscription)
        .theme(Tile::theme)
        .run()
}

// ------ Main function end --------

// ------ MacOS Configuration functions end--------

// ------ Core logic --------

#[derive(Debug)]
pub struct App {
    open_command: Command,
    icon_path: PathBuf,
    name: String,
}

#[derive(Debug)]
struct Tile {
    query: String,
    theme: Theme,
    results: Vec<App>,
    visible: bool,
}

#[derive(Debug, Clone)]
enum Message {
    OpenWindow,
    SearchQueryChanged(String),
    KeyPressed(u32),
    HideWindow(Id),
    Nothing,
}

#[derive(Debug, Clone)]
pub enum Hotkeys {
    AltSpace,
    Esc,
    Nothing,
}

impl Hotkeys {
    pub fn from_u32_hotkey_id(id: u32) -> Hotkeys {
        match id {
            65598 => Hotkeys::AltSpace,
            _ => Hotkeys::Nothing,
        }
    }
}

pub fn default_settings() -> Settings {
    let mut sets = window::Settings::default();
    sets.resizable = false;
    sets.decorations = false;
    sets.minimizable = false;
    sets.level = window::Level::AlwaysOnTop;
    sets.transparent = true;
    sets.blur = true;
    sets.size = iced::Size {
        width: 500.,
        height: 65.,
    };

    sets
}

impl Tile {
    /// A base window
    fn new() -> (Self, Task<Message>) {
        let (id, open) = window::open(default_settings());
        let _ = window::run(id, |handle| {
            macos::macos_window_config(&handle.window_handle().expect("Unable to get window handle"));
        });

        (
            Self {
                theme: Theme::KanagawaWave,
                query: String::new(),
                results: vec![],
                visible: true,
            },
            Task::batch([open.map(|_| Message::OpenWindow)]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenWindow => {
                println!("OpenWindow");
                Task::none()
            }

            Message::SearchQueryChanged(input) => {
                self.query = input;
                Task::none()
            }

            Message::KeyPressed(hk_id) => match Hotkeys::from_u32_hotkey_id(hk_id) {
                Hotkeys::AltSpace => {
                    self.visible = !self.visible;
                    if self.visible {
                        Task::chain(window::open(default_settings()).1.map(|_| Message::OpenWindow), operation::focus("query"))
                    } else {
                        let to_close = window::latest().map(|x| x.unwrap());
                        to_close.map(|x| Message::HideWindow(x))

                    }
                }
                _ => Task::none(),
            },

            Message::HideWindow(a) => {
                window::close(a)
            }

            Message::Nothing => Task::none(),
        }
    }

    fn view(&self, wid: window::Id) -> Element<'_, Message> {
        if self.visible {
            let title_input = Column::new().push(
                text_input("Time to be productive!", &self.query)
                    .on_input(|a| Message::SearchQueryChanged(a))
                    .on_paste(|a| Message::SearchQueryChanged(a))
                    .id("query")
                    .width(Fill)
                    .padding(20)
                    .line_height(LineHeight::Relative(1.5)),
            );
            title_input.into()
        } else {
            space().into()
        }
    }

    fn theme(&self, _: window::Id) -> Option<Theme> {
        Some(self.theme.clone())
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([Subscription::run(handle_hotkeys), window::close_events().map(|a| Message::HideWindow(a))])
    }
}

fn handle_hotkeys() -> impl futures::Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                if event.state == HotKeyState::Pressed {
                    output.try_send(Message::KeyPressed(event.id)).unwrap();
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
}
