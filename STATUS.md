# kbdviz v0.2 Development Status

## What Works Now

### ✅ Core Architecture
- **Wayland Layer-Shell integration**: Creates centered popup overlay
- **Keyboard input handling**: Captures keypresses, ESC to exit
- **Character index**: 70+ special characters with their key combinations
  - Covers e, a, o, u, i, n, c, s, y, z, l variants
  - Currency symbols (€, £, ¥, ¢)
  - Special ligatures (æ, œ, ß)
- **Input filtering**: Type a letter to filter results
- **Compiles and runs**: Application starts and displays window

### 🚧 Currently Limited
- **No text rendering**: Text drawing functions are placeholders
  - You'll see a colored window but no actual text
  - cosmic-text is included but not yet integrated
- **Hardcoded compose sequences**: Not parsing from system Compose files yet
- **Basic keyboard handling**: Only handles alphanumeric input and backspace

## Next Steps

### Priority 1: Make it Usable
1. **Implement cosmic-text rendering** (src/ui.rs:131-137)
   - Integrate FontSystem and SwashCache properly
   - Render title, input text, and results
   - This will make the UI actually visible and usable

### Priority 2: Better Data
2. **Parse system Compose file** (src/compose.rs:24)
   - Read from `/usr/share/X11/locale/*/Compose`
   - OR use libxkbcommon compose API
   - Get ALL available compose sequences, not just hardcoded ones

### Priority 3: Polish
3. **Improved keyboard handling**
   - Handle Shift for uppercase
   - Better text editing (cursor movement, etc.)
   - Show only uppercase OR lowercase variants based on input

4. **Better UX**
   - Scrolling for long result lists
   - Visual feedback for input focus
   - Better colors/styling

## How to Test

```bash
# Build and run
cargo build --release
./target/release/kbdviz

# You should see a dark gray centered window
# Type letters (e, a, o, etc.) - input is captured but not visible yet
# Press ESC to close
```

## Architecture

```
main.rs                 - Wayland app setup, event loop
├── keyboard.rs         - XKB manager (simplified)
├── compose.rs          - Character index (base char -> variants)
└── ui.rs               - Rendering with tiny-skia + cosmic-text
    ├── CharRefUI       - Main UI struct
    ├── handle_key_press()  - Process keyboard input
    ├── render()        - Draw everything
    └── draw_text()     - TODO: Implement with cosmic-text
```

## Dependencies

- `smithay-client-toolkit` - Wayland layer-shell
- `tiny-skia` - 2D graphics
- `cosmic-text` - Text rendering (not fully integrated yet)
- `xkbcommon` - Keyboard layout queries
- `calloop` - Event loop

Total: ~13 dependencies (lightweight!)

## File Status

| File | Status | Notes |
|------|--------|-------|
| README.md | ✅ Updated | New vision documented |
| ARCHITECTURE.md | ✅ Complete | Framework comparison, plan |
| STATUS.md | ✅ This file | Current status |
| Cargo.toml | ✅ Updated | New dependencies |
| src/main.rs | ✅ Complete | Event loop working |
| src/keyboard.rs | ✅ Basic | XKB integration |
| src/compose.rs | 🚧 Hardcoded | Need to parse Compose file |
| src/ui.rs | 🚧 No text | Need cosmic-text integration |

## Known Issues

1. **Text is invisible** - draw_text() is a stub
2. **Limited character coverage** - only ~70 hardcoded sequences
3. **No visual feedback** - can't see what you're typing yet
4. **Keyboard handling is basic** - no modifiers, limited keys

## Quick Wins

To make this actually usable ASAP:

1. **30 min**: Integrate cosmic-text for basic text rendering
   - Get title and input text visible
   - Results list showing character + key combo

2. **1 hour**: Parse Compose file
   - Read `/usr/share/X11/locale/en_US.UTF-8/Compose`
   - Build index from actual system data
   - Cover ALL available characters

After these two tasks, the tool will be functional!
