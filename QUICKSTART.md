# kbdviz Quick Start

## Build and Run

```bash
cargo build --release
./target/release/kbdviz
```

## Controls

- **Type a letter** (e, a, o, i, u, n, c, s, z, l, y) to see variants
- **Backspace** - Clear input
- **ESC** - Close the window

## What You'll See

A centered overlay showing:

1. **Input area** at top - shows the letter you typed
2. **Results** below - each row shows:
   - The special character (large)
   - The modifier (AltGr or AltGr-Shift)
   - The key(s) to press

### Direct Combinations

```
å    AltGr    w
```
Hold AltGr, press w.

### Dead Key Sequences

```
è    AltGr    `  e
```
Press AltGr+` (releases dead key), then press e.

## Example Session

1. Run `./target/release/kbdviz`
2. Type `e`
3. See variants: é, è, ë, ê, €, etc.
4. Note the key combinations
5. Press ESC to close
6. Type the combination in your application

## Tips

- The tool reads your actual keyboard layout from the compositor
- Works best with layouts that have AltGr combinations (e.g., `us(altgr-intl)`)
- Dead key sequences show two keys separated by space
