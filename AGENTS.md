# Agent Instructions

## Screenshot Script

Use `scripts/screenshot.sh` to capture the kbdviz UI for verification and documentation.

### Usage

```bash
./scripts/screenshot.sh [output_path]
```

Default output: `assets/screenshot.png`

### How it works

1. Builds the release binary
2. Launches kbdviz with `--anchor top-left --char e` (positioned at top-left, showing 'e' variants)
3. Captures the focused output using `grim`
4. Crops to window size (280x420) using ImageMagick

### Requirements

- `grim` - Wayland screenshot tool
- `magick` - ImageMagick for cropping
- `niri` - For getting focused output (niri compositor)
- `jq` - JSON parsing

### When to use

- **After UI changes**: Run the script to verify rendering looks correct
- **Before releases**: Update the screenshot for documentation
- **Debugging rendering issues**: Compare screenshots before/after changes

### Verifying UI changes

After making changes to `ui.rs` or text rendering:

```bash
# Build and capture
./scripts/screenshot.sh /tmp/test.png

# View the result (use your preferred image viewer)
# Check for:
# - Text rendering artifacts (black backgrounds, aliasing issues)
# - Correct colors and contrast
# - Proper layout and alignment
```
