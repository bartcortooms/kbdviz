use xkbcommon::xkb;

/// Wrapper around XKB keymap for querying keyboard layout
pub struct XkbKeymap {
    keymap: xkb::Keymap,
}

impl XkbKeymap {
    /// Create from a keymap string (received from compositor)
    pub fn from_string(keymap_string: &str) -> Result<Self, String> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_string(
            &context,
            keymap_string.to_string(),
            xkb::KEYMAP_FORMAT_TEXT_V1,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .ok_or("Failed to parse keymap string")?;

        Ok(Self { keymap })
    }

    /// Get the XKB keymap
    pub fn keymap(&self) -> &xkb::Keymap {
        &self.keymap
    }
}
