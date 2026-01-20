use xkbcommon::xkb;

/// Simplified XKB manager for querying keyboard state
pub struct XkbManager {
    context: xkb::Context,
    keymap: xkb::Keymap,
    state: xkb::State,
}

impl XkbManager {
    /// Create a new XKB manager with system default keymap
    pub fn new() -> Result<Self, String> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_names(
            &context,
            "",
            "",
            "",
            "",
            None,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .ok_or("Failed to create keymap")?;

        let state = xkb::State::new(&keymap);

        Ok(Self {
            context,
            keymap,
            state,
        })
    }

    /// Get the XKB context
    pub fn context(&self) -> &xkb::Context {
        &self.context
    }

    /// Get the XKB keymap
    pub fn keymap(&self) -> &xkb::Keymap {
        &self.keymap
    }

    /// Get the XKB state
    pub fn state(&self) -> &xkb::State {
        &self.state
    }

    /// Get character for a keycode with specific modifiers
    pub fn get_char_for_keycode(&self, keycode: xkb::Keycode, _mods: &[&str]) -> Option<char> {
        // Clone state to avoid mutating original
        let test_state = xkb::State::new(&self.keymap);

        // Get the keysym for this keycode
        let keysym = test_state.key_get_one_sym(keycode);
        let utf32 = xkb::keysym_to_utf32(keysym);

        if utf32 != 0 {
            char::from_u32(utf32)
        } else {
            None
        }
    }
}
