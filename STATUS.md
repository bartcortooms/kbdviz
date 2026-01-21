# kbdviz Development Status

## Current State: Functional

The character reference tool is working with all core features implemented.

### What Works

- **Keyboard layout from compositor**: Reads the actual XKB keymap from Wayland
- **AltGr combinations**: Scans all AltGr and AltGr+Shift levels
- **Dead key support**: Detects dead keys and shows their combinations
- **Text rendering**: cosmic-text integration working
- **Base character filtering**: Type a letter to see all variants
- **Smart filtering**: Filters out obvious uppercase variants (É from AltGr+Shift+e)
- **Clean UI**: Readable modifier display, prominent key indicators

### File Status

| File | Status | Description |
|------|--------|-------------|
| src/main.rs | Complete | Wayland app, compositor keymap handling |
| src/keyboard.rs | Complete | XKB keymap wrapper |
| src/compose.rs | Complete | Scans keymap for characters, dead key support |
| src/ui.rs | Complete | Rendering with cosmic-text |

### Technical Details

**Keymap Scanning:**
- Iterates keycodes 8-255
- Gets keysyms at levels 0 (base), 1 (Shift), 2 (AltGr), 3 (AltGr+Shift)
- Converts keysyms to UTF-32 characters
- Uses Unicode NFD decomposition to find base characters

**Dead Keys:**
- Detects keysyms starting with `dead_`
- Maps dead key types to character combinations
- Supports: acute, grave, circumflex, diaeresis, tilde, cedilla, ogonek, caron, breve, macron, abovedot, abovering, stroke

**Filtering:**
- Skips ASCII letters at base levels
- Skips obvious Shift capitalizations
- Filters AltGr+Shift characters that are just uppercase of AltGr characters

## Potential Improvements

- [ ] Auto-resize window height based on result count
- [ ] Scrolling for many results
- [ ] Global hotkey integration
- [ ] Search by character name (e.g., "euro")
- [ ] Show Unicode codepoint
