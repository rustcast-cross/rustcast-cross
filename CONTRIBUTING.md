# Welcome to the RustCast contributing guide!

Thank you for wanting to contribute to RustCast!

Some areas you can help work on are

1. Bug fixes
1. New Features
1. Help people in solving their github issues

For bug fixes, and helping people to solve their github issues: see
[https://github.com/rustcast-cross/rustcast-cross/issues] For features, see
[the planned features in the README](README.md) or
[the existing feature list](FEATURES.md)

## Code Guidelines:

1. All code must be formatted with `cargo fmt`
1. Code must not be malicious or be intended to harm someones device
1. All features added must work as intended
1. Code must compile...
1. A video recording / screenshot would be an added bonus in getting your pull
   request merged faster.
1. Being warning-free with pedantic lints (`clippy -- "-Wclippy::pedantic"`) is nice, but not
   necessary

## Codebase:

```
.
├── assets
│   ├── icon              # The icon
|   │   ├── icon_opt.svg  # Same stuff as icon.svg, minus the inkscape metadata
|   │   ├── icon.svg      # The icon SVG (with inkscape metadata)
|   │   ├── icon64.png    # The icon bitmap in various sizes
|   │   ├── icon128.png   # |
|   │   ├── icon256.png   # |
|   │   ├── icon512.png   # |
|   |
│   └── macos         # MacOS stuff, you can look deeper if it's relevant to what you're doing
├── bundling          # MacOS stuff
│   ├── entitlements.plist
│   ├── icon.icns
│   └── Info.plist
├── debian            # Debian stuff
├── docs              # Website and documentation related stuff. If something new is added to config,
│                     # then modify this as well before PR-ing
├── Cargo.lock 
├── Cargo.toml
├── CONTRIBUTING.md   # Contributing guidelines and codebase structure
├── EXTENSIONS.md     # Discussions about extensions implementation
├── LICENSE.md        # License file
├── README.md         # Readme file
├── FEATURES.md       # List of features currently implemented that should be updated when new 
├── scripts           # Fancy scripts for building on macos (presumably)
└── src
    ├── app
    │   ├── apps.rs             # Logic for the "apps" / commands that rustcast can perform
    │   ├── menubar.rs          # All the code related to the tray icon / menu bar icon
    │   ├── pages.rs            # TODO: convert to pages/mod.rs for consistency
    │   ├── pages               # Definitions for "pages" of the app
    │   │   ├── clipboard.rs
    │   │   ├── common.rs
    │   │   ├── emoji.rs
    │   │   ├── prelude.rs      # Prelude for individual pages
    │   │   └── settings.rs
    │   └── tile                # Logic for the tile (rustcast window)
    │       ├── mod.rs
    │       ├── search_query.rs
    │       ├── elm.rs          # Logic for the elm architecture of the rustcast window (New and View)
    │       └── update.rs       # Logic for the updating (elm architecture update) of the rustcast window
    ├── app_finding             # All the logic for app discovery
    |   ├── mod.rs
    │   ├── linux.rs            # Platform specific code
    │   ├── macos.rs            # |
    │   └── windows.rs          # |
    ├── config                  # Where configs are defined
    |   ├── mod.rs
    |   ├── include_patterns.rs # Custom serde parsers
    |   └── patterns.rs         # | 
    ├── cross_platform          # Stuff that's platform specific
    │   ├── mod.rs
    │   ├── linux.rs
    │   ├── macos
    │   |   ├── mod.rs
    |   |   └── haptics.rs      # Haptics stuff, ask secretised if you don't understand this stuff
    │   └── windows
    │       ├── mod.rs
    │       ├── appicon.rs      # Some code for extracting app icons (slightly wonky)
    │       └── app_finding.rs  # TODO: move this to the right place
    ├── unit_conversion         # Stuff for converting between units         
    │   ├── mod.rs
    │   └── defs.rs
    ├── app.rs                  # All code related to the app
    ├── calculator.rs           # Calculator logic 
    ├── commands.rs             # Logic for different commands
    ├── clipboard.rs            # Logic for the clipboard history feature of rustcast
    ├── icon.rs                 # Bundles icons into the binary
    ├── styles.rs               # Styles for the app
    ├── main.rs                 # Start app
    └── utils.rs                # Common functions that are used across files
```
