# Docs for rustcast-cross

> [!WARNING]
>
> These docs are probably incomplete and wonky. The most up to date docs are in the doc comments,
> which is kinda suboptimal, but it is what it is

## Default config

```toml
{{#include default.toml}}
```

## Config file
The path for the config file is
`~/.config/rustcast/config.toml`

## Intro to the config

A manual setup is not required, as rustcast will create the config file for you upon loading.
Deleting the file will also cause a recreation of the file when rustcast launches. If the file gets
deleted, rustcast will make a new file based on the default config

If your config file has syntax errors (TOML syntax errors), rustcast will use to the default config.

If any parts of the file are missing, the default values are used. This also includes minor naming
errors such as `background_color` being `background_colour`

Here is the default config.toml for quick reference: 

```toml
toggle_hotkey = "ALT+SPACE"
clipboard_hotkey = "SUPER+SHIFT+C"
placeholder = "Time to be productive!"
search_url = "https://google.com/search?q=%s"
haptic_feedback = false
show_trayicon = true
shells = []
log_path = "/tmp/rustcast.log"

[buffer_rules]
clear_on_hide = true
clear_on_enter = true

[theme]
text_color = [0.95, 0.95, 0.96]
background_color = [0.0, 0.0, 0.0]
blur = false
show_icons = true
show_scroll_bar = true
```


