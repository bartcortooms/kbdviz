#!/usr/bin/env bash
# Take a screenshot of kbdviz for documentation
# Requires: grim, magick (ImageMagick), niri, jq

set -e

cd "$(dirname "$0")/.."
OUTPUT="${1:-assets/screenshot.png}"

for cmd in grim magick niri jq; do
    command -v "$cmd" &>/dev/null || { echo "Missing: $cmd"; exit 1; }
done

# Get the focused output
FOCUSED_OUTPUT=$(niri msg -j focused-output | jq -r '.name')

# Build and run with char pre-set, anchored to top-left
cargo build --release --quiet
./target/release/kbdviz --anchor top-left --char e &
PID=$!
trap "kill $PID 2>/dev/null" EXIT

sleep 0.5

# Capture the focused output and crop to window size (280x420 at 0,0)
grim -l 0 -o "$FOCUSED_OUTPUT" - | magick - -crop 280x420+0+0 +repage "$OUTPUT"
echo "Saved: $OUTPUT"
