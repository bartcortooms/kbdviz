# Architecture for Character Reference Tool (v0.2)

## Research Summary

### Existing Solutions
None of the existing tools do what we need:
- [rofimoji](https://github.com/fdw/rofimoji) - Character picker that INSERTS characters via wtype
- [charpicker-wayland](https://github.com/akhiljalagam/charpicker-wayland) - Emoji/character picker with clipboard
- [wofi-emoji](https://github.com/dln/wofi-emoji) - Emoji selector for Wayland

**Gap**: These tools insert characters for you. We want to SHOW key combinations so users can type them themselves.

### Relevant Libraries

**XKB/Compose:**
- [libxkbcommon compose API](https://xkbcommon.org/doc/current/group__compose.html) - Parse and query compose sequences
- Compose files at `/usr/share/X11/locale/*/Compose` - All dead key combinations
- We already use `xkbcommon = "0.8"` crate

**Virtual Keyboard (Not Needed):**
- [wrtype](https://docs.rs/wrtype) - Rust implementation of wtype
- [zwp-virtual-keyboard](https://github.com/grelltrier/zwp-virtual-keyboard) - Virtual keyboard protocol bindings
- **Decision**: Not needed since we're showing combinations, not typing

## Proposed Architecture

### Core Functionality

```
User Flow:
1. Press keybind → Open overlay (layer-shell)
2. Type 'e' in text input
3. Display results:
   é → AltGr+' e
   è → AltGr+` e
   ë → AltGr+" e
   ê → AltGr+^ e
   ...
4. User reads combo, closes tool (ESC)
5. User types combination themselves
```

### Technical Approach

#### Option 1: Parse Compose Files (Recommended)
**Pros:**
- Complete data from system compose files
- Accurate representation of what's actually available
- Can show alternative sequences (multiple ways to type same char)

**Cons:**
- Need to parse Compose file format
- May include sequences not in current layout

**Implementation:**
```rust
struct ComposeSequence {
    result: char,
    keys: Vec<KeySequence>,  // e.g., ["AltGr+'", "e"]
    description: String,      // e.g., "LATIN SMALL LETTER E WITH ACUTE"
}

// Parse /usr/share/X11/locale/en_US.UTF-8/Compose
// Build index: base_char -> Vec<ComposeSequence>
// e.g., 'e' -> [é, è, ë, ê, ẽ, ē, ė, ę, ...]
```

#### Option 2: Query XKB State Directly
**Pros:**
- Only shows what current layout actually supports
- No file parsing needed

**Cons:**
- Must enumerate all possible key combinations (expensive)
- Harder to represent multi-key sequences (dead keys)
- May miss compose sequences

#### Option 3: Hybrid (Best of Both)
1. Use libxkbcommon to get current keymap
2. Parse compose file for sequences
3. Filter compose sequences to only show what's available in current layout
4. Build base_char -> variants index

### UI Design

**Window:**
- Layer-shell overlay (already have code in `src/wayland/layer_shell.rs`)
- Small centered popup (~400x300px)
- Semi-transparent background
- Auto-focus on text input

**Layout:**
```
┌─────────────────────────────────┐
│  [Input: e_]                    │
├─────────────────────────────────┤
│  é  →  AltGr+' e                │
│  è  →  AltGr+` e                │
│  ë  →  AltGr+" e                │
│  ê  →  AltGr+^ e                │
│  ẽ  →  AltGr+~ e                │
│  ē  →  AltGr+- e                │
│  ė  →  AltGr+. e                │
│  ę  →  AltGr+; e                │
└─────────────────────────────────┘
```

**Interactions:**
- Type base character → Filter results
- ESC → Close tool
- Enter → Close tool (after reading combo)
- Arrow keys → Highlight result (future: copy combo to clipboard)

### Technology Stack Evaluation

**Current Stack (v0.1):**
- ✅ `xkbcommon` - XKB state and compose handling
- ✅ `wayland-client` + `smithay-client-toolkit` - Layer shell
- ✅ `tiny-skia` - 2D graphics (NO text rendering)
- ✅ `calloop` - Event loop
- ❌ iced - NOT actually used (found in Cargo.toml but main.rs uses custom WaylandApp)

**What We Actually Need:**
- Text input field with keyboard focus
- Scrollable list of results
- Simple text rendering (character + key combo)
- Layer-shell popup overlay
- Compose sequence parsing

**Framework Options:**

**Option 1: Current Stack + cosmic-text** ⭐ LIGHTWEIGHT
- Keep: smithay-client-toolkit + tiny-skia + calloop
- Add: [cosmic-text](https://docs.rs/cosmic-text) for text rendering (uses rustybuzz for shaping)
- Add: [piet-tiny-skia](https://docs.rs/piet-tiny-skia) for text integration
- Pros: Minimal dependencies, full control, similar to [fuzzel's approach](https://codeberg.org/dnkl/fuzzel)
- Cons: Need to build text input widget, scrolling logic from scratch
- Best for: Maximum performance and minimal footprint

**Option 2: egui + egui-layer-shell** ⭐ RECOMMENDED
- Use: [egui-layer-shell](https://github.com/kierandrewett/egui-layer-shell) (egui + Wayland layer-shell)
- Built-in: Text input, scrollable lists, layout, focus handling
- Text: Uses cosmic-text internally
- Pros: Has all widgets we need, immediate mode = simple state management
- Cons: Slightly heavier than Option 1
- Best for: Fast development with good UX

**Option 3: GTK4-rs + Layer Shell**
- Use: [gtk4-rs](https://github.com/gtk-rs/gtk4-rs) with gtk4-layer-shell
- Pros: Native widgets, accessibility, mature
- Cons: Heavy dependency, complex API, slower startup
- Best for: Integration with existing GTK apps (not our case)

**Option 4: Slint**
- Use: [Slint](https://slint.dev/) declarative UI
- Pros: Modern, clean API, good performance
- Cons: Need to check Wayland layer-shell support, adds DSL
- Best for: Complex UIs (overkill for us)

**RECOMMENDATION: Option 2 (egui + egui-layer-shell)**
- egui is lightweight and has ALL widgets we need built-in
- Immediate mode GUI = simpler state management (perfect for a popup tool)
- egui-layer-shell handles Wayland integration
- Text input, scrolling, keyboard focus all work out of the box
- Used by several Wayland tools already

**New Dependencies:**
- ✅ Keep `xkbcommon` for XKB/compose parsing
- ➕ Add `egui` + `egui-layer-shell` for UI
- ➕ Add compose file parser OR use libxkbcommon compose API
- ➕ Character index/search structure
- ❌ Remove `tiny-skia`, `calloop` (egui handles rendering/events)

### Implementation Plan

**Phase 1: Core Data** (Foundation)
1. Parse compose file or use libxkbcommon compose API
2. Build base_char -> variants index
3. Format key sequences for display (human-readable)

**Phase 2: UI Overhaul** (Remove v0.1 cruft)
1. Remove full keyboard visualization
2. Remove real-time input listener
3. Create simple popup window with text input + list
4. Wire up text input to filter results

**Phase 3: Polish**
1. Keybind to show/hide (global hotkey via niri config)
2. Auto-close on focus loss
3. Better formatting of key sequences
4. Handle edge cases (characters with multiple sequences)

**Phase 4: Stretch Goals**
1. While holding dead key, show all completions
2. Copy key sequence to clipboard
3. Show preview of character (large rendering)
4. Category browsing (all currency symbols, all math symbols, etc.)

## Key Decisions

1. **Parse compose files** - Most complete data source
2. **Keep iced** - Already works for overlay, has text input
3. **Layer-shell** - Already implemented, perfect for popup
4. **Show combinations, don't type** - Core UX decision
5. **Filter by base character** - Fast, intuitive UX

## Next Steps

1. Experiment with libxkbcommon compose API vs file parsing
2. Build simple proof-of-concept: parse some compose sequences
3. Create prototype UI: text input + results list
4. Test with real keyboard layout (us-altgr-intl)
