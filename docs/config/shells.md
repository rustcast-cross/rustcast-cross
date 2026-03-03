# Shell Commands

A shell command can do as much as modes can, but they can have custom icons + different display names / search names

A shell command is configured slightly differently:
```toml
[[shells]] # note that its encased in double square brackets

command = "osascript -e 'tell application \"Spotify\" to play next track'"
icon_path = "/Applications/Spotify.app/Contents/Resources/AppIcon.icns"
alias = "Next Spotify Song" 
alias_lc = "next"
```

- `command` is the shell command to run (can be a shell script as well, which is useful for longer scripts)
- `icon_path` can point to a png, jpg, or icns
- `alias` is the text displayed 
- `alias_lc` is the text used to search

As seen in the below image:

- The spotify icon is loaded from `icon_path`
- The yellow rectangle is whats used to search, aka `alias_lc` (Ideally should be fully lowercased)
- The `alias` is the one in the dark red rectangle
<img width="1038" height="380" alt="image" src="https://github.com/user-attachments/assets/e06ae0cf-b390-4111-a01c-ae87cb794e9e" />

