# kbdviz Architecture

## Overview

kbdviz is a Wayland layer-shell overlay that shows how to type special characters using your keyboard layout. It reads the actual keymap from the compositor and builds an index of characters accessible via AltGr combinations.

## Components

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                              │
│  - Wayland connection and event loop                        │
│  - Layer-shell surface setup                                │
│  - KeyboardHandler: receives keymap from compositor         │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
     ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
     │ keyboard.rs │  │ compose.rs  │  │   ui.rs     │
     │             │  │             │  │             │
     │ XkbKeymap   │  │ComposeIndex │  │ CharRefUI   │
     │ wrapper     │  │ builder     │  │ renderer    │
     └─────────────┘  └─────────────┘  └─────────────┘
```

## Data Flow

1. **Keymap Reception**
   - Compositor sends keymap via `wl_keyboard::keymap` event
   - `KeyboardHandler::update_keymap()` receives the keymap string
   - `XkbKeymap::from_string()` parses it

2. **Index Building** (`ComposeIndex::build()`)
   - Scans keycodes 8-255
   - For each keycode, checks levels 0-3 (Base, Shift, AltGr, AltGr+Shift)
   - Converts keysyms to characters
   - Finds base character via Unicode NFD decomposition
   - Detects dead keys and adds their combinations
   - Builds HashMap: base_char → Vec<ComposeEntry>

3. **User Input**
   - User types a letter
   - `CharRefUI::handle_key_press()` updates filter
   - `ComposeIndex::find_variants()` returns matching entries

4. **Rendering** (`CharRefUI::render()`)
   - Draws to Pixmap using tiny-skia
   - Renders text using cosmic-text
   - Copies to Wayland buffer

## Key Structures

### ComposeEntry
```rust
pub struct ComposeEntry {
    pub character: String,      // "é"
    pub key_sequence: String,   // "AltGr-'"  or  "AltGr-`  e"
}
```

### ComposeIndex
```rust
pub struct ComposeIndex {
    index: HashMap<char, Vec<ComposeEntry>>,  // 'e' → [é, è, ë, ...]
}
```

## XKB Levels

| Level | Modifier | Example (US altgr-intl) |
|-------|----------|------------------------|
| 0 | None | a |
| 1 | Shift | A |
| 2 | AltGr | á |
| 3 | AltGr+Shift | Á (filtered if obvious) |

## Dead Key Handling

Dead keys are detected by checking if the keysym name starts with `dead_`:
- `dead_acute` → á, é, í, ó, ú, ...
- `dead_grave` → à, è, ì, ò, ù
- `dead_circumflex` → â, ê, î, ô, û
- etc.

The combinations are defined in `get_dead_key_combinations()`.

## Display Format

**Direct combination:**
```
å    AltGr    w
     ↑        ↑
     modifier key
```

**Dead key sequence:**
```
è    AltGr    `  e
     ↑        ↑  ↑
     modifier  dead key  base letter
```

## Dependencies

- **smithay-client-toolkit**: Wayland layer-shell, seat, keyboard protocols
- **xkbcommon**: Keymap parsing and keysym handling
- **tiny-skia**: 2D graphics primitives
- **cosmic-text**: Text shaping and rendering
- **unicode-normalization**: NFD decomposition for base character detection
- **calloop**: Event loop integration
