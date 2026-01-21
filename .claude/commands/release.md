---
description: Create a new release
---
Create a new release for kbdviz. The GitHub Action handles building and uploading artifacts automatically when a tag is pushed.

1. **Determine version**: Ask the user what version to release (e.g., patch 0.4.1 -> 0.4.2, minor 0.4.1 -> 0.5.0), or let them specify explicitly.

2. **Update version**: Edit `Cargo.toml` to update the version number.

3. **Update screenshot**: Run `./scripts/screenshot.sh` to update the README screenshot. Requires: grim, magick (ImageMagick), niri, jq.

4. **Generate release notes**: Get commits since last tag and summarize changes:
   ```bash
   git log $(git describe --tags --abbrev=0)..HEAD --oneline
   ```
   Write release notes with sections: New Features, Improvements, Bug Fixes, Documentation.

5. **Commit and tag**:
   ```bash
   git add -A
   git commit -m "Bump version to {version}"
   git tag v{version}
   ```

6. **Create release with notes** (before pushing tag):
   ```bash
   gh release create v{version} --title "v{version}" --notes "release notes here"
   ```

7. **Push commit and tag**:
   ```bash
   git push && git push origin v{version}
   ```
   The GitHub Action will add build artifacts to the existing release.

8. **Report**: Show the user the release URL: `https://github.com/bartcortooms/kbdviz/releases/tag/v{version}`
