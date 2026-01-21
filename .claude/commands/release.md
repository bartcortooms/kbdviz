---
description: Create a new release
---
Create a new release for kbdviz. The GitHub Action handles building and uploading artifacts automatically when a tag is pushed.

1. **Determine version**: Ask the user what version to release (e.g., patch 0.4.1 -> 0.4.2, minor 0.4.1 -> 0.5.0), or let them specify explicitly.

2. **Update version**: Edit `Cargo.toml` to update the version number.

3. **Update screenshot**: Run `./scripts/screenshot.sh` to update the README screenshot. This requires a Wayland session with grim and wtype installed. The script launches kbdviz anchored to the top-left corner, types 'e' to show variants, and captures automatically.

4. **Commit and tag**:
   - Stage all changes
   - Commit with message "Bump version to {version}"
   - Create a git tag `v{version}`
   - Push the commit and tag to origin

5. **Report**: The GitHub Action will build and create the release automatically. Show the user the release URL: `https://github.com/bartcortooms/kbdviz/releases/tag/v{version}`
