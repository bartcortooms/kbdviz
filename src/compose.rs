use crate::keyboard::XkbKeymap;
use std::collections::HashMap;

/// Represents a single compose sequence result
#[derive(Clone, Debug)]
pub struct ComposeEntry {
    pub character: String,
    pub key_sequence: String,
}

/// Index of base characters to their variants
pub struct ComposeIndex {
    // Maps base character (e.g., 'e') to all its variants
    index: HashMap<char, Vec<ComposeEntry>>,
}

impl ComposeIndex {
    /// Count of base characters with variants
    pub fn count(&self) -> usize {
        self.index.len()
    }

    /// Build the compose index from XKB keymap
    pub fn build(xkb: &XkbKeymap) -> Result<Self, String> {
        use xkbcommon::xkb;

        let mut index: HashMap<char, Vec<ComposeEntry>> = HashMap::new();
        let keymap = xkb.keymap();

        // Iterate through all keycodes (8-255 is the standard range)
        for keycode_raw in 8..256 {
            let keycode = xkb::Keycode::new(keycode_raw);

            // Skip if no key name
            if keymap.key_get_name(keycode).is_none() {
                continue;
            }

            // XKB levels: 0=Base, 1=Shift, 2=AltGr, 3=AltGr+Shift
            let num_levels = keymap.num_levels_for_key(keycode, 0);

            // Get level 2 character to compare with level 3
            let level2_char: Option<char> = if num_levels > 2 {
                let syms = keymap.key_get_syms_by_level(keycode, 0, 2);
                if !syms.is_empty() {
                    char::from_u32(xkb::keysym_to_utf32(syms[0]))
                } else {
                    None
                }
            } else {
                None
            };

            for level in 0..num_levels {
                // Get the keysyms for this level
                let syms = keymap.key_get_syms_by_level(keycode, 0, level);

                if syms.is_empty() {
                    continue;
                }

                let keysym = syms[0];

                // Convert keysym to UTF-32 character
                let utf32 = xkb::keysym_to_utf32(keysym);
                if let Some(ch) = char::from_u32(utf32) {
                    // Skip control characters and whitespace
                    if ch.is_control() || ch.is_whitespace() {
                        continue;
                    }

                    // Determine modifier prefix based on level
                    let mod_prefix = match level {
                        0 => {
                            // Skip basic ASCII letters at level 0 (no modifiers)
                            if ch.is_ascii_lowercase() {
                                continue;
                            }
                            ""
                        },
                        1 => {
                            // Skip uppercase ASCII at level 1 (Shift)
                            if ch.is_ascii_uppercase() {
                                continue;
                            }
                            "Shift+"
                        },
                        2 => "AltGr+",
                        3 => {
                            // Skip if this is just the uppercase of level 2
                            // (obvious Shift capitalization)
                            if let Some(l2_char) = level2_char {
                                if ch == l2_char.to_uppercase().next().unwrap_or(l2_char)
                                   && ch != l2_char {
                                    continue;
                                }
                            }
                            "AltGr+Shift+"
                        },
                        _ => continue,
                    };

                    // Get the physical key name (what key to press)
                    let key_name = keymap.key_get_name(keycode).unwrap_or("?");

                    // Convert XKB key name to something user-friendly
                    // XKB names are like "AD01" (row A, position D, key 01) or "AC01", etc.
                    // We need to show the actual character on that key (level 0)
                    let base_sym = keymap.key_get_syms_by_level(keycode, 0, 0);
                    let physical_key = if !base_sym.is_empty() {
                        let base_char = xkb::keysym_to_utf32(base_sym[0]);
                        if let Some(c) = char::from_u32(base_char) {
                            if c.is_ascii_graphic() {
                                c.to_string()
                            } else {
                                key_name.to_string()
                            }
                        } else {
                            key_name.to_string()
                        }
                    } else {
                        key_name.to_string()
                    };

                    // Build the key sequence string (use dash for simultaneous keys)
                    let key_sequence = format!("{}{}", mod_prefix.replace('+', "-"), physical_key);

                    // Try to find a base character to index this under
                    if let Some(base) = find_base_char(ch) {
                        add_entry(&mut index, base, &ch.to_string(), &key_sequence);
                    }
                }
            }
        }

        // Now scan for dead keys and add their combinations
        let mut dead_keys: Vec<(String, String)> = Vec::new(); // (physical_key, dead_key_type)

        for keycode_raw in 8..256 {
            let keycode = xkb::Keycode::new(keycode_raw);

            if keymap.key_get_name(keycode).is_none() {
                continue;
            }

            let num_levels = keymap.num_levels_for_key(keycode, 0);

            for level in 2..num_levels.min(4) {  // Only AltGr levels
                let syms = keymap.key_get_syms_by_level(keycode, 0, level);
                if syms.is_empty() {
                    continue;
                }

                let keysym = syms[0];
                let keysym_name = xkb::keysym_get_name(keysym);

                // Check if this is a dead key
                if keysym_name.starts_with("dead_") {
                    // Get the physical key
                    let base_sym = keymap.key_get_syms_by_level(keycode, 0, 0);
                    let physical_key = if !base_sym.is_empty() {
                        let base_char = xkb::keysym_to_utf32(base_sym[0]);
                        if let Some(c) = char::from_u32(base_char) {
                            if c.is_ascii_graphic() {
                                c.to_string()
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };

                    let mod_prefix = if level == 2 { "AltGr-" } else { "AltGr-Shift-" };
                    dead_keys.push((format!("{}{}", mod_prefix, physical_key), keysym_name.clone()));
                }
            }
        }

        // Add dead key combinations
        for (dead_key_combo, dead_type) in &dead_keys {
            let combinations = get_dead_key_combinations(dead_type);
            for (base_letter, result_char) in combinations {
                // Use space to separate sequential steps
                let key_sequence = format!("{}  {}", dead_key_combo, base_letter);
                if let Some(base) = find_base_char(result_char) {
                    add_entry(&mut index, base, &result_char.to_string(), &key_sequence);
                }
            }
        }

        eprintln!("Found {} base characters with variants", index.len());

        Ok(Self { index })
    }

    /// Find all character variants for a given base character
    pub fn find_variants(&self, input: &str) -> Vec<ComposeEntry> {
        // Get the first character from input
        let base_char = input.chars().next();

        if let Some(ch) = base_char {
            // Try both lowercase and uppercase
            let mut results = Vec::new();

            if let Some(variants) = self.index.get(&ch.to_lowercase().next().unwrap()) {
                results.extend(variants.clone());
            }

            if ch.is_uppercase() {
                if let Some(variants) = self.index.get(&ch.to_uppercase().next().unwrap()) {
                    results.extend(variants.clone());
                }
            }

            results
        } else {
            Vec::new()
        }
    }
}

/// Helper function to add an entry to the index
fn add_entry(
    index: &mut HashMap<char, Vec<ComposeEntry>>,
    base: char,
    character: &str,
    key_sequence: &str,
) {
    let entry = ComposeEntry {
        character: character.to_string(),
        key_sequence: key_sequence.to_string(),
    };

    index
        .entry(base)
        .or_insert_with(Vec::new)
        .push(entry);
}

/// Get common dead key combinations
/// Returns pairs of (base_letter, result_character)
fn get_dead_key_combinations(dead_type: &str) -> Vec<(char, char)> {
    match dead_type {
        "dead_acute" => vec![
            ('a', 'á'), ('e', 'é'), ('i', 'í'), ('o', 'ó'), ('u', 'ú'),
            ('y', 'ý'), ('c', 'ć'), ('n', 'ń'), ('s', 'ś'), ('z', 'ź'),
        ],
        "dead_grave" => vec![
            ('a', 'à'), ('e', 'è'), ('i', 'ì'), ('o', 'ò'), ('u', 'ù'),
        ],
        "dead_circumflex" => vec![
            ('a', 'â'), ('e', 'ê'), ('i', 'î'), ('o', 'ô'), ('u', 'û'),
        ],
        "dead_diaeresis" => vec![
            ('a', 'ä'), ('e', 'ë'), ('i', 'ï'), ('o', 'ö'), ('u', 'ü'), ('y', 'ÿ'),
        ],
        "dead_tilde" => vec![
            ('a', 'ã'), ('n', 'ñ'), ('o', 'õ'),
        ],
        "dead_cedilla" => vec![
            ('c', 'ç'), ('s', 'ş'),
        ],
        "dead_ogonek" => vec![
            ('a', 'ą'), ('e', 'ę'), ('i', 'į'), ('u', 'ų'),
        ],
        "dead_caron" => vec![
            ('c', 'č'), ('s', 'š'), ('z', 'ž'), ('r', 'ř'), ('e', 'ě'),
        ],
        "dead_breve" => vec![
            ('a', 'ă'), ('g', 'ğ'),
        ],
        "dead_macron" => vec![
            ('a', 'ā'), ('e', 'ē'), ('i', 'ī'), ('o', 'ō'), ('u', 'ū'),
        ],
        "dead_abovedot" => vec![
            ('e', 'ė'), ('z', 'ż'),
        ],
        "dead_abovering" => vec![
            ('a', 'å'), ('u', 'ů'),
        ],
        "dead_stroke" => vec![
            ('l', 'ł'), ('o', 'ø'),
        ],
        _ => vec![],
    }
}

/// Find the base character for an accented character
/// e.g., é → e, ñ → n, ø → o
fn find_base_char(ch: char) -> Option<char> {
    // Use Unicode NFD decomposition to strip accents
    use unicode_normalization::UnicodeNormalization;

    let decomposed: String = ch.nfd().collect();
    let base = decomposed.chars().next()?;

    // Only return if it's a letter
    if base.is_alphabetic() && base.is_ascii() {
        Some(base.to_ascii_lowercase())
    } else {
        // For special characters like €, £, etc., try to map to related letters
        match ch {
            '€' => Some('e'),
            '£' => Some('l'),
            '¥' => Some('y'),
            '¢' => Some('c'),
            'æ' => Some('a'),
            'œ' => Some('o'),
            'ß' => Some('s'),
            'ð' => Some('d'),
            'þ' => Some('t'),
            _ => None,
        }
    }
}
