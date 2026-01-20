# kbdviz v0.2 - Quick Start

## How to Use

```bash
# Build
cargo build --release

# Run
./target/release/kbdviz
```

## What You'll See

1. **A centered window** with colored boxes (text rendering is placeholder mode)
2. **Blue box at top** - Instructions area
3. **Dark box in middle** - Input area
   - When you type a letter, a green indicator appears
4. **Purple boxes below** - Results (each box = one character variant)

## Controls

- **Type any letter** (e.g., `e`, `a`, `o`, `i`, `u`, `n`, `c`, `s`)
  - Only ONE letter at a time (it replaces the previous one)
  - Console will show all the variants for that letter
- **Backspace** - Clear input
- **ESC** - Close the window

## Understanding the Console Output

Since text rendering is still placeholder mode, **watch the terminal output**:

```
=== RENDERING UI ===
INSTRUCTIONS: Press ESC or Q to close
Type a letter (e.g., 'e', 'a', 'o') to see variants
Current input: 'e'

Found 8 variants for 'e':
  é → AltGr+' e
  è → AltGr+` e
  ë → AltGr+" e
  ê → AltGr+^ e
  ẽ → AltGr+~ e
  ē → AltGr+- e
  ė → AltGr+. e
  ę → AltGr+; e
```

Each purple box in the window represents one of these variants.

## Example Usage

1. Run the app
2. Press `e`
3. See 8 purple boxes appear (one for each variant)
4. Check console - it shows: é, è, ë, ê, ẽ, ē, ė, ę with their key combos
5. Press `a` - boxes change, console shows a variants
6. Press ESC - window closes

## Currently Working

✅ Wayland layer-shell popup
✅ Keyboard input capture
✅ Character filtering (single letter)
✅ 70+ special characters indexed
✅ Console output shows all info
✅ ESC to close
✅ Visual feedback (colored boxes)

## Not Yet Working

🚧 Actual text rendering in the window (using placeholders + console)
🚧 Scrolling for long lists
🚧 System Compose file parsing (using hardcoded sequences)

## Covered Characters

**Letters with variants:**
- e, a, o, u, i (8-7 variants each)
- n, c, s, y, z, l (2-3 variants each)

**Special:**
- Currency: € £ ¥ ¢
- Ligatures: æ œ ß

Try typing these letters to see their variants!

## Next Steps

1. Add proper cosmic-text rendering → See text in window
2. Parse `/usr/share/X11/locale/*/Compose` → Get ALL system sequences
