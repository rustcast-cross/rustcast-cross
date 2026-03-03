# Buffer Rules and Theme Config:

## Default config
```toml
{{#include ../default.toml:15:20}}
```
- `text_color`   
  This is the text color that rustcast should use inside the app with the format
  `text_color = [R, G, B]`.

- `background_color`  
  This is the background color that rustcast should use inside the app with the same format as
  `text_color` but different name.

- `show_icons`
  Sets whether rustcast displays app icons or not.
  
- `show_scroll_bar`
  If false, stops rustcast from displaying a scrollbar in the app list.
