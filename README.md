# Keyboard Character Discovery Tool (kbdviz)

A "fuzzel"-like overlay for Wayland/niri that helps you discover how to type special characters (like ë, é, ñ, etc.) using your keyboard layout.

## Project Vision

Transform kbdviz into an unobtrusive, discoverable tool for finding key combinations to type special characters:

**Core Requirements:**
- **Unobtrusive**: Easily shown and dismissed (like fuzzel) without disturbing current window layouts
- **Fast Lookup**: Type a base character (e.g., `e`) to see all variants with their key combinations
- **Layout-Aware**: Show only characters available in your current keyboard layout
- **Interactive** (stretch goal): While holding down a dead key, show what characters you can complete

**Use Cases:**
- Need to type ë? → Open tool, type `e`, see "ë → AltGr+" then e", close tool, type combination
- Quick reference for dead key combinations without trial-and-error
- Learn combinations through repeated use

## Current State (v0.1 - Full Keyboard Visualizer)

The current implementation is a full keyboard layout visualizer with real-time highlighting:
- Visual ANSI 104-key keyboard display
- XKB integration showing all modifier combinations
- Real-time key press highlighting via Wayland seat protocol
- Dead key support with clickable popups
- Always-on-top overlay mode

**Limitation**: Too obtrusive for quick character lookup - takes up significant screen space and always visible.

## Planned Features (v0.2 - Character Reference Tool)

Transform into a fuzzel-like character reference tool:
- **Quick Launch**: Show/hide with keybind (like fuzzel)
- **Base Character Input**: Type a letter (e.g., `e`) in the tool
- **Variant Display**: Show all variants (é è ë ê ẽ ē ė ę...) with their key combinations
- **Compact Display**: Small popup showing only relevant characters
- **Fast**: One keypress to filter, read combo, close, type combination yourself
- **Learning**: Repeated use helps memorize common combinations

## Building

```bash
cd kbdviz
cargo build --release
```

## Running

```bash
cargo run --release
```

Or after building:

```bash
./target/release/kbdviz
```

## Usage

- **View Keyboard Layout**: The application displays your current XKB keyboard layout automatically in an overlay window
- **Real-Time Visualization**: As you type (in ANY application), the keyboard visualizer shows which keys are being pressed
- **Dead Keys**: Dead keys are highlighted in a distinct purple color
- **See Combinations**: Click on any dead key to see a popup showing all possible character combinations
- **Close**: Press **Escape** to exit the visualizer
- **Overlay Behavior**: The window is transparent, borderless, and always on top - perfect for keeping it visible while working

## How It Works

kbdviz connects to your Wayland compositor at two levels:

1. **XKB Layout Reading**: Uses libxkbcommon to read your keyboard layout configuration
2. **Real-Time Input Monitoring**: Uses Wayland seat protocol to listen for keyboard events system-wide

The application runs as a standalone overlay that monitors all keyboard input, making it perfect for:
- Learning keyboard layouts
- Discovering dead key combinations
- Recording keyboard demos or tutorials
- Debugging keyboard issues

## Project Structure

```
kbdviz/
├── Cargo.toml
└── src/
    ├── main.rs                  # Entry point with overlay configuration
    ├── keyboard/
    │   ├── mod.rs
    │   ├── layout.rs           # Keyboard layout data structures
    │   ├── xkb_state.rs        # XKB state management
    │   └── geometry.rs         # ANSI 104-key geometry
    ├── wayland/
    │   ├── mod.rs
    │   └── input_listener.rs   # Wayland keyboard event listener
    └── ui/
        ├── mod.rs
        ├── app.rs              # Main application state with subscriptions
        ├── keyboard_view.rs    # Keyboard rendering
        ├── key.rs              # Individual key widget
        ├── deadkey_popup.rs    # Dead key popup
        └── theme.rs            # Colors and styling
```

## Technology Stack

- **iced**: Pure Rust GUI framework with excellent Wayland support
- **xkbcommon**: XKB layout handling and keymap parsing
- **wayland-client**: Wayland protocol support
- **smithay-client-toolkit**: High-level Wayland helpers for seat and keyboard handling
- **async-channel**: Async event channel for Wayland→iced communication
- **calloop**: Event loop for Wayland dispatcher

## Configuration

### Window Settings

The overlay can be customized in `src/main.rs`:

```rust
.window_size(Size::new(1200.0, 500.0))  // Window size
.transparent(true)                       // Transparent background
.decorations(false)                      // No window decorations
.level(window::Level::AlwaysOnTop)       // Always on top
```

### Themes

The color scheme can be customized in `src/ui/theme.rs`. The default theme uses semi-transparent backgrounds for overlay effect:

```rust
background: Color::from_rgba(0.11, 0.11, 0.13, 0.95),  // 95% opacity
```

### Keyboard Layouts

To visualize a specific keyboard layout instead of the system default, modify `src/ui/app.rs`:

```rust
// In the new() function:
let xkb_manager = XkbManager::from_layout("us", "altgr-intl")
    .expect("Failed to initialize XKB manager");
```

Common layouts:
- `("us", "")` - US QWERTY
- `("us", "altgr-intl")` - US International with AltGr
- `("de", "")` - German
- `("fr", "")` - French

## Requirements

- Rust 1.70 or later
- Wayland compositor
- libxkbcommon
- Running Wayland session (not X11)

## Future Enhancements

- Toggle visibility with a global hotkey
- Support for switching between multiple layouts at runtime
- ISO and JIS keyboard layout support
- Export keyboard layout diagrams
- Modifier key state indicators
- Custom key label editing
- Configuration file support

## Troubleshooting

### "Failed to connect to Wayland"

Make sure you're running under a Wayland session. You can check with:
```bash
echo $XDG_SESSION_TYPE
```

### "Wayland keyboard listener error"

Ensure your compositor supports the wl_seat protocol (most modern compositors do). The visualizer will still work for layout viewing even if real-time highlighting fails.

### Key highlighting not working

The Wayland listener runs in a background thread. Check the console output for any initialization errors. The visualizer may need seat access permissions depending on your compositor configuration.

## License

This project was created as part of a demonstration of Rust GUI development with iced and Wayland integration.
